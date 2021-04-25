use std::fmt::{Display, Formatter};
use std::net::{IpAddr, Ipv6Addr};
use std::{fmt, io};

use dns_lookup::{getaddrinfo, LookupErrorKind};
use thiserror::Error;

use crate::dbus::lookup_dbus;
use crate::DnsCheckError::{NoDBus, NoResolved};

#[cfg(all(feature = "dbus", target_os = "linux"))]
mod dbus;

#[derive(Debug, Copy, Clone)]
pub enum Query<'a> {
    Address(IpAddr),
    Domain(&'a str),
}

impl Display for Query<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Query::Address(addr) => {
                write!(f, "{}", addr)
            }
            Query::Domain(domain) => {
                write!(f, "{}", domain)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum DnsCheckError {
    #[error("DBus reported {0}: {1}")]
    DBus(String, String),
    #[error("DBus support is missing")]
    NoDBus,
    #[error("systemd-resolved not found: {0}")]
    NoResolved(#[source] anyhow::Error),
    #[error("NXDOMAIN {0}")]
    NxDomain(String),
    #[error("Something went wrong: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl From<io::Error> for DnsCheckError {
    fn from(e: io::Error) -> Self {
        DnsCheckError::Unknown(e.into())
    }
}

pub struct DnsListMembership {
    pub name: String,
    pub list: String,
    pub found: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Output {
    Quiet,
    Normal,
    Verbose,
}

pub fn lookup(
    source: &str,
    query: &Query,
    output: &Output,
) -> Result<DnsListMembership, DnsCheckError> {
    match lookup_dbus(source, query, output) {
        Ok(r) => return Ok(r),
        Err(NoDBus) => {
            if output == &Output::Verbose {
                eprintln!("DBus not compiled in, falling back to internal resolution")
            }
        }
        Err(NoResolved(e)) => {
            if output == &Output::Verbose {
                eprintln!("DBus resolution failed: {:?}", e)
            }
        }
        Err(e) => return Err(e),
    };

    lookup_dns(source, query, output)
}

fn lookup_dns(
    source: &str,
    query: &Query,
    _output: &Output,
) -> Result<DnsListMembership, DnsCheckError> {
    let queryhost = match query {
        Query::Domain(d) => format!("{}.", d),
        Query::Address(ip) => format_ip(&ip),
    };

    let hostname = format!("{}{}.", queryhost, source);

    let mut addrinfo = match getaddrinfo(Some(&hostname), None, None) {
        Ok(a) => Ok(a),
        Err(e) => match e.kind() {
            LookupErrorKind::NoName => {
                return Ok(DnsListMembership {
                    name: query.to_string(),
                    list: format!("{:?}", source),
                    found: false,
                })
            }
            _ => Err(DnsCheckError::Unknown(io::Error::from(e).into())),
        },
    }?;

    Ok(DnsListMembership {
        name: source.to_string(),
        list: format!("{}", query),
        found: addrinfo.next().is_some(),
    })
}

#[cfg(not(all(feature = "dbus", target_os = "linux")))]
mod dbus {
    use crate::DnsCheckError::NoDBus;
    use crate::{DnsCheckError, DnsListMembership, Output, Query};

    pub fn lookup_dbus(_: &str, _: &Query, _: &Output) -> Result<DnsListMembership, DnsCheckError> {
        Err(NoDBus)
    }
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

pub fn count_lists(
    queries: &[Query],
    sources: &[&str],
    output: Output,
) -> Result<Vec<DnsListMembership>, DnsCheckError> {
    queries
        .iter()
        .flat_map(|query| {
            sources
                .iter()
                .map(move |&source| lookup(source, query, &output))
        })
        .collect()
}
