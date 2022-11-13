//! Registration should fail no anything that is not an impl

use udf_derive::register;

// Registration is not allowed on non-impls
#[register]
struct X {}

fn main() {}
