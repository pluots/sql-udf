
//! Implement a simple UUID

use udf::prelude::*;

/// We have no data to share among
struct Metaphon {}

const MAXMETAPH:u32=8;

impl BasicUdf for Metaphon {
    type Returns<'a> = Option<String>;

    /// The only thing to validate here is that we have no arguments
    fn init(cfg: &mut InitCfg, args: &ArgList<Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err(format!("One argument expected; {} received", args.len()));
        }

        cfg.set_max_len(0);

        Ok(Self {})
    }

    /// Just create a v4 UUID and return it
    fn process<'a>(
        &'a mut self,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let arg = args.get(0).unwrap().value.as_str().unwrap();
        if arg.len()==0 {
            return Ok(None);
        }

        todo!()
    }
}
