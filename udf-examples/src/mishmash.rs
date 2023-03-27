//! A dummy function that combines the binary representation of all its
//! arguments
//!
//! # Usage
//!
//! ```sql
//! CREATE FUNCTION mismmash RETURNS string SONAME 'libudf_examples.so';
//! SELECT lipsum(8);
//! ```

use udf::prelude::*;

#[derive(Debug, Default)]
pub struct Mishmash(Vec<u8>);

#[register]
impl BasicUdf for Mishmash {
    type Returns<'a> = Option<&'a [u8]>;

    /// We expect LIPSUM(n) or LIPSUM(n, m)
    fn init(_cfg: &UdfCfg<Init>, _args: &ArgList<Init>) -> Result<Self, String> {
        Ok(Self::default())
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        for arg in args {
            match arg.value() {
                SqlResult::String(Some(v)) => self.0.extend_from_slice(v),
                SqlResult::Real(Some(v)) => self.0.extend_from_slice(&v.to_ne_bytes()),
                SqlResult::Int(Some(v)) => self.0.extend_from_slice(&v.to_ne_bytes()),
                SqlResult::Decimal(Some(v)) => self.0.extend_from_slice(v.as_bytes()),
                other => panic!("unexpected type {other:?}"),
            }
        }

        let res = if self.0.is_empty() {
            None
        } else {
            Some(self.0.as_ref())
        };

        Ok(res)
    }
}
