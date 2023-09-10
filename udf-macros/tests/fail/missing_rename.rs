#![allow(unused)]

use udf::prelude::*;

struct MyUdf;

#[register(name = "foo", alias = "bar")]
impl BasicUdf for MyUdf {
    type Returns<'a> = Option<i64>;

    fn init(cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
        todo!();
    }

    fn process<'a>(
        &'a mut self,
        cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        todo!();
    }
}

#[register]
impl AggregateUdf for MyUdf {
    // Required methods
    fn clear(&mut self, cfg: &UdfCfg<Process>, error: Option<NonZeroU8>) -> Result<(), NonZeroU8> {
        todo!()
    }
    fn add(
        &mut self,
        cfg: &UdfCfg<Process>,
        args: &ArgList<'_, Process>,
        error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        todo!()
    }
}

fn main() {}
