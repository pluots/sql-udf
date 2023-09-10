#[test]
#[cfg(not(miri))]
fn tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*.rs");
}
