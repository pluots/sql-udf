//! Lookup hostname to IPv6 conversion
//!
//! # Usage
//!
//! ```sql
//! CREATE FUNCTION lookup6 RETURNS string SONAME 'libudf_examples.so';
//! SELECT lookup6('0.0.0.0');
//! ```

#![allow(unused)]

use std::net::{SocketAddr, ToSocketAddrs};

use udf::prelude::*;

/// No data to persist
#[derive(Debug)]
struct Lookup6;

const IPV6_MAX_LEN: u64 = 39;

#[register]
impl BasicUdf for Lookup6 {
    type Returns<'a> = Option<String>
    where
        Self: 'a;

    fn init<'a>(cfg: &UdfCfg<Init>, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err(format!("Expected 1 argument; got {}", args.len()));
        }

        let arg_val = args.get(0).unwrap().value();

        if !arg_val.is_string() {
            return Err(format!(
                "Expected string argument; got {}",
                arg_val.display_name()
            ));
        }

        // max ipv6 address with colons
        cfg.set_max_len(IPV6_MAX_LEN);
        cfg.set_maybe_null(true);

        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let arg = args.get(0).unwrap().value();

        let Some(hostname) = arg.as_string()  else {
            return Err(ProcessError);
        };

        // `to_socket_addrs` checks the given hostname and port (0) and returns
        // an iterator over all valid resolutions
        let Ok(mut sock_addrs) = (hostname, 0).to_socket_addrs() else { return Ok(None) };

        // Prioritize an ipv6 address if it is available, take first address if
        // not.
        let first = sock_addrs.next();

        let Some(ret_sock_addr) = sock_addrs.find(SocketAddr::is_ipv6).or(first) else { return Ok(None) };

        // Get an ipv6 version
        let ret_addr = match ret_sock_addr {
            SocketAddr::V4(a) => a.ip().to_ipv6_mapped(),
            SocketAddr::V6(a) => *a.ip(),
        };

        Ok(Some(ret_addr.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use udf::mock::*;

    use super::*;

    #[test]
    fn test_init_ok() {
        let mut mock_cfg = MockUdfCfg::new();
        let mut mock_args = mock_args![("localhost", "attr1", true)];
        let res = Lookup6::init(mock_cfg.build_init(), mock_args.build_init());

        assert_eq!(*mock_cfg.max_len(), IPV6_MAX_LEN);
        assert!(res.is_ok());
    }

    #[test]
    fn test_init_wrong_arg_count() {
        let mut mock_cfg = MockUdfCfg::new();
        let mut mock_args = mock_args![("localhost", "attr1", true), ("localhost", "attr2", true)];
        let res = Lookup6::init(mock_cfg.build_init(), mock_args.build_init());

        assert_eq!(res.unwrap_err(), "Expected 1 argument; got 2");
    }

    #[test]
    fn test_init_wrong_arg_type() {
        let mut mock_cfg = MockUdfCfg::new();
        let mut mock_args = mock_args![(Int 500, "attr1", true)];
        let res = Lookup6::init(mock_cfg.build_init(), mock_args.build_init());

        assert_eq!(res.unwrap_err(), "Expected string argument; got int");
    }

    #[test]
    #[cfg(not(miri))] // need to skip Miri because. it can't cross FFI
    fn process() {
        // Test with some random arguments
        let mut inited = Lookup6;
        let mut mock_cfg = MockUdfCfg::new();
        let mut mock_args = mock_args![("localhost", "attr1", false),];

        let res = Lookup6::process(
            &mut inited,
            mock_cfg.build_process(),
            mock_args.build_process(),
            None,
        );

        // Our result can be weird, so we just check it has a colon
        assert!(res.unwrap().unwrap().contains(':'));
    }
}
