#[macro_use]
extern crate clap;

use anyhow::{Context, Result};
use clap::{App, Arg, Values};
use std::net::IpAddr;
use std::str::FromStr;

use libdnscheck::Output::{Normal, Quiet, Verbose};
use libdnscheck::{count_lists, Output, Query};

fn main() {
    let err = main_().unwrap_err();
    eprintln!("Error: {:?}", err);
    // We exit with a "success" on error, as the error code is the count of the number of lists hit
    // so `0` is the fail-safe
    std::process::exit(0)
}

// Argument Names
const IP_BLOCK: &str = "IP Block";
const IP_ALLOW: &str = "IP Allow";
const STANDARD: &str = "Standard";
const SPAMHAUS_KEY: &str = "SpamHaus key";
const QUIET: &str = "Quiet";
const VERBOSE: &str = "Verbose";
const QUERY: &str = "Query";

fn main_() -> Result<()> {
    let matches = App::new("dabl")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Queries DNS Allow- or Block-Lists")
        .arg(
            Arg::with_name(IP_BLOCK)
                .short("b")
                .long("block")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .help("Specify a block list to use"),
        )
        .arg(
            Arg::with_name(IP_ALLOW)
                .short("a")
                .long("allow")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .help("Specify an allow list to use"),
        )
        .arg(
            Arg::with_name(STANDARD)
                .long("standard")
                .help("Use the built-in 'standard' set of lists"),
        )
        .arg(
            Arg::with_name(SPAMHAUS_KEY)
                .long("spamhaus-key")
                .takes_value(true)
                .help("Your SpamHaus subscription key"),
        )
        .arg(
            Arg::with_name(QUIET)
                .short("q")
                .help("Only output on error"),
        )
        .arg(
            Arg::with_name(VERBOSE)
                .short("v")
                .help("Output more information about what's happening"),
        )
        .arg(
            Arg::with_name(QUERY)
                .multiple(false)
                .number_of_values(1)
                .help("An IP address or domain name to check"),
        )
        .get_matches_safe()?;

    let allow_sources: Vec<&str> = matches
        .values_of(IP_ALLOW)
        .map(Values::collect)
        .unwrap_or_default();

    let block_sources: Vec<&str> = matches
        .values_of(IP_BLOCK)
        .map(Values::collect)
        .unwrap_or_default();

    let params: Vec<&str> = matches
        .values_of(QUERY)
        .map(Values::collect)
        .context("Expected a query")?;

    let output = if matches.is_present(QUIET) {
        Output::Quiet
    } else if matches.is_present(VERBOSE) {
        Verbose
    } else {
        Normal
    };

    if output != Quiet {
        println!(
            "Allow: {:?}, Block: {:?}, Queries: {:?}",
            allow_sources, block_sources, params
        );
    }

    let queries: Vec<Query> = params
        .iter()
        .map(|&p| match IpAddr::from_str(p) {
            Ok(ip) => Query::Address(ip),
            _ => Query::Domain(p.to_string()),
        })
        .collect();

    let allows: Vec<_> = count_lists(&queries, &allow_sources, output)?
        .into_iter()
        .filter(|m| m.found)
        .collect();

    if !allows.is_empty() {
        if output != Quiet {
            println!(
                "Found in {} allow lists: {}",
                allows.len(),
                allows
                    .into_iter()
                    .map(|m| m.list)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        std::process::exit(0)
    }

    let blocks: Vec<_> = count_lists(&queries, &block_sources, output)?
        .into_iter()
        .filter(|m| m.found)
        .collect();

    if !blocks.is_empty() {
        let len = blocks.len() as i32;
        if output != Quiet {
            println!(
                "Found in {} block lists: {}",
                len,
                blocks
                    .into_iter()
                    .map(|m| m.list)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        std::process::exit(len)
    }

    std::process::exit(0);
}
