//! Aggregate cost window function
//!
//! Takes a quantity and a real to return the average value within the window.
//!
//! ```
//! CREATE FUNCTION avg2 RETURNS integer SONAME 'libudf_examples.so';
//! SELECT avg2(int_column, real_column);
//! ```

use udf::prelude::*;

// We reuse our errors from avgcost here, for convenience
use crate::avgcost::Errors;

#[derive(Debug, Default)]
struct Avg2 {
    count: u64,
    sum: f64,
}

// #[register]
impl BasicUdf for Avg2 {
    type Returns<'a> = Option<f64>;

    fn init<'a>(cfg: &UdfCfg<Init>, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() != 2 {
            return Err(Errors::WrongArgCount(args.len()).to_string());
        }

        let a0 = args.get(0).unwrap();
        let a1 = args.get(1).unwrap();

        if !a0.value.is_int() {
            return Err(Errors::FirstArgType(&a0).to_string());
        }
        if !a1.value.is_real() {
            return Err(Errors::SecondArgType(&a1).to_string());
        }

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

#[register]
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
        let in_qty;
        let in_sum;

        if let Some(q) = args.get(0).unwrap().value.as_int() {
            in_qty = q;
        } else {
            return Ok(());
        };

        if let Some(s) = args.get(1).unwrap().value.as_real() {
            in_sum = s;
        } else {
            return Ok(());
        }

        self.count += in_qty as u64;
        self.sum += in_sum;

        Ok(())
    }

    /// For MariaDB only:
    fn remove(
        &mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        let in_qty;
        let in_sum;

        if let Some(q) = args.get(0).unwrap().value.as_int() {
            in_qty = q;
        } else {
            return Ok(());
        };

        if let Some(s) = args.get(1).unwrap().value.as_real() {
            in_sum = s;
        } else {
            return Ok(());
        }

        self.count -= in_qty as u64;
        self.sum -= in_sum;

        Ok(())
    }
}
