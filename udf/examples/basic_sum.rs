use udf::prelude::*;

#[derive(Debug, PartialEq, Eq, Default)]
struct SumInt {
    baseline: u64,
}

impl BasicUdf for SumInt {
    type Returns<'a> = i64;

    /// Here we evaluate all const arguments possible
    fn init<'a>(args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        let baseline = 0u64;
        for mut arg in args {
            arg.set_type_coercion(udf::SqlType::Decimal);
        }
        Ok(Self { baseline })
    }

    fn process<'a>(
        &'a mut self,
        _args: &ArgList<Process>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
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
