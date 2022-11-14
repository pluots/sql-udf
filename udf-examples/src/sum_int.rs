//! Create a function called `sum_int` that coerces all arguments to integers and
//! adds them. Accepts any number of arguments.
//!
//! # Usage
//!
//! ```sql
//! CREATE FUNCTION sum_int RETURNS integer SONAME 'libudf_examples.so';
//! SELECT sum_int(1, 2, 3, 4, '5', 6.2)
//! ```

use udf::prelude::*;

#[derive(Debug, PartialEq, Eq, Default)]
struct SumInt {}

#[register]
impl BasicUdf for SumInt {
    type Returns<'a> = i64;

    /// All we do here is set our type coercion. SQL will cancel our function if
    /// the coercion is not possible.
    fn init<'a>(cfg: &UdfCfg<Init>, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        // Coerce each arg to an integer
        args.iter()
            .for_each(|mut arg| arg.set_type_coercion(udf::SqlType::Int));

        // This will produce the same result
        cfg.set_is_const(true);
        Ok(Self {})
    }

    /// This is the process
    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        // Iterate all arguments, sum all that are integers. This should
        // be all of them, since we set coercion
        Ok(args.iter().filter_map(|arg| arg.value().as_int()).sum())

        // If you're not familiar with rust's combinators, here's the for loop
        // version:
        // let mut res = 0;
        // for arg in args {
        //     if let Some(v) = arg.value.as_int() {
        //         res += v
        //     }
        // }

        // Ok(res)
    }
}
