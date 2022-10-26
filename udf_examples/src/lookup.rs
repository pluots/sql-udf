//! Lookup and reverse lookup hostname/IP conversions

#![allow(unused)]

use std::net::{SocketAddr, ToSocketAddrs};

use udf::prelude::*;

struct Lookup6;
struct ReverseLookup;

#[register]
impl BasicUdf for Lookup6 {
    type Returns<'a> = Option<String>
    where
        Self: 'a;

    fn init<'a>(cfg: &UdfCfg<Init>, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err(format!("Expected 1 argument; got {}", args.len()));
        }

        let arg_val = args.get(0).unwrap().value;

        if !arg_val.is_string() {
            return Err(format!(
                "Expected string argument; got {}",
                arg_val.display_name()
            ));
        }

        // max ipv6 address with colons
        cfg.set_max_len(39);
        cfg.set_maybe_null(true);

        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let arg = args.get(0).unwrap();

        let hostname = if let Some(v) = arg.value.as_string() {
            v
        } else {
            return Err(ProcessError);
        };

        // `to_socket_addrs` checks the given hostname and port (0) and returns
        // an iterator over all valid resolutions
        let mut sock_addrs = match (hostname, 0).to_socket_addrs() {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        // Prioritize an ipv6 address if it is available, take first address if
        // not. Return
        let ret_sock_addr = match sock_addrs
            .find(SocketAddr::is_ipv6)
            .or_else(|| sock_addrs.next())
        {
            Some(v) => v,
            None => return Ok(None),
        };

        // Get an ipv6 version
        let ret_addr = match ret_sock_addr {
            SocketAddr::V4(a) => a.ip().to_ipv6_mapped(),
            SocketAddr::V6(a) => *a.ip(),
        };

        Ok(Some(ret_addr.to_string()))
    }
}
