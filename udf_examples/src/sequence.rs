//! sequence() function
//!
//! Select
//!
//! Start at a given number if an argument is given

use std::fmt::Display;

use udf::prelude::*;

struct SqlSequence {
    last_val: i64,
}

/// Non exhaustive
#[non_exhaustive]
enum Errors {
    BadArguments(usize),
}

impl Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::BadArguments(n) => write!(f, "This function takes 0 or 1 arguments; got {n}"),
        }
    }
}

#[register]
impl BasicUdf for SqlSequence {
    type Returns<'a> = i64
    where
        Self: 'a;

    /// Init just validates the argument count and initializes our empty struct
    fn init<'a>(cfg: &mut UdfCfg, args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        if args.len() > 1 {
            return Err(Errors::BadArguments(args.len()).to_string());
        }

        // If we have an argument, set its type coercion to an integer
        if let Some(mut a) = args.get(0) {
            a.set_type_coercion(SqlType::Int);
        }

        // Result will differ for each call
        cfg.set_is_const(false);
        Ok(Self { last_val: 0 })
    }

    fn process<'a>(
        &'a mut self,
        args: &ArgList<Process>,
        _error: Option<NonZeroU8>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        // If we have an argument, that will provide our base value
        let arg_val = match args.get(0) {
            Some(v) => v.value.as_int().unwrap(),
            None => 0,
        };

        // Increment our last value, return the total
        self.last_val += 1;
        Ok(self.last_val + arg_val)
    }
}

// setup:
// create database db0;
//
// use db0;
//
// create table t1 (
//     id int auto_increment,
//     primary key (id)
// );
//
// insert into t1 () values ();
// insert into t1 () values ();
// insert into t1 () values ();
// insert into t1 () values ();
// insert into t1 () values ();
// insert into t1 () values ();

// test
// MariaDB [db0]> select sql_sequence();
// +----------------+
// | sql_sequence() |
// +----------------+
// |              1 |
// +----------------+

// MariaDB [db0]> select sql_sequence(1);
// +-----------------+
// | sql_sequence(1) |
// +-----------------+
// |               2 |
// +-----------------+
// 1 row in set (0.000 sec)

// MariaDB [db0]> select id, sql_sequence() from t1;
// +----+----------------+
// | id | sql_sequence() |
// +----+----------------+
// |  1 |              1 |
// |  2 |              2 |
// |  3 |              3 |
// |  4 |              4 |
// |  5 |              5 |
// |  6 |              6 |
// +----+----------------+

// MariaDB [db0]> select id, sql_sequence(10) from t1;
// +----+------------------+
// | id | sql_sequence(10) |
// +----+------------------+
// |  1 |               11 |
// |  2 |               12 |
// |  3 |               13 |
// |  4 |               14 |
// |  5 |               15 |
// |  6 |               16 |
// +----+------------------+

// MariaDB [db0]> select id, sql_sequence(-1) from t1;
// +----+------------------+
// | id | sql_sequence(-1) |
// +----+------------------+
// |  1 |                0 |
// |  2 |                1 |
// |  3 |                2 |
// |  4 |                3 |
// |  5 |                4 |
// |  6 |                5 |
// +----+------------------+
