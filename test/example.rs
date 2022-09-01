use std::os::raw::c_char;
use core::ffi::Cstr;

struct MyUDF {
    v: Vec<u8>,
}

impl MyUdf {
    #[udf(maybe_null)]
    fn init() -> Result<Self,String>{

    }
}
