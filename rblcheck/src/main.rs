#[macro_use]
extern crate clap;

use anyhow::{Context, Result};
use clap::{App, Arg, Values};
use std::net::IpAddr;
use std::str::FromStr;

use libdnscheck::{lookup, Query};

const BASE_SOURCES: Vec<&str> = vec![];

fn main() {
    let err = main_().unwrap_err();
    eprintln!("Error: {:?}", err);
    // We exit with a "success" on error, as the error code is the count of the number of lists hit
    // so `0` is the fail-safe
    std::process::exit(0)
}

fn main_() -> Result<()> {
    let matches = App::new("rblcheck")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Queries DNS block-lists (or allow lists!)")
        .arg(
            Arg::with_name("clear")
                .short("c")
                .requires("source")
                .help("Clear the built-in list"),
        )
        .arg(
            Arg::with_name("source")
                .short("s")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .help("Specify a list to use"),
        )
        .arg(Arg::with_name("query").multiple(true).min_values(1))
        .get_matches_safe()?;

    let base_sources = if matches.value_of("clear").is_some() {
        vec![]
    } else {
        BASE_SOURCES
    };

    let extra_sources: Vec<&str> = matches
        .values_of("source")
        .map(Values::collect)
        .unwrap_or_default();

    let params: Vec<&str> = matches
        .values_of("query")
        .map(Values::collect)
        .context("Expected at least one query")?;

    println!(
        "Base: {:?}, Extra: {:?}, Queries: {:?}",
        base_sources, extra_sources, params
    );

    let queries: Vec<Query> = params
        .iter()
        .map(|&p| match IpAddr::from_str(p) {
            Ok(ip) => Query::Address(ip),
            _ => Query::Domain(p.to_string()),
        })
        .collect();

    let result = queries
        .iter()
        .flat_map(|query| {
            let sources = base_sources.iter().chain(extra_sources.iter());
            sources.map(move |&source| lookup(source, query))
        })
        .fold::<Result<i32>, _>(Ok(0), |r, i| if i? { r.map(|n| n + 1) } else { r })?;

    println!("Hit {} lists", result);

    std::process::exit(result);
}
