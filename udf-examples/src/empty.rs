//! This function is the bare minimum to do literally nothing

use udf::prelude::*;

struct EmptyCall;

#[register]
impl BasicUdf for EmptyCall {
    type Returns<'a> = Option<i64>;

    fn init(_cfg: &UdfCfg<Init>, _args: &ArgList<Init>) -> Result<Self, String> {
        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        Ok(None)
    }
}
