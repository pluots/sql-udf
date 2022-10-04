use udf::prelude::*;

#[derive(Debug, PartialEq, Eq, Default)]
struct SumInt {}

impl BasicUdf for SumInt {
    type Returns<'a> = Option<i64>;

    /// Here we evaluate all const arguments possible
    fn init<'a>(args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        for mut arg in args {
            arg.set_type_coercion(udf::SqlType::Int);
        }
        Ok(Self {  })
    }

    fn process<'a>(
        &'a mut self,
        args: &ArgList<Process>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let mut res = 0;

        for arg in args {
            match arg.value.as_int() {
                Some(v) => res += v,
                None => return Err(ProcessError),
            }
        }

        Ok(Some(res))
    }
}


#[no_mangle]
pub unsafe extern "C" fn sum_int_init(
    initid: *mut udf::ffi::bindings::UDF_INIT,
    args: *mut udf::ffi::bindings::UDF_ARGS,
    message: *mut std::os::raw::c_char,
) -> bool {
    unsafe { udf::ffi::wrapper::wrap_init::<SumInt>(initid, args, message) }
}

#[no_mangle]
pub unsafe extern "C" fn sum_int_deinit(initid: *mut udf::ffi::bindings::UDF_INIT) {
    unsafe { udf::ffi::wrapper::wrap_deinit::<SumInt>(initid) }
}

#[no_mangle]
pub unsafe extern "C" fn sum_int(
    initid: *mut udf::ffi::bindings::UDF_INIT,
    args: *const udf::ffi::bindings::UDF_ARGS,
    is_null: *mut std::ffi::c_char,
    error: *mut std::ffi::c_char,
) -> std::ffi::c_longlong {
    unsafe { udf::ffi::wrapper::wrap_process_int_null::<SumInt>(initid, args, is_null, error) }
}


// #[cfg(test)]
// mod tests {
//     // use super::*;

//     // #[test]
//     // fn it_works() {
//     //     let result = add(2, 2);
//     //     assert_eq!(result, 4);
//     // }
// }
