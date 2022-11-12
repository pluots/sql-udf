//! Registration should fail no anything that is not an impl

use std::ops::Add;

use udf_derive::register;

// Registration is not allowed on non-impls
struct X {}

#[register]
impl Add for X {
    type Output = u8;

    fn add(self, _other: Self) -> u8 {
        0
    }
}

fn main() {}
