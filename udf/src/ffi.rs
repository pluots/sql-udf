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
#[derive(Debug, PartialEq, Eq)]
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
    #[inline]
    fn try_from(tag: i8) -> Result<Self, Self::Error> {
        let val = match tag {
            x if x == Self::String as i8 => Self::String,
            x if x == Self::Real as i8 => Self::Real,
            x if x == Self::Int as i8 => Self::Int,
            x if x == Self::Decimal as i8 => Self::Decimal,
            _ => return Err("invalid arg type {tag} received".to_owned()),
        };

        Ok(val)
    }
}

impl TryFrom<SqlTypeTag> for SqlType {
    type Error = String;

    /// Create an [`SqlType`] from an [`SqlTypeTag`] (a `c_int`)
    #[inline]
    fn try_from(tag: SqlTypeTag) -> Result<Self, Self::Error> {
        let val = match tag {
            x if x == Self::String as SqlTypeTag => Self::String,
            x if x == Self::Real as SqlTypeTag => Self::Real,
            x if x == Self::Int as SqlTypeTag => Self::Int,
            x if x == Self::Decimal as SqlTypeTag => Self::Decimal,
            _ => return Err("invalid arg type {tag} received".to_owned()),
        };

        Ok(val)
    }
}

impl TryFrom<&SqlResult<'_>> for SqlType {
    type Error = String;

    /// Create an [`SqlType`] from an [`SqlResult`]
    #[inline]
    fn try_from(tag: &SqlResult) -> Result<Self, Self::Error> {
        let val = match *tag {
            SqlResult::String(_) => Self::String,
            SqlResult::Real(_) => Self::Real,
            SqlResult::Int(_) => Self::Int,
            SqlResult::Decimal(_) => Self::Decimal,
        };

        Ok(val)
    }
}
