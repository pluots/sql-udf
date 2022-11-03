//! A pretty useless function that just writes to the server log whenever it's
//! used.
//!
//! # Usage
//!
//! ```sql
//! CREATE FUNCTION log_calls RETURNS integer SONAME 'libudf_examples.so';
//! SELECT log_calls();
//! ```

use udf::prelude::*;

struct LogCalls {}

#[register]
impl BasicUdf for LogCalls {
    type Returns<'a> = Option<i64>;

    fn init<'a>(_cfg: &UdfCfg<Init>, _args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        udf_log!(Note: "called init!");
        Ok(Self {})
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        udf_log!(Note: "called process!");
        Ok(None)
    }
}

#[register]
impl AggregateUdf for LogCalls {
    fn clear(
        &mut self,
        _cfg: &UdfCfg<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        udf_log!(Note: "called clear!");
        Ok(())
    }

    fn add(
        &mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        udf_log!(Note: "called add!");
        Ok(())
    }

    fn remove(
        &mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        udf_log!(Note: "called remove!");
        Ok(())
    }
}
