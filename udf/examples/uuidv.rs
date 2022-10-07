//! Implement a simple UUID

use udf::prelude::*;
use uuid::Uuid;

/// We have no data to share among
struct UuidGenerateV4 {}

impl BasicUdf for UuidGenerateV4 {
    type Returns<'a> = String;

    /// The only thing to validate here is that we have no arguments
    fn init(cfg: &mut InitCfg, args: &ArgList<Init>) -> Result<Self, String> {
        if !args.is_empty() {
            return Err("No arguments expected".to_owned());
        }

        let mut a1 = args.get(0).unwrap();
        a1.set_type_coercion(udf::SqlType::Real);

        Ok(Self {})
    }

    /// Just create a v4 UUID and return it
    fn process<'a>(
        &'a mut self,
        _args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let uuid = Uuid::new_v4();
        Ok(uuid.as_hyphenated().to_string())
    }
}

#[no_mangle]
pub unsafe extern "C" fn uuid_generate_v4_init(
    initid: *mut udf::ffi::bindings::UDF_INIT,
    args: *mut udf::ffi::bindings::UDF_ARGS,
    message: *mut std::os::raw::c_char,
) -> bool {
    unsafe { udf::ffi::wrapper::wrap_init::<UuidGenerateV4>(initid, args, message) }
}

#[no_mangle]
pub unsafe extern "C" fn uuid_generate_v4(initid: *mut udf::ffi::bindings::UDF_INIT) {
    unsafe { udf::ffi::wrapper::wrap_deinit::<UuidGenerateV4>(initid) }
}
