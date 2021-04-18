use anyhow::{Context, Result};
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, Values,
};
use lazy_static::lazy_static;
use std::net::IpAddr;
use std::str::FromStr;

use libdnscheck::Output::Normal;
use libdnscheck::{lookup, Query};

lazy_static! {
    // These are the defaults from the Debian package
    static ref BASE_SOURCES: Vec<&'static str> = vec![
        "sbl.spamhaus.org",
        "xbl.spamhaus.org",
        "pbl.spamhaus.org",
        "bl.spamcop.net",
        "psbl.surriel.com",
        "dul.dnsbl.sorbs.net",
    ];
}

fn main() {
    let err = main_().unwrap_err();
    eprintln!("Error: {:?}", err);
    // We exit with a "success" on error, as the error code is the count of the number of lists hit
    // so `0` is the fail-safe
    std::process::exit(0)
}

// From the original rblcheck
//
//     -q           Quiet mode; print only listed addresses
//     -t           Print a TXT record, if any
//     -m           Stop checking after first address match in any list
//     -l           List default DNSBL services to check
//     -c           Clear the current list of DNSBL services
//     -s <service> Toggle a service to the DNSBL services list
//     -h, -?       Display this help message
//     -v           Display version information
//     <address>    An IP address to look up; specify `-' to read multiple
//                  addresses from standard input.

fn main_() -> Result<()> {
    let matches = app_from_crate!()
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
        BASE_SOURCES.clone()
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
            sources.map(move |&source| lookup(source, query, &Normal))
        })
        .fold::<Result<i32>, _>(Ok(0), |r, i| if i? { r.map(|n| n + 1) } else { r })?;

    println!("Hit {} lists", result);

    std::process::exit(result);
}
