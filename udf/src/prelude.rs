//! Module that can be imported with `use udf::prelude::*;` to quickly get the
//! most often used imports.

pub use std::num::NonZeroU8;

pub use crate::{
    register, AggregateUdf, ArgList, BasicUdf, Init, Process, ProcessError, SqlArg, SqlType, UdfCfg,
};
