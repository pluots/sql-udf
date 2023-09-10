#![allow(unused)]

use udf::prelude::*;

struct MyUdf1;
struct MyUdf2;
struct MyUdf3;

#[register]
impl BasicUdf for MyUdf1 {
    type Returns<'a> = Option<i64>;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
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

#[register(name = "foo")]
impl BasicUdf for MyUdf2 {
    type Returns<'a> = Option<i64>;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
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

#[register(alias = "banana")]
impl BasicUdf for MyUdf3 {
    type Returns<'a> = Option<i64>;

    fn init(_cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
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

fn main() {
    // check that expected symbols exist
    let _ = my_udf1 as *const ();
    let _ = my_udf1_init as *const ();
    let _ = my_udf1_deinit as *const ();
    let _ = foo as *const ();
    let _ = foo_init as *const ();
    let _ = foo_deinit as *const ();
    let _ = my_udf3 as *const ();
    let _ = my_udf3_init as *const ();
    let _ = my_udf3_deinit as *const ();
    let _ = banana as *const ();
    let _ = banana_init as *const ();
    let _ = banana_deinit as *const ();
}
