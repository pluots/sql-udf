//! Implement a simple UUID

use udf::prelude::*;
use uuid::Uuid;

/// We have no data to share among
struct UuidGenerateV4 {}

impl BasicUdf for UuidGenerateV4 {
    type Returns<'a> = String;

    /// The only thing to validate here is that we have no arguments
    fn init(args: &[SqlArg<Init>]) -> Result<Self, String> {
        if !args.is_empty() {
            return Err("No arguments expected".to_owned());
        }

        Ok(Self {})
    }

    /// Just create a v4 UUID and return it
    fn process<'a>(&'a mut self, _args: &[SqlArg<Process>]) -> Result<Self::Returns<'a>, ()> {
        let uuid = Uuid::new_v4();
        Ok(uuid.as_hyphenated().to_string())
    }
}
