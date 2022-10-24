/// Create a function called sum_int that coerces all arguments to integers and
/// adds them.
use udf::prelude::*;

#[derive(Debug, PartialEq, Eq, Default)]
struct SumInt {}

#[register]
impl BasicUdf for SumInt {
    type Returns<'a> = Option<i64>;

    /// All we do here is set our type coercion. SQL will cancel our function if
    /// the coercion is not possible.
    fn init<'a>(cfg: &mut InitCfg, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        for mut arg in args {
            arg.set_type_coercion(udf::SqlType::Int);
        }
        // This will produce the same result
        cfg.set_const_item(true);
        Ok(Self {})
    }

    /// This is the process
    fn process<'a>(
        &'a mut self,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let mut res = 0;

        // Iterate all arguments
        for arg in args {
            // Try to get the argument as an integer (this should be possible
            // for all args - we set type coercion). If we can get it, add it
            // to our
            match arg.value.as_int() {
                Some(v) => res += v,
                None => return Err(ProcessError),
            }
        }

        // At the end we have a nonnull successful result
        Ok(Some(res))
    }
}
