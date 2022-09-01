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

/// # Panics
/// 
/// The returned error string MUST be less than 

register_udf! {
    returns: integer,

    clear: my_clear,
    reset: my_reset,
    add: my_add,
    remove: my_remove

}
