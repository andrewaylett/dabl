use std::io;
use std::io::Write;
use std::net::IpAddr;
use std::ops::Deref;
use std::str::FromStr;

use anyhow::Result;
use lazy_static::lazy_static;
use structopt::StructOpt;

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
#[derive(Debug, StructOpt)]
#[structopt()]
struct Arguments {
    #[structopt(short, help = "Quiet mode; print only listed addresses")]
    quiet: bool,
    #[structopt(short, help = "Print a TXT record, if any")]
    text: bool,
    #[structopt(short, help = "Stop checking after first address match in any list")]
    match_one: bool,
    #[structopt(short, help = "List default DNSBL services to check")]
    list: bool,
    #[structopt(short, help = "Clear the current list of DNSBL services")]
    clear: bool,
    #[structopt(
        short,
        number_of_values = 1,
        help = "Toggle a service to the DNSBL services list"
    )]
    services: Vec<String>,
    #[structopt(
        help = "An IP address to look up; specify `-' to read multiple addresses from standard input"
    )]
    addresses: Vec<String>,
}

fn main_() -> Result<()> {
    let args: Arguments = Arguments::from_args_safe()?;

    let base_sources = if args.clear {
        vec![]
    } else {
        BASE_SOURCES.clone()
    };

    let extra_sources: Vec<&str> = args.services.iter().map(Deref::deref).collect();

    let params: Vec<&str> = args.addresses.iter().map(Deref::deref).collect();

    println!(
        "Base: {:?}, Extra: {:?}, Queries: {:?}",
        base_sources, extra_sources, params
    );

    let queries: Vec<Query> = params
        .iter()
        .map(|&p| match IpAddr::from_str(p) {
            Ok(ip) => Query::Address(ip),
            _ => Query::Domain(p),
        })
        .collect();

    let result = queries
        .iter()
        .flat_map(|query| {
            let sources = base_sources.iter().chain(extra_sources.iter());
            sources.map(move |&source| lookup(source, query, &Normal))
        })
        .fold::<Result<i32>, _>(Ok(0), |r, i| if i?.found { r.map(|n| n + 1) } else { r })?;

    println!("Hit {} lists", result);

    std::process::exit(result);
}
