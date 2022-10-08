use udf::prelude::*;
struct Median {
    v: Vec<i64>,
}

impl BasicUdf for Median {
    type Returns<'a> = Option<i64>;

    fn init<'a>(_cfg: &mut InitCfg, _args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        Ok(Self { v: Vec::new() })
    }

    fn process<'a>(
        &'a mut self,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        if self.v.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.v[self.v.len() / 2]))
        }
    }
}

impl AggregateUdf for Median {
    fn clear(&mut self, _error: Option<NonZeroU8>) -> Result<(), NonZeroU8> {
        Ok(self.v.clear())
    }

    fn add(&mut self, args: &ArgList<Process>, _error: Option<NonZeroU8>) -> Result<(), NonZeroU8> {
        if let Some(a) = args.get(0) {
            if let Some(v) = a.value.as_int() {
                self.v.push(v);
            }
        }
        Ok(())
    }
}
