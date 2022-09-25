#![allow(unused)]

use lipsum::{lipsum, lipsum_from_seed};
use udf::prelude::*;
use udf_derive::registerx;

// Cap potential resource usage, this gives us more than enough to
// populate LONGTEXT
const MAX_WORDS: i64 = (u32::MAX >> 4) as i64;

/// We expect to return a long string here so we need to contain it in
// #[register]
struct Lipsum {
    res: String,
}

//
// #[registerx]
impl BasicUdf for Lipsum {
    type Returns<'a> = &'a str;

    /// We expect LIPSUM(n) or LIPSUM(n, m)
    fn init(args: &[SqlArg<Init>]) -> Result<Self, String> {
        if args.is_empty() || args.len() > 2 {
            return Err(format!("Expected 1 or 2 args; got {}", args.len()));
        }

        let n = args[0]
            .arg
            .as_int()
            .ok_or("First argument must be an integer".to_owned())?;

        // Perform error checks
        if n > MAX_WORDS {
            return Err(format!("Maximum of {MAX_WORDS} words, got {n}"));
        }
        if n < 0 {
            return Err(format!("Word count must be greater than 0, got {n}"));
        }

        // If there is an extra arg, verify it is also an integer
        match args.get(1) {
            Some(v) => {
                let seed = v
                    .arg
                    .as_int()
                    .ok_or("Second argument must be an integer".to_owned())?;
                if seed < 0 {
                    return Err(format!("Seed must be a positive integer, got {seed}"));
                }
            }
            None => (),
        };

        Ok(Self {
            res: String::default(),
        })
    }

    fn process<'a>(&'a mut self, args: &[SqlArg<Process>]) -> Result<Self::Returns<'a>, ()> {
        // We have already checked that these values fit into usize in init
        // Do need to ensure our argument isn't null
        let n = args[0].arg.as_int().ok_or(())? as usize;

        let res = match args.get(1) {
            Some(v) => {
                // If we have a seed argument, use it.
                let seed = v.arg.as_int().ok_or(())?;
                lipsum_from_seed(n, seed as u64)
            }
            None => {
                // If no seed argument, just generate word count
                lipsum(n)
            }
        };

        self.res = res;

        Ok(&self.res)
    }
}

struct LipsumA {
    res: String,
}

// #[registerx]
impl BasicUdf for LipsumA {
    type Returns<'a> = u64;

    /// We expect LIPSUM(n) or LIPSUM(n, m)
    fn init(args: &[SqlArg<Init>]) -> Result<Self, String> {
        if args.is_empty() || args.len() > 2 {
            return Err(format!("Expected 1 or 2 args; got {}", args.len()));
        }

        let n = args[0]
            .arg
            .as_int()
            .ok_or("First argument must be an integer".to_owned())?;

        // Perform error checks
        if n > MAX_WORDS {
            return Err(format!("Maximum of {MAX_WORDS} words, got {n}"));
        }
        if n < 0 {
            return Err(format!("Word count must be greater than 0, got {n}"));
        }

        // If there is an extra arg, verify it is also an integer
        match args.get(1) {
            Some(v) => {
                let seed = v
                    .arg
                    .as_int()
                    .ok_or("Second argument must be an integer".to_owned())?;
                if seed < 0 {
                    return Err(format!("Seed must be a positive integer, got {seed}"));
                }
            }
            None => (),
        };

        Ok(Self {
            res: String::default(),
        })
    }

    fn process<'a>(&'a mut self, args: &[SqlArg<Process>]) -> Result<Self::Returns<'a>, ()> {
        Ok(0)
    }
}

struct LipsumB {}

#[registerx]
impl BasicUdf for LipsumB {
    type Returns<'a> = Option<u8>;

    /// We expect LIPSUM(n) or LIPSUM(n, m)
    fn init(args: &[SqlArg<Init>]) -> Result<Self, String> {
        Ok(Self {})
    }

    fn process<'a>(&'a mut self, args: &[SqlArg<Process>]) -> Result<Self::Returns<'a>, ()> {
        Ok(Some(0))
    }
}
