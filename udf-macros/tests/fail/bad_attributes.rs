#![allow(unused)]

use udf::prelude::*;

struct MyUdf1;
struct MyUdf2;

#[register(foo = "foo")]
impl BasicUdf for MyUdf1 {
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

#[register(name = "bar", name = "name")]
impl BasicUdf for MyUdf2 {
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

fn main() {}
