// use udf::BasicUdf;
// use udf_macros::register;

// // Registration is not allowed on non-impls
// struct X {}
// struct Y {}

// #[register]
// impl BasicUdf for crate::X {
//     type Returns<'a> = &'a str;

//     fn init<'a>(
//         _cfg: &mut udf::InitCfg,
//         _args: &'a udf::ArgList<'a, udf::Init>,
//     ) -> Result<Self, String> {
//         todo!()
//     }

//     fn process<'a>(
//         &'a mut self,
//         _args: &udf::ArgList<udf::Process>,
//         _error: Option<std::num::NonZeroU8>,
//     ) -> Result<Self::Returns<'a>, udf::ProcessError> {
//         todo!()
//     }
// }

// #[register]
// impl BasicUdf for crate::Y {
//     type Returns<'a> = Option<i64>;

//     fn init<'a>(
//         _cfg: &mut udf::InitCfg,
//         _args: &'a udf::ArgList<'a, udf::Init>,
//     ) -> Result<Self, String> {
//         todo!()
//     }

//     fn process<'a>(
//         &'a mut self,
//         _args: &udf::ArgList<udf::Process>,
//         _error: Option<std::num::NonZeroU8>,
//     ) -> Result<Self::Returns<'a>, udf::ProcessError> {
//         todo!()
//     }
// }

// fn main() {}
