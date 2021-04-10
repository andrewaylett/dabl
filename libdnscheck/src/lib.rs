use dbus::blocking::Connection;
use dbus::MethodErr;
use std::net::{IpAddr, Ipv6Addr};
use std::time::Duration;
use thiserror::Error;

use libdbusdnscheck::OrgFreedesktopResolve1Manager;

#[derive(Debug)]
pub enum Query {
    Address(IpAddr),
    Domain(String),
}

#[derive(Error, Debug)]
pub enum DnsCheckError {
    #[error("DBus reported {0}: {1}")]
    DBus(String, String),
    #[error("NXDOMAIN {0}")]
    NxDomain(String),
    #[error("Something went wrong")]
    Unknown,
}

impl From<MethodErr> for DnsCheckError {
    fn from(e: MethodErr) -> Self {
        if e.errorname()
            .starts_with("org.freedesktop.resolve1.DnsError.NXDOMAIN")
        {
            DnsCheckError::NxDomain(e.description().to_string())
        } else {
            DnsCheckError::DBus(e.errorname().to_string(), e.description().to_string())
        }
    }
}

impl From<dbus::Error> for DnsCheckError {
    fn from(error: dbus::Error) -> Self {
        DnsCheckError::from(MethodErr::from(error))
    }
}

pub fn lookup(source: &str, query: &Query) -> Result<bool, DnsCheckError> {
    println!("Source: {:?}, Query: {:?}", source, query);

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.freedesktop.resolve1",
        "/org/freedesktop/resolve1",
        Duration::from_secs(30),
    );

    let queryhost = match query {
        Query::Domain(d) => format!("{}.", d),
        Query::Address(ip) => format_ip(&ip),
    };

    let hostname = format!("{}{}.", queryhost, source);

    println!("Querying: {}", hostname);

    type DBusDnsResponse = (Vec<(i32, i32, Vec<u8>)>, String, u64);
    let result: Result<DBusDnsResponse, DnsCheckError> = proxy
        .resolve_hostname(0, &hostname, libc::AF_INET, 0)
        .map_err(From::from);

    println!("Result: {:?}", result);

    result.map_or_else(
        |error| match error {
            DnsCheckError::NxDomain(_) => Ok(false),
            e => Err(e),
        },
        |r| Ok(!r.0.is_empty()),
    )
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
