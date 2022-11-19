//! Just print the attribute of whatever is called, usually variable name
//!
//! # Usage
//!
//! ```sql
//! CREATE FUNCTION udf_attribute RETURNS string SONAME 'libudf_examples.so';
//! SELECT sum_int(1, 2, 3, 4, '5', 6.2)
//! ```

use udf::prelude::*;

#[derive(Debug, PartialEq, Eq, Default)]
struct UdfAttribute;

#[register]
impl BasicUdf for UdfAttribute {
    type Returns<'a> = String;

    /// Nothing to do here
    fn init<'a>(_cfg: &UdfCfg<Init>, _args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        Ok(Self)
    }

    /// Just iterate the arguments and add their atttribute to the string
    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let mut v: Vec<String> = Vec::new();

        for arg in args {
            v.push(arg.attribute().to_owned());
        }

        Ok(v.join(", "))
    }
}
