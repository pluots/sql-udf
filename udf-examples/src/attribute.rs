//! Just print the attribute of whatever is called, usually variable name
//!
//! # Usage
//!
//! ```sql
//! CREATE FUNCTION udf_attribute RETURNS string SONAME 'libudf_examples.so';
//! SELECT sum_int(1, 2, 3, 4, '5', 6.2)
//! ```

use udf::prelude::*;

#[derive(Debug, PartialEq, Eq, Default)]
struct UdfAttribute;

#[register]
impl BasicUdf for UdfAttribute {
    type Returns<'a> = String;

    /// Nothing to do here
    fn init(_cfg: &UdfCfg<Init>, _args: &ArgList<Init>) -> Result<Self, String> {
        Ok(Self)
    }

    /// Just iterate the arguments and add their atttribute to the string
    fn process<'a>(
        &'a mut self,
        _cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        let mut v: Vec<String> = Vec::new();

        for arg in args {
            v.push(arg.attribute().to_owned());
        }

        Ok(v.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use udf::mock::*;

    use super::*;

    #[test]
    fn test_init() {
        // Not really anything to test here
        let mut mock_cfg = MockUdfCfg::new();
        let mut mock_args = mock_args![];

        assert!(UdfAttribute::init(mock_cfg.as_init(), mock_args.as_init()).is_ok());
    }

    #[test]
    fn process_empty() {
        // No arguments should give us an empty string
        let mut inited = UdfAttribute;
        let mut mock_cfg = MockUdfCfg::new();
        let mut mock_args = mock_args![];

        let res = UdfAttribute::process(
            &mut inited,
            mock_cfg.as_process(),
            mock_args.as_process(),
            None,
        );

        assert_eq!(res.unwrap(), "");
    }

    #[test]
    fn process_nonempty() {
        // Test with some random arguments
        let mut inited = UdfAttribute;
        let mut mock_cfg = MockUdfCfg::new();
        let mut mock_args = mock_args![
            (String None, "attr1", false),
            (Int 42, "attr2", false),
            (Decimal None, "attr3", false),
            (Int None, "attr4", false),
        ];

        let res = UdfAttribute::process(
            &mut inited,
            mock_cfg.as_process(),
            mock_args.as_process(),
            None,
        );

        assert_eq!(res, Ok("attr1, attr2, attr3, attr4".to_owned()));
    }
}
