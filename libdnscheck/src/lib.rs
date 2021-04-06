use dbus::blocking::Connection;
use std::net::{IpAddr, Ipv6Addr};
use std::time::Duration;
use thiserror::Error;

use libdbusdnscheck::OrgFreedesktopResolve1Manager;


#[derive(Debug)]
pub enum Query {
    IP(IpAddr),
    Domain(String),
}

#[derive(Error,Debug)]
pub enum DNSCheckError {
    #[error("Problem resolving with DBus: {0}")]
    DBus (
        #[from]
        dbus::Error,
    ),
    #[error("Something went wrong")]
    Unknown,
}

pub fn lookup(source: &str, query: &Query) -> Result<bool, DNSCheckError> {
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
