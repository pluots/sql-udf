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

/// We just use this to show one way of handling errors a bit cleaner
///
/// This is `pub` because we reuse it for `avg2`
pub enum Errors<'a> {
    WrongArgCount(usize),
    FirstArgType(&'a SqlArg<'a, Init>),
    SecondArgType(&'a SqlArg<'a, Init>),
}

impl Errors<'_> {
    pub fn to_string(&self) -> String {
        match self {
            Self::WrongArgCount(n) => format!("This function takes two arguments; got {n}"),
            Self::FirstArgType(a) => format!(
                "First argument must be an integer; received {} {}",
                a.value.display_name(),
                a.attribute
            ),
            Self::SecondArgType(a) => format!(
                "Second argument must be an integer; received {} {}",
                a.value.display_name(),
                a.attribute
            ),
        }
    }
}

#[register]
impl BasicUdf for AvgCost {
    type Returns<'a> = Option<f64>
    where
        Self: 'a;

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
        Ok(Some(self.total_price / self.total_qty as f64))
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
        let in_qty;
        let mut price;

        // We can unwrap because we are guaranteed to have 2 args (from checks in init)
        if let Some(q) = args.get(0).unwrap().value.as_int() {
            in_qty = q;
        } else {
            return Ok(());
        };

        if let Some(p) = args.get(1).unwrap().value.as_real() {
            price = p;
        } else {
            return Ok(());
        }

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
