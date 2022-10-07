use udf::prelude::*;
struct IsConst {
    is_const:bool
}

impl BasicUdf for IsConst {
    type Returns<'a> = &'static str;

    fn init<'a>(cfg: &mut InitCfg, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len()!=1{
            return Err("IS_CONST only accepts one argument".to_owned());
        }

        Ok(Self{
            is_const: args.get(0).unwrap().is_const()
        })
    }

    fn process<'a>(
        &'a mut self,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let ret = if self.is_const {
            "const"
        } else {
            "not const"
        };
        Ok(ret)
    }
}
