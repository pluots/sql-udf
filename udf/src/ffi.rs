//! Module containing Rust bindings and wrapper for MySQL/MariaDB C interface

use crate::SqlResult;

pub mod bindings;

#[doc(hidden)]
pub mod wrapper;
pub(crate) mod wrapper_impl;

/// Type of the `Item_result` enum indicator in the FFI
pub type SqlTypeTag = bindings::Item_result;

/// Enum representing possible SQL result types
///
/// This simply represents the possible types, but does not contain any values.
/// [`SqlResult`] is the corresponding enum that actually contains data.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
#[repr(i8)]
pub enum SqlType {
    /// Integer result
    Int = bindings::Item_result_INT_RESULT as i8,
    /// Real result
    Real = bindings::Item_result_REAL_RESULT as i8,
    /// String result
    String = bindings::Item_result_STRING_RESULT as i8,
    /// Decimal result
    Decimal = bindings::Item_result_DECIMAL_RESULT as i8,
}

impl TryFrom<i8> for SqlType {
    type Error = String;

    /// Create an [`ItemResult`] from an integer
    fn try_from(tag: i8) -> Result<Self, Self::Error> {
        let val = match tag {
            x if x == SqlType::String as i8 => SqlType::String,
            x if x == SqlType::Real as i8 => SqlType::Real,
            x if x == SqlType::Int as i8 => SqlType::Int,
            x if x == SqlType::Decimal as i8 => SqlType::Decimal,
            _ => return Err("invalid arg type {tag} received".to_owned()),
        };

        Ok(val)
    }
}

impl TryFrom<SqlTypeTag> for SqlType {
    type Error = String;

    /// Create an [`SqlType`] from an SqlTypeTag (a `c_int`)
    fn try_from(tag: SqlTypeTag) -> Result<Self, Self::Error> {
        let val = match tag {
            x if x == SqlType::String as SqlTypeTag => SqlType::String,
            x if x == SqlType::Real as SqlTypeTag => SqlType::Real,
            x if x == SqlType::Int as SqlTypeTag => SqlType::Int,
            x if x == SqlType::Decimal as SqlTypeTag => SqlType::Decimal,
            _ => return Err("invalid arg type {tag} received".to_owned()),
        };

        Ok(val)
    }
}

impl TryFrom<&SqlResult<'_>> for SqlType {
    type Error = String;

    /// Create an [`SqlType`] from an [`SqlResult`]
    fn try_from(tag: &SqlResult) -> Result<Self, Self::Error> {
        let val = match tag {
            SqlResult::String(_) => SqlType::String,
            SqlResult::Real(_) => SqlType::Real,
            SqlResult::Int(_) => SqlType::Int,
            SqlResult::Decimal(_) => SqlType::Decimal,
        };

        Ok(val)
    }
}
