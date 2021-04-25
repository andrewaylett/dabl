use std::io;
use std::io::Write;
use std::net::IpAddr;
use std::ops::Deref;
use std::str::FromStr;

use anyhow::Result;
use structopt::StructOpt;

use libdnscheck::{count_lists, Output, Query};

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
    #[structopt(
        short,
        long,
        conflicts_with = "quiet",
        help = "Output debugging information"
    )]
    /// Output information about every DNS lookup made and every result returned.
    verbose: bool,
    #[structopt(short, long, conflicts_with = "verbose", help = "Only output errors")]
    /// Only output when something fails unexpectedly
    quiet: bool,
    #[structopt(help = "An IP address (v4 or v6) or domain name")]
    /// An IP address (v4 or v6) or domain name to check against the DNS lists provided with -a or -b
    query: String,
}

fn main_() -> Result<()> {
    let args: Arguments = Arguments::from_args_safe()?;

    let output = if args.quiet {
        Output::Quiet
    } else if args.verbose {
        Output::Verbose
    } else {
        Output::Normal
    };

    let allow: Vec<&str> = args.allow.iter().map(Deref::deref).collect();
    let block: Vec<&str> = args.block.iter().map(Deref::deref).collect();

    if output != Output::Quiet {
        println!(
            "Allow: {}\nBlock: {}\nQuery: {}",
            allow.join(", "),
            block.join(", "),
            args.query
        );
    }

    let query: Query = match IpAddr::from_str(&args.query) {
        Ok(ip) => Query::Address(ip),
        _ => Query::Domain(&args.query),
    };

    let allows: Vec<_> = count_lists(&[query], &allow, output)?
        .into_iter()
        .filter(|m| m.found)
        .collect();

    if !allows.is_empty() {
        if output != Output::Quiet {
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

    let blocks: Vec<_> = count_lists(&[query], &block, output)?
        .into_iter()
        .filter(|m| m.found)
        .collect();

    if !blocks.is_empty() {
        let len = blocks.len() as i32;
        if output != Output::Quiet {
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
