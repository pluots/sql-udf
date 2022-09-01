use udf::{BasicUdf};

#[udf::register]
struct Sum {}

impl BasicUdf for Sum {
    type Returns = i64;
    fn init<Self>(_: &[InitArgInfo<'_>]) -> Result<Self, String> { todo!() }
    fn process(&self, _: &[ArgInfo<'_>]) -> Result<Self::Returns, String> { todo!() }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
