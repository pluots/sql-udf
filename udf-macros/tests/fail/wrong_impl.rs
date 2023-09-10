//! Registration should fail no anything that is not an impl

use udf_macros::register;

// Registration is not allowed on non-impls
struct X;

#[register]
impl Foo for X {}

fn main() {}
