use udf::prelude::*;

struct AvgCost {
    count: usize,
    total_qty: i64,
    total_price: f64,
}

impl BasicUdf for AvgCost {
    type Returns<'a> = Option<f64>
    where
        Self: 'a;

    fn init<'a>(cfg: &mut InitCfg, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() != 2 {
            return Err("AVGCOST() requires two arguments".to_owned());
        }
        args.get(0)
            .unwrap()
            .value
            .as_int()
            .ok_or("First argument must be an integer")?;

        args.get(2)
            .unwrap()
            .value
            .as_real()
            .ok_or("Second argument must be a real")?;

        Ok(Self {
            count: 0,
            total_qty: 0,
            total_price: 0.0,
        })
    }

    fn process<'a>(
        &'a mut self,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        if self.count == 0 || self.total_qty == 0 {
            return Ok(None);
        }
        Ok(Some(self.total_price / self.total_qty as f64))
    }
}

impl AggregateUdf for AvgCost {
    fn clear(&mut self, error: Option<NonZeroU8>) -> Result<(), NonZeroU8> {
        error.map_or(Ok(()), |e| Err(e))?;

        self.total_price = 0.0;
        self.total_qty = 0;
        self.count = 0;
        Ok(())
    }

    fn add(&mut self, args: &ArgList<Process>, error: Option<NonZeroU8>) -> Result<(), NonZeroU8> {
        error.map_or(Ok(()), |e| Err(e))?;

        let qty = args.get(0).unwrap().value.as_int().unwrap();
        let newqty = self.total_qty+qty;
        self.count+=1;

        // More work comes here

        if self.total_qty==0 {
            self.total_price=0.0;
        }

        Ok(())
    }
}
