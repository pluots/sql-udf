use udf::{BasicUdf, InitArgInfo, ArgInfo};

#[udf::register]
struct Sum {}

impl BasicUdf for Sum {
    type Returns = i64;
    fn init(args: &[InitArgInfo]) -> Result<Self, String> {
        todo!()
    }
    fn process(&self, args: &[ArgInfo]) -> Result<Self::Returns, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
