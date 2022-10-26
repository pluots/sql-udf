use udf::prelude::*;

#[derive(Debug)]
struct UdfMedian {
    v: Vec<i64>,
}

#[register]
impl BasicUdf for UdfMedian {
    type Returns<'a> = Option<i64>;

    fn init<'a>(_cfg: &UdfCfg<Init>, _args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        Ok(Self { v: Vec::new() })
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        if self.v.is_empty() {
            Ok(None)
        } else {
            // Safely get the middle element; dereference if it is `Some`
            Ok(self.v.get(self.v.len() / 2).copied())
        }
    }
}

#[register]
impl AggregateUdf for UdfMedian {
    fn clear(
        &mut self,
        _cfg: &UdfCfg<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        self.v.clear();
        Ok(())
    }

    fn add(
        &mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        if let Some(a) = args.get(0) {
            if let Some(v) = a.value.as_int() {
                self.v.push(v);
            } else if let Some(v) = a.value.as_real() {
                self.v.push(v as i64);
            }
        }
        Ok(())
    }
}
