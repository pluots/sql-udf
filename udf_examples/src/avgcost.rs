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

    fn init<'a>(cfg: &UdfCfg<Init>, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() != 2 {
            return Err("AVGCOST() requires two arguments".to_owned());
        }

        let a0 = args.get(0).unwrap();
        let a1 = args.get(1).unwrap();

        if !a0.value.is_int() {
            return Err(format!(
                "First argument must be an integer; received {} {}",
                a0.value.display_name(),
                a0.attribute
            ));
        }
        if !a1.value.is_real() {
            return Err(format!(
                "Second argument must be a real; received {} {}",
                a1.value.display_name(),
                a1.attribute
            ));
        }

        // args.get(1).unwrap().set_type_coercion(SqlType::Real);

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
    fn clear(&mut self, _cfg: &UdfCfg<Process>, error: Option<NonZeroU8>) -> Result<(), NonZeroU8> {
        // If there is an error, re-return the error
        error.map_or(Ok(()), Err)?;

        // Reset our struct and return
        *self = Self::default();
        Ok(())
    }

    fn add(
        &mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        error.map_or(Ok(()), Err)?;
        let qty;
        let mut price;

        if let Some(q) = args.get(0).unwrap().value.as_int() {
            qty = q;
        } else {
            return Ok(());
        };

        if let Some(p) = args.get(1).unwrap().value.as_real() {
            price = p;
        } else {
            return Ok(());
        }

        let newqty = self.total_qty + qty;
        self.count += 1;

        if (self.total_qty >= 0 && qty < 0) || (self.total_qty < 0 && qty > 0) {
            if !((qty < 0 && newqty < 0) || (qty > 0 && newqty > 0)) {
                price = self.total_price / self.total_qty as f64;
            }

            self.total_price = price * newqty as f64;
        } else {
            self.total_qty += qty;
            self.total_price += price * qty as f64;
        }

        if self.total_qty == 0 {
            self.total_price = 0.0;
        }

        Ok(())
    }
}
