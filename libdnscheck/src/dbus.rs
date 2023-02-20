use dbus::blocking::Connection;
use dbus::{Error as DBusError, MethodErr};

use generate_dbus_resolve1::OrgFreedesktopResolve1Manager;

use crate::{format_ip, DnsCheckError, DnsListMembership, Output, Query};
use log::*;
use std::time::Duration;

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

impl From<DBusError> for DnsCheckError {
    fn from(error: DBusError) -> Self {
        MethodErr::from(error).into()
    }
}

pub fn lookup_dbus(
    source: &str,
    query: &Query,
    output: &Output,
) -> Result<DnsListMembership, DnsCheckError> {
    info!("Source: {:?}, Query: {:?}", source, query);

    let conn = Connection::new_system().map_err(|e| DnsCheckError::NoResolved(e.into()))?;
    let proxy = conn.with_proxy(
        "org.freedesktop.resolve1",
        "/org/freedesktop/resolve1",
        Duration::from_secs(30),
    );

    let queryhost = match query {
        Query::Domain(d) => format!("{}.", d),
        Query::Address(ip) => format_ip(ip),
    };

    let hostname = format!("{}{}.", queryhost, source);

    debug!("Querying: {}", hostname);

    type DBusDnsResponse = (Vec<(i32, i32, Vec<u8>)>, String, u64);
    let result: Result<DBusDnsResponse, DnsCheckError> = proxy
        .resolve_hostname(0, &hostname, libc::AF_INET, 0)
        .map_err(From::from);

    debug!("Result: {:?}", result);

    result.map_or_else(
        |error| match error {
            DnsCheckError::NxDomain(_) => Ok(DnsListMembership {
                name: format!("{}", query),
                list: source.to_string(),
                found: false,
            }),
            e => Err(e),
        },
        |r| {
            Ok(DnsListMembership {
                name: format!("{}", query),
                list: source.to_string(),
                found: !r.0.is_empty(),
            })
        },
    )
}
