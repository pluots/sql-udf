//! A very simple function that checks whether an argument is const or not
//!
//! Functionality is simple: check for constness in `init` (the only time this
//! is possible), save the result in the struct, and return it in `process`

use udf::prelude::*;

#[derive(Debug)]
struct IsConst {
    is_const: bool,
}

#[register]
impl BasicUdf for IsConst {
    type Returns<'a> = &'static str;

    fn init<'a>(_cfg: &UdfCfg<Init>, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err("IS_CONST only accepts one argument".to_owned());
        }

        // Get the first argument, check if it is const, and store it in our
        // struct
        Ok(Self {
            is_const: args.get(0).unwrap().is_const(),
        })
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        // Just return a result based on our init step
        Ok(if self.is_const { "const" } else { "not const" })
    }
}

#[cfg(all(test, sql_integration))]
mod int_tests {}
