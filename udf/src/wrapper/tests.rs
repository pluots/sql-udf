use super::*;
use crate::prelude::*;

struct ExampleInt;
struct ExampleIntOpt;
struct ExampleBufRef;
struct ExampleBufOpt;
struct ExampleBufOptRef;

impl BasicUdf for ExampleInt {
    type Returns<'a> = i64;

    fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
        todo!()
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<crate::Process>,
        _args: &ArgList<crate::Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        todo!()
    }
}
impl BasicUdf for ExampleIntOpt {
    type Returns<'a> = Option<i64>;

    fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
        todo!()
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<crate::Process>,
        _args: &ArgList<crate::Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        todo!()
    }
}

impl BasicUdf for ExampleBufRef {
    type Returns<'a> = &'a str;

    fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
        todo!()
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<crate::Process>,
        _args: &ArgList<crate::Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        todo!()
    }
}
impl BasicUdf for ExampleBufOpt {
    type Returns<'a> = Option<Vec<u8>>;

    fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
        Ok(Self)
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<crate::Process>,
        _args: &ArgList<crate::Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        Ok(Some(vec![1, 2, 3, 4]))
    }
}

impl AggregateUdf for ExampleBufOpt {
    fn clear(
        &mut self,
        _cfg: &UdfCfg<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        todo!()
    }

    fn add(
        &mut self,
        _cfg: &UdfCfg<Process>,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<(), NonZeroU8> {
        todo!()
    }
}

impl BasicUdf for ExampleBufOptRef {
    type Returns<'a> = Option<&'a str>;

    fn init(_cfg: &UdfCfg<crate::Init>, _args: &ArgList<crate::Init>) -> Result<Self, String> {
        todo!()
    }

    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<crate::Process>,
        _args: &ArgList<crate::Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        todo!()
    }
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
#[allow(clippy::diverging_sub_expression)]
fn test_fn_sig() {
    // Just validate our function signatures with compile tests

    unsafe {
        wrap_process_basic::<ExampleInt, _, _>(todo!(), todo!(), todo!(), todo!());
        wrap_process_basic_option::<ExampleIntOpt, _, _>(todo!(), todo!(), todo!(), todo!());
        wrap_process_buf::<ExampleBufRef, _>(todo!(), todo!(), todo!(), todo!(), todo!(), todo!());
        wrap_process_buf_option::<ExampleBufOpt, _, _>(
            todo!(),
            todo!(),
            todo!(),
            todo!(),
            todo!(),
            todo!(),
        );
        wrap_process_buf_option_ref::<ExampleBufOptRef, _, _>(
            todo!(),
            todo!(),
            todo!(),
            todo!(),
            todo!(),
            todo!(),
        );
    }
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
#[allow(clippy::diverging_sub_expression)]
fn test_wrapper_basic() {
    type ExampleIntWrapper = ExampleInt;
    unsafe {
        wrap_init::<ExampleIntWrapper, ExampleInt>(todo!(), todo!(), todo!());
    }
}

#[test]
#[should_panic]
#[allow(unreachable_code)]
#[allow(clippy::diverging_sub_expression)]
fn test_wrapper_bufwrapper() {
    unsafe {
        wrap_init::<ExampleBufOpt, _>(todo!(), todo!(), todo!());
    }
}
