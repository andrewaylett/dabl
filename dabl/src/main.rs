#[macro_use]
extern crate clap;

use anyhow::{Context, Result};
use clap::{App, Arg, Values};
use std::net::IpAddr;
use std::str::FromStr;

use libdnscheck::{lookup, Query};

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
        .arg(Arg::with_name("query").multiple(true).min_values(1))
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
        .values_of("query")
        .map(Values::collect)
        .context("Expected at least one query")?;

    println!(
        "Allow: {:?}, Block: {:?}, Queries: {:?}",
        allow_sources, block_sources, params
    );

    let queries: Vec<Query> = params
        .iter()
        .map(|&p| match IpAddr::from_str(p) {
            Ok(ip) => Query::Address(ip),
            _ => Query::Domain(p.to_string()),
        })
        .collect();

    let count_lists = |queries: &Vec<Query>, sources: &Vec<&str>| -> Result<i32> {
        queries
            .iter()
            .flat_map(|query| sources.iter().map(move |&source| lookup(source, query)))
            .fold::<Result<i32>, _>(Ok(0), |r, i| if i? { r.map(|n| n + 1) } else { r })
    };

    let allow_count = count_lists(&queries, &allow_sources)?;
    let block_count = count_lists(&queries, &block_sources)?;

    println!(
        "Hit {} allow lists, {} block lists",
        allow_count, block_count
    );

    if allow_count > 0 {
        std::process::exit(0);
    }

    std::process::exit(block_count);
}
