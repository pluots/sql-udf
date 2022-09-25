use udf::prelude::*;

#[udf::register]
#[derive(Debug, PartialEq, Eq, Default)]
struct SumInt {
    baseline: u64,
}

impl BasicUdf for SumInt {
    type Returns<'a> = i64;

    /// Here we evaluate all const arguments possible
    fn init(args: &[SqlArg<Init>]) -> Result<Self, String> {
        let baseline = 0u64;
        for arg in args {}
        Ok(Self { baseline })
    }
    fn process<'a>(&'a mut self, args: &[SqlArg<Process>]) -> Result<Self::Returns<'a>, String> {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     // use super::*;

//     // #[test]
//     // fn it_works() {
//     //     let result = add(2, 2);
//     //     assert_eq!(result, 4);
//     // }
// }
