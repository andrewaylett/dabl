use std::io;
use std::io::Write;
use std::net::IpAddr;
use std::ops::Deref;
use std::str::FromStr;

use anyhow::Result;
use log::*;
use structopt::StructOpt;

use libdnscheck::{count_lists, Query};

fn main() {
    let err = main_().unwrap_err();
    if let Some(err) = err.downcast_ref::<structopt::clap::Error>() {
        let out = io::stdout();
        writeln!(&mut out.lock(), "{}", err.message).expect("Error writing Error to stdout");
    } else {
        eprintln!("Error: {:?}", err);
    }

    // We exit with a "success" on error, as the error code is the count of the number of lists hit
    // so `0` is the fail-safe
    std::process::exit(0)
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct Arguments {
    #[structopt(short, long, number_of_values = 1, help = "A DNS block list")]
    /// The base name of a DNS block list, either IP or domain-based depending on the query you pass in.
    block: Vec<String>,
    #[structopt(short, long, number_of_values = 1, help = "A DNS allow list")]
    /// The base name of a DNS block list, either IP or domain-based depending on the query you pass in.
    /// If an allow list matches, we won't check any block lists.
    allow: Vec<String>,
    /// Silence all output
    #[structopt(
        short = "q",
        long = "quiet",
        conflicts_with = "verbose",
        help = "Silence all output"
    )]
    quiet: bool,
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        conflicts_with = "quiet",
        help = "Output more details"
    )]
    verbose: usize,
    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "t", long = "timestamp")]
    ts: Option<stderrlog::Timestamp>,
    #[structopt(help = "An IP address (v4 or v6) or domain name")]
    /// An IP address (v4 or v6) or domain name to check against the DNS lists provided with -a or -b
    query: String,
}

fn main_() -> Result<()> {
    let args: Arguments = Arguments::from_args_safe()?;

    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose)
        .timestamp(args.ts.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();

    let allow: Vec<&str> = args.allow.iter().map(Deref::deref).collect();
    let block: Vec<&str> = args.block.iter().map(Deref::deref).collect();

    info!(
        "Allow: {}\nBlock: {}\nQuery: {}",
        allow.join(", "),
        block.join(", "),
        args.query
    );

    let query: Query = match IpAddr::from_str(&args.query) {
        Ok(ip) => Query::Address(ip),
        _ => Query::Domain(&args.query),
    };

    let allows: Vec<_> = count_lists(&[query], &allow)?
        .into_iter()
        .filter(|m| m.found)
        .collect();

    if !allows.is_empty() {
        warn!(
            "Found in {} allow lists: {}",
            allows.len(),
            allows
                .into_iter()
                .map(|m| m.list)
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(0)
    }

    let blocks: Vec<_> = count_lists(&[query], &block)?
        .into_iter()
        .filter(|m| m.found)
        .collect();

    if !blocks.is_empty() {
        let len = blocks.len() as i32;
        error!(
            "Found in {} block lists: {}",
            len,
            blocks
                .into_iter()
                .map(|m| m.list)
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(len)
    }

    info!("Not found in any lists");

    std::process::exit(0);
}
