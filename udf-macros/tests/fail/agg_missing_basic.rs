#![allow(unused)]

use udf::prelude::*;

struct MyUdf;

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
