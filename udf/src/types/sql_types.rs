//! Module containing bindings & wrappers to SQL types

use std::{slice, str};

use udf_sys::Item_result;

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
    /// work with [`udf_sys`] bindings directly.
    #[inline]
    pub fn to_item_result(&self) -> Item_result {
        match *self {
            Self::Int => Item_result::INT_RESULT,
            Self::Real => Item_result::REAL_RESULT,
            Self::String => Item_result::STRING_RESULT,
            Self::Decimal => Item_result::DECIMAL_RESULT,
        }
    }

    /// Small helper function to get a displayable type name.
    #[inline]
    pub fn display_name(&self) -> &'static str {
        match *self {
            Self::String => "string",
            Self::Real => "real",
            Self::Int => "int",
            Self::Decimal => "decimal",
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

/// A possible SQL result consisting of a type and nullable value
///
/// This enum is similar to [`SqlType`], but actually contains the object.
///
/// It is of note that both [`SqlResult::String`] contains a `u8` slice rather
/// than a representation like `&str`. This is because there is no guarantee
/// that the data is `utf8`. Use [`SqlResult::as_string()`] if you need an easy
/// way to get a `&str`.
///
/// This enum is labeled `non_exhaustive` to leave room for future types and
/// coercion options.
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum SqlResult<'a> {
    // INVALID_RESULT and ROW_RESULT are other options, but not valid for UDFs
    /// A string result
    String(Option<&'a [u8]>),
    /// A floating point result
    Real(Option<f64>),
    /// A nullable integer
    Int(Option<i64>),
    /// This is a string that is to be represented as a decimal
    Decimal(Option<&'a str>),
}

impl<'a> SqlResult<'a> {
    /// Construct a `SqlResult` from a pointer and a tag
    ///
    /// SAFETY: pointer must not be null. If a string or decimal result, must be
    /// exactly `len` long.
    pub(crate) unsafe fn from_ptr(
        ptr: *const u8,
        tag: Item_result,
        len: usize,
    ) -> Result<SqlResult<'a>, String> {
        // Handle nullptr right away here

        let marker =
            SqlType::try_from(tag).map_err(|_| format!("invalid arg type {tag:?} received"))?;

        let arg = if ptr.is_null() {
            match marker {
                SqlType::Int => SqlResult::Int(None),
                SqlType::Real => SqlResult::Real(None),
                SqlType::String => SqlResult::String(None),
                SqlType::Decimal => SqlResult::Decimal(None),
            }
        } else {
            // SAFETY: `tag` guarantees type. If decimal or String, caller
            // guarantees length
            unsafe {
                #[allow(clippy::cast_ptr_alignment)]
                match marker {
                    SqlType::Int => SqlResult::Int(Some(*(ptr.cast::<i64>()))),
                    SqlType::Real => SqlResult::Real(Some(*(ptr.cast::<f64>()))),
                    SqlType::String => SqlResult::String(Some(slice::from_raw_parts(ptr, len))),
                    // SAFETY: decimals should always be UTF8
                    SqlType::Decimal => SqlResult::Decimal(Some(str::from_utf8_unchecked(
                        slice::from_raw_parts(ptr, len),
                    ))),
                }
            }
        };

        Ok(arg)
    }

    /// Small helper function to get a displayable type name.
    #[inline]
    pub fn display_name(&self) -> &'static str {
        SqlType::try_from(self).map_or("unknown", |v| v.display_name())
    }

    /// Check if this argument is an integer type, even if it may be null
    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(*self, Self::Int(_))
    }
    /// Check if this argument is an real type, even if it may be null
    #[inline]
    pub fn is_real(&self) -> bool {
        matches!(*self, Self::Real(_))
    }
    /// Check if this argument is an string type, even if it may be null
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(*self, Self::String(_))
    }
    /// Check if this argument is an decimal type, even if it may be null
    #[inline]
    pub fn is_decimal(&self) -> bool {
        matches!(*self, Self::Decimal(_))
    }

    /// Return this type as an integer if possible
    ///
    /// This will exist if the variant is [`SqlResult::Int`], and it contains a
    /// value.
    ///
    /// These `as_*` methods are helpful to quickly obtain a value when you
    /// expect it to be of a specific type and present.
    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match *self {
            Self::Int(v) => v,
            _ => None,
        }
    }

    /// Return this type as a float if possible
    ///
    /// This will exist if the variant is [`SqlResult::Real`], and it contains a
    /// value. See [`SqlResult::as_int()`] for further details on `as_*` methods
    #[inline]
    pub fn as_real(&'a self) -> Option<f64> {
        match *self {
            Self::Real(v) => v,
            _ => None,
        }
    }

    /// Return this type as a string if possible
    ///
    /// This will exist if the variant is [`SqlResult::String`], or
    /// [`SqlResult::Decimal`], and it contains a value, _and_ the string can
    /// successfully be converted to `utf8` (using [`str::from_utf8`]). It does
    /// not distinguish among errors (wrong type, `None` value, or invalid utf8)
    /// - use pattern matching if you need that.
    ///
    /// See [`SqlResult::as_int()`] for further details on `as_*` methods
    #[inline]
    pub fn as_string(&'a self) -> Option<&'a str> {
        match *self {
            Self::String(Some(v)) => Some(str::from_utf8(v).ok()?),
            Self::Decimal(Some(v)) => Some(v),
            _ => None,
        }
    }

    /// Return this type as a byte slice if possible
    ///
    /// This will exist if the variant is [`SqlResult::String`], or
    /// [`SqlResult::Decimal`]. See [`SqlResult::as_int()`] for further details
    /// on `as_*` methods
    #[inline]
    pub fn as_bytes(&'a self) -> Option<&'a [u8]> {
        match *self {
            Self::String(Some(v)) => Some(v),
            Self::Decimal(Some(v)) => Some(v.as_bytes()),
            _ => None,
        }
    }
}
