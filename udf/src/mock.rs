//! Useful helpers for testing

use std::{marker::PhantomData, ops::Deref};

use crate::{
    ffi::SqlTypeTag,
    types::{Init, SqlArg},
    SqlResult, SqlType, UdfState,
};

pub struct MockSqlArg<'a, S: UdfState> {
    sql_arg: SqlArg<'a, S>,
}

impl<'a, S: UdfState> Deref for MockSqlArg<'a, S> {
    type Target = SqlArg<'a, S>;

    fn deref(&self) -> &Self::Target {
        &self.sql_arg
    }
}

impl<'a, S: UdfState> Drop for MockSqlArg<'a, S> {
    /// Just take over the pointer
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.sql_arg.type_ptr) };
    }
}

impl<'a> MockSqlArg<'a, Init> {
    pub fn new(arg: &SqlResult<'a>, maybe_null: bool, attribute: &'a str) -> Self {
        let stype = SqlType::try_from(arg).unwrap();
        let b = Box::new(stype as SqlTypeTag);
        let type_ptr = Box::into_raw(b);

        MockSqlArg {
            sql_arg: SqlArg {
                arg: arg.clone(),
                maybe_null,
                attribute,
                type_ptr,
                marker: PhantomData,
            },
        }
    }
}
