//! Function to create a sequence
//!
//! This will return an incrementing value for each row, starting with 1 by
//! default, or any given
//!
//! ```
//! CREATE FUNCTION udf_sequence RETURNS integer SONAME 'libudf_examples.so';
//! SELECT some_col, sequence() from some_table;
//! SELECT some_col, sequence(8) from some_table;
//! ```

use udf::prelude::*;

struct UdfSequence {
    last_val: i64,
}

#[register]
impl BasicUdf for UdfSequence {
    type Returns<'a> = i64
    where
        Self: 'a;

    /// Init just validates the argument count and initializes our empty struct
    fn init<'a>(cfg: &UdfCfg<Init>, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() > 1 {
            return Err(format!(
                "This function takes 0 or 1 arguments; got {}",
                args.len()
            ));
        }

        // If we have an argument, set its type coercion to an integer
        if let Some(mut a) = args.get(0) {
            a.set_type_coercion(SqlType::Int);
        }

        // Result will differ for each call
        cfg.set_is_const(false);
        Ok(Self { last_val: 0 })
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        // If we have an argument, that will provide our base value
        let arg_val = match args.get(0) {
            Some(v) => v.value.as_int().unwrap(),
            None => 0,
        };

        // Increment our last value, return the total
        self.last_val += 1;
        Ok(self.last_val + arg_val)
    }
}
