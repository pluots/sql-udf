//! Function to create a sequence
//!
//! This will return an incrementing value for each row, starting with 1 by
//! default, or any given
//!
//! ```sql
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
    fn init(cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
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
        // If we have an argument, that will provide our offset value
        let arg_val = match args.get(0) {
            Some(v) => v.value().as_int().unwrap(),
            None => 0,
        };

        // Increment our last value, return the total
        self.last_val += 1;
        Ok(self.last_val + arg_val)
    }
}

#[cfg(test)]
mod tests {
    use udf::mock::*;

    use super::*;

    #[test]
    fn test_init() {
        // Not really anything to test here
        let mut mock_cfg = MockUdfCfg::new();
        let mut mock_args = mock_args![(Int 1, "", false)];

        assert!(UdfSequence::init(mock_cfg.as_init(), mock_args.as_init()).is_ok());
    }

    #[test]
    fn test_process() {
        // Test with some random arguments
        let mut inited = UdfSequence { last_val: 0 };
        let mut mock_cfg = MockUdfCfg::new();
        let mut arglist: Vec<(_, i64)> = vec![
            (mock_args![(Int 0, "", false)], 1),
            (mock_args![(Int 0, "", false)], 2),
            (mock_args![(Int 0, "", false)], 3),
        ];

        for (arg, expected) in arglist.iter_mut() {
            let res =
                UdfSequence::process(&mut inited, mock_cfg.as_process(), arg.as_process(), None)
                    .unwrap();
            assert_eq!(res, *expected)
        }
    }
}
