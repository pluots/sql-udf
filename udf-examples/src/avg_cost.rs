//! Aggregate cost function
//!
//! Takes a quantity and a price to return the average cost within the group
//!
//! # Usage
//!
//! ```sql
//! CREATE AGGREGATE FUNCTION avg_cost RETURNS real SONAME 'libudf_examples.so';
//! SELECT avg_cost(int_column, real_column);
//! ```

#![allow(clippy::cast_precision_loss)]

use udf::prelude::*;

#[derive(Debug, Default, PartialEq)]
struct AvgCost {
    count: usize,
    total_qty: i64,
    total_price: f64,
}

#[register]
impl BasicUdf for AvgCost {
    type Returns<'a> = Option<f64>
    where
        Self: 'a;

    fn init(cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        if args.len() != 2 {
            return Err(format!("expected two arguments; got {}", args.len()));
        }

        let mut a0 = args.get(0).unwrap();
        let mut a1 = args.get(1).unwrap();

        a0.set_type_coercion(SqlType::Int);
        a1.set_type_coercion(SqlType::Real);

        cfg.set_maybe_null(true);
        cfg.set_decimals(10);
        cfg.set_max_len(20);

        // Derived default just has 0 at all fields
        Ok(Self::default())
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        if self.count == 0 || self.total_qty == 0 {
            return Ok(None);
        }
        dbg!(Ok(Some(self.total_price / self.total_qty as f64)))
    }
}

#[register]
impl AggregateUdf for AvgCost {
    fn clear(
        &mut self,
        _cfg: &UdfCfg<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        // Reset our struct and return
        *self = Self::default();
        Ok(())
    }

    fn add(
        &mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        // We can unwrap because we are guaranteed to have 2 args (from checks in init)
        let in_qty = args.get(0).unwrap().value().as_int().unwrap();
        let mut price = args.get(1).unwrap().value().as_real().unwrap();

        dbg!(&in_qty, &price, &self);

        self.count += 1;

        if (self.total_qty >= 0 && in_qty < 0) || (self.total_qty < 0 && in_qty > 0) {
            // Case where given quantity has an opposite sign from our current quantity
            let newqty = self.total_qty + in_qty;

            if !((in_qty < 0 && newqty < 0) || (in_qty > 0 && newqty > 0)) {
                // If we will be switching from - to +,
                price = self.total_price / self.total_qty as f64;
            }

            self.total_price = price * newqty as f64;
        } else {
            // Normal case
            self.total_qty += in_qty;
            self.total_price += price * in_qty as f64;
        }

        if self.total_qty == 0 {
            self.total_price = 0.0;
        }

        Ok(())
    }
}
