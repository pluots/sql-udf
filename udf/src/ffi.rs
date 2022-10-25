//! Module containing Rust bindings and wrapper for MySQL/MariaDB C interface

use crate::SqlResult;

pub mod bindings;

#[doc(hidden)]
pub mod wrapper;
pub(crate) mod wrapper_impl;
use bindings::Item_result;

/// Enum representing possible SQL result types
///
/// This simply represents the possible types, but does not contain any values.
/// [`SqlResult`] is the corresponding enum that actually contains data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
#[repr(i8)]
pub enum SqlType {
    /// Integer result
    Int = Item_result::INT_RESULT as i8,
    /// Real result
    Real = Item_result::REAL_RESULT as i8,
    /// String result
    String = Item_result::STRING_RESULT as i8,
    /// Decimal result
    Decimal = Item_result::DECIMAL_RESULT as i8,
}

impl SqlType {
    /// Convert this enum to a SQL [`Item_result`]. This is only useful if you
    /// use [`crate::ffi::bindings`].
    #[inline]
    pub fn to_item_result(&self) -> Item_result {
        match *self {
            Self::Int => Item_result::INT_RESULT,
            Self::Real => Item_result::REAL_RESULT,
            Self::String => Item_result::STRING_RESULT,
            Self::Decimal => Item_result::DECIMAL_RESULT,
        }
    }
}

impl TryFrom<i8> for SqlType {
    type Error = String;

    /// Create an [`SqlType`] from an integer
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

impl TryFrom<Item_result> for SqlType {
    type Error = String;

    /// Create an [`SqlType`] from an [`Item_result`], located in the `bindings`
    /// module.
    #[inline]
    fn try_from(tag: Item_result) -> Result<Self, Self::Error> {
        let val = match tag {
            Item_result::STRING_RESULT => Self::String,
            Item_result::REAL_RESULT => Self::Real,
            Item_result::INT_RESULT => Self::Int,
            Item_result::DECIMAL_RESULT => Self::Decimal,
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
