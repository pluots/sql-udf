//! Example average function
//!
//! This is just a reimplemenation of builtin `AVG`
//!
//! ```sql
//! CREATE FUNCTION avg2 RETURNS integer SONAME 'libudf_examples.so';
//! SELECT avg2(value);
//! ```
// Ignore loss of precision when we cast i64 to f64
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]

use udf::prelude::*;

#[derive(Debug, Default)]
struct Avg2 {
    count: u64,
    sum: f64,
}

#[register(alias = "test_avg2_alias")]
impl BasicUdf for Avg2 {
    type Returns<'a> = Option<f64>;

    fn init(cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.len() != 1 {
            return Err(format!(
                "this function expected 1 argument; got {}",
                args.len()
            ));
        }

        let mut a0 = args.get(0).unwrap();
        a0.set_type_coercion(SqlType::Real);

        cfg.set_maybe_null(true);
        cfg.set_decimals(10);
        cfg.set_max_len(20);

        Ok(Self::default())
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        if self.count == 0 {
            return Ok(None);
        }

        Ok(Some(self.sum / self.count as f64))
    }
}

#[register(alias = "test_avg2_alias")]
impl AggregateUdf for Avg2 {
    fn clear(
        &mut self,
        _cfg: &UdfCfg<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        *self = Self::default();
        Ok(())
    }

    fn add(
        &mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        self.count += 1;
        self.sum += args.get(0).unwrap().value().as_real().unwrap();

        Ok(())
    }

    /// For MariaDB only:
    fn remove(
        &mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        self.count -= 1;
        self.sum -= args.get(0).unwrap().value().as_real().unwrap();

        Ok(())
    }
}
