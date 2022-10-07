//! sequence() function
//!
//! Start at a given number if an argument is given

use udf::prelude::*;

struct Sequence {
    last_val: i64,
}

impl BasicUdf for Sequence {
    type Returns<'a> = i64
    where
        Self: 'a;

    /// Init just validates the argument count and initializes our empty struct
    fn init<'a>(cfg: &mut InitCfg, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() > 1 {
            return Err("This function takes 0 or 1 arguments".to_owned());
        }
        if let Some(mut a) = args.get(0) {
            a.set_type_coercion(SqlType::Int);
        }
        cfg.set_const_item(false);
        Ok(Self { last_val: 0 })
    }

    fn process<'a>(
        &'a mut self,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let val = match args.get(0) {
            Some(v) => v.value.as_int().unwrap(),
            None => 0,
        };
        self.last_val += 1;
        Ok(self.last_val + val)
    }
}
