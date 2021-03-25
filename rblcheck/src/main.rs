#[macro_use]
extern crate clap;

use anyhow::{Context, Result};
use clap::{App, Arg, Values};
use dbus::blocking::Connection;
use std::net::{IpAddr, Ipv6Addr};
use std::str::FromStr;
use std::time::Duration;

use libdbusdnscheck::OrgFreedesktopResolve1Manager;

const BASE_SOURCES: Vec<&str> = vec![];

#[derive(Debug)]
enum Query {
    IP(IpAddr),
    Domain(String),
}

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

    let base_sources = if let Some(_) = matches.value_of("clear") {
        vec![]
    } else {
        BASE_SOURCES
    };

    let extra_sources = matches
        .values_of("source")
        .map(Values::collect)
        .unwrap_or(vec![]);

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
            Ok(ip) => Query::IP(ip),
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

fn lookup(source: &str, query: &Query) -> Result<bool> {
    println!("Source: {:?}, Query: {:?}", source, query);

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.freedesktop.resolve1",
        "/org/freedesktop/resolve1",
        Duration::from_secs(30),
    );

    let queryhost = match query {
        Query::Domain(d) => format!("{}.", d),
        Query::IP(ip) => format_ip(&ip),
    };

    let hostname = format!("{}{}.", queryhost, source);

    println!("Querying: {}", hostname);

    let result: (Vec<(i32, i32, Vec<u8>)>, String, u64) =
        proxy.resolve_hostname(0, &hostname, libc::AF_INET, 0)?;

    println!("Result: {:?}", result);

    Ok(!result.0.is_empty())
}

fn format_ip(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => format!(
            "{}.{}.{}.{}.",
            v4.octets()[3],
            v4.octets()[2],
            v4.octets()[1],
            v4.octets()[0]
        ),
        IpAddr::V6(v6) => format_v6(v6),
    }
}

fn format_v6(ip: &Ipv6Addr) -> String {
    ip.octets()
        .iter()
        .flat_map(|o| vec![o >> 4, o & 0xF])
        .map(|d| format!("{:x}", d))
        .fold("".to_owned(), |a: String, d: String| format!("{}.{}", d, a))
}
