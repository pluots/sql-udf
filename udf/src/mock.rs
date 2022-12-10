//! Mocks of argument types found in this library, available with the feature
//! `mock`
//!
//! This module can be used to create the kind of arguments passed to the
//! [`BasicUdf`](crate::BasicUdf) and [`AggregateUdf`](crate::AggregateUdf)
//! traits, for unit testing UDFs. To use anything in the `mock` module, ensure
//! that the `mock` feature is enabled in your `Cargo.toml`.
//!
//! Do not use these for anything outside of test cases, as the inner workings
//! are considered unstable.
//!
//! Full (lengthy) example:
//!
//! ```
//! use std::cmp::min;
//!
//! use udf::mock::*;
//! use udf::mock_args;
//! use udf::prelude::*;
//!
//! // ****** Fake UDF Implementation ******
//!
//! struct ExampleUdf;
//!
//! impl BasicUdf for ExampleUdf {
//!     type Returns<'a> = String;
//!
//!     // This init function is just to demonstrate our test
//!     fn init(cfg: &UdfCfg<Init>, args: &ArgList<Init>) -> Result<Self, String> {
//!         assert_eq!(cfg.get_max_len(), 10);
//!
//!         let arg0 = args.get(0).unwrap();
//!         assert_eq!(arg0.attribute(), "first arg");
//!         // eprintln!("arg val: {:?}", arg0);
//!         assert_eq!(arg0.value().as_string().unwrap(), "input value");
//!
//!         let arg1 = args.get(1).unwrap();
//!         assert_eq!(arg1.attribute(), "second arg");
//!         assert_eq!(arg1.value().as_int().unwrap(), 5);
//!
//!         Ok(Self)
//!     }
//!
//!     // Out example splits a string (first argument) at an index (second argument)
//!     fn process<'a>(
//!         &'a mut self,
//!         cfg: &UdfCfg<Process>,
//!         args: &ArgList<Process>,
//!         error: Option<std::num::NonZeroU8>,
//!     ) -> Result<Self::Returns<'a>, ProcessError> {
//!         let arg0 = args.get(0).unwrap().value();
//!         let arg1 = args.get(1).unwrap().value();
//!
//!         let s = arg0.as_string().unwrap();
//!         let n = arg1.as_int().unwrap();
//!
//!         let slice = &s[0..min(s.len(), n as usize)];
//!
//!         Ok(slice.to_owned())
//!     }
//! }
//!
//! // ****** Setup section ******
//!
//! let mut mock_cfg = MockUdfCfg::new();
//! *mock_cfg.max_len() = 10;
//!
//! // Replicate the call `select example_udf('input value', 5)`
//! let mut mock_arglist = mock_args![
//!     ("input value", "first arg", false),
//!     (5, "second arg", false),
//! ];
//!
//! // ****** Test section ******
//!
//! // Run the init function
//! let init_cfg = mock_cfg.as_init();
//! let init_args = mock_arglist.as_init();
//! let mut udf_struct = ExampleUdf::init(init_cfg, init_args).unwrap();
//!
//! // Run the process function
//! let proc_cfg = mock_cfg.as_process();
//! let proc_args = mock_arglist.as_process();
//! let res = udf_struct.process(proc_cfg, proc_args, None).unwrap();
//!
//! // The goal of our fake UDF was to split the input string ("input value") at the specified
//! // index (5). Verify our end result
//! assert_eq!(res, "input")
//! ```

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::new_without_default)]
use std::cell::UnsafeCell;
use std::ffi::{c_char, c_uint, c_ulong};
use std::fmt::Debug;
use std::marker::PhantomPinned;
use std::ptr;

use udf_sys::{Item_result, UDF_ARGS, UDF_INIT};

pub use crate::mock_args;
use crate::traits::{Init, Process};
use crate::types::{ArgList, UdfCfg};
use crate::UdfState;

/// A structure that allows generating a `&UdfCfg` object. See [module
/// documentation](crate::mock) for further information.
///
/// ```
/// use udf::mock::MockUdfCfg;
/// use udf::prelude::*;
///
/// // For demo purposes
/// fn example_init(cfg: &UdfCfg<Init>) {
///     cfg.set_max_len(4);
///     cfg.set_is_const(true);
/// }
///
/// let mut mock_cfg = MockUdfCfg::new();
/// *mock_cfg.max_len() = 0;
/// *mock_cfg.is_const() = false;
/// *mock_cfg.decimals() = 0;
///
/// // You would really call `MyUdf::init(...)` here, this is just for an example
/// example_init(mock_cfg.as_init());
///
/// assert_eq!(*mock_cfg.max_len(), 4);
/// assert_eq!(*mock_cfg.is_const(), true);
/// assert_eq!(*mock_cfg.decimals(), 0);
/// ```
#[derive(Debug)]
pub struct MockUdfCfg {
    inner: UnsafeCell<UDF_INIT>,
    // Workaround for MSVC 32 bit ulong
    maxlen_tmp: u64,
    _marker: PhantomPinned,
}

impl MockUdfCfg {
    /// Create a new structure that can be turned into a `UdfCfg` object
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: UnsafeCell::new(UDF_INIT {
                maybe_null: false,
                decimals: 0,
                max_length: 0,
                ptr: ptr::null_mut(),
                const_item: false,
                extension: ptr::null_mut(),
            }),
            maxlen_tmp: 0,
            _marker: PhantomPinned,
        }
    }

    /// Create a `&UdfCfg<Init>` object to test calling a UDF `init` function
    ///
    /// # Panics
    ///
    /// Panics if on Windows and the set `max_len` is greater than `c_ulong::MAX`
    #[allow(clippy::useless_conversion)]
    pub fn as_init(&mut self) -> &UdfCfg<Init> {
        // flush maxlen workaround for windows
        let tmp: c_ulong = self.maxlen_tmp.try_into().unwrap_or_else(|_| {
            panic!(
                "Max length is limited to c_ulong::MAX, got {}",
                self.maxlen_tmp,
            )
        });
        unsafe { (*self.inner.get()).max_length = tmp };
        unsafe { UdfCfg::from_raw_ptr(self.inner.get()) }
    }

    /// Create a `&UdfCfg<Process>` object to test callingg a UDF `process` function
    ///
    /// # Panics
    ///
    /// Panics if on Windows and the set `max_len` is greater than `c_ulong::MAX`
    #[allow(clippy::useless_conversion)]
    pub fn as_process(&mut self) -> &UdfCfg<Process> {
        // flush maxlen workaround for windows
        let tmp: c_ulong = self.maxlen_tmp.try_into().unwrap_or_else(|_| {
            panic!(
                "Max length is limited to `c_ulong::MAX`, got {}",
                self.maxlen_tmp,
            )
        });
        unsafe { (*self.inner.get()).max_length = tmp };
        unsafe { UdfCfg::from_raw_ptr(self.inner.get()) }
    }

    /// Get or set the `maybe_null` field
    pub fn maybe_null(&mut self) -> &mut bool {
        unsafe { &mut (*self.inner.get()).maybe_null }
    }

    /// Get or set the `decimals` field
    pub fn decimals(&mut self) -> &mut u32 {
        unsafe { &mut (*self.inner.get()).decimals }
    }

    /// Get or set the `max_len` field
    #[allow(clippy::useless_conversion)]
    pub fn max_len(&mut self) -> &mut u64 {
        // Workaround for MSVC 32 bit ulong
        self.maxlen_tmp = unsafe { (*self.inner.get()).max_length }.into();
        &mut self.maxlen_tmp
        // let tmp = unsafe { &mut (*self.0.get()).max_length };
        // tmp.into() // Accounting for c_ulong differences
    }
    /// Get or set the `is_const` field
    pub fn is_const(&mut self) -> &mut bool {
        unsafe { &mut (*self.inner.get()).const_item }
    }
}

/// Private struct to hold information about a single argument
#[derive(Debug)]
struct BuiltArgs {
    args: Vec<*const c_char>,
    lengths: Vec<c_ulong>,
    maybe_null: Vec<c_char>,
    attributes: Vec<*const c_char>,
    attribute_lengths: Vec<c_ulong>,
    arg_types: Vec<Item_result>,
}

impl BuiltArgs {
    /// Create a new empty `ArgData` object
    fn new() -> Self {
        Self {
            args: Vec::new(),
            lengths: Vec::new(),
            maybe_null: Vec::new(),
            attributes: Vec::new(),
            attribute_lengths: Vec::new(),
            arg_types: Vec::new(),
        }
    }
}

/// A single mock argument to be used when constructing a [`MockArgList`]
#[derive(Debug)]
pub struct MockArg {
    value: MockArgData,
    attribute: String,
    maybe_null: bool,
}

/// A representation of data within a mock argument
#[derive(Debug)]
#[non_exhaustive]
pub enum MockArgData {
    String(Option<String>),
    Bytes(Option<Vec<u8>>),
    Real(Option<f64>),
    Int(Option<i64>),
    Decimal(Option<String>),
}

impl MockArgData {
    fn as_item_result(&self) -> Item_result {
        match *self {
            Self::String(_) | Self::Bytes(_) => Item_result::STRING_RESULT,
            Self::Real(_) => Item_result::REAL_RESULT,
            Self::Int(_) => Item_result::INT_RESULT,
            Self::Decimal(_) => Item_result::DECIMAL_RESULT,
        }
    }
}

impl From<&str> for MockArgData {
    fn from(value: &str) -> Self {
        Self::String(Some(value.to_owned()))
    }
}

impl From<Option<&str>> for MockArgData {
    fn from(value: Option<&str>) -> Self {
        Self::String(value.map(std::borrow::ToOwned::to_owned))
    }
}
impl From<&[u8]> for MockArgData {
    fn from(value: &[u8]) -> Self {
        Self::Bytes(Some(value.to_owned()))
    }
}

impl From<Option<&[u8]>> for MockArgData {
    fn from(value: Option<&[u8]>) -> Self {
        Self::Bytes(value.map(std::borrow::ToOwned::to_owned))
    }
}

impl From<i64> for MockArgData {
    fn from(value: i64) -> Self {
        Self::Int(Some(value))
    }
}

impl From<Option<i64>> for MockArgData {
    fn from(value: Option<i64>) -> Self {
        Self::Int(value)
    }
}
impl From<f64> for MockArgData {
    fn from(value: f64) -> Self {
        Self::Real(Some(value))
    }
}

impl From<Option<f64>> for MockArgData {
    fn from(value: Option<f64>) -> Self {
        Self::Real(value)
    }
}

impl MockArg {
    /// Create a new mock argument with given contents, attribute, and
    /// nullability
    #[inline]
    pub fn new(value: MockArgData, attr: &str, maybe_null: bool) -> Self {
        Self {
            value,
            attribute: attr.to_owned(),
            maybe_null,
        }
    }

    /// Get a reference to this argument's value for setting or updating
    #[inline]
    pub fn value(&mut self) -> &mut MockArgData {
        &mut self.value
    }

    /// Get a reference to this argument's attribute for setting or updating
    #[inline]
    pub fn attribute(&mut self) -> &mut String {
        &mut self.attribute
    }
}

/// Structure to create a `&mut ArgList` for testing
#[derive(Debug)]
pub struct MockArgList {
    /// User-updated list of arguments
    ///
    /// Must remain pinned!
    unbuilt_args: Vec<MockArg>,

    /// Temporary place to store arguments while building
    ///
    /// This holds pointers to `unbuilt_args` in a rusty way. Must remain
    /// pinned!
    built_args: Option<BuiltArgs>,

    /// The resulting UDF_ARGS struct that points to `built_args`
    udf_args: Option<UnsafeCell<UDF_ARGS>>,

    /// We use a phantom pin here because our `out_list` item will reference
    /// `arg_data`, so we need to make sure this doesn't move
    _pin: PhantomPinned,
}

impl MockArgList {
    #[inline]
    pub fn new() -> Self {
        Self {
            unbuilt_args: Vec::new(),
            built_args: None,
            udf_args: None,
            _pin: PhantomPinned,
        }
    }

    #[inline]
    pub fn push_arg(&mut self, arg: MockArg) {
        self.unbuilt_args.push(arg);
    }

    /// Build the arguments
    ///
    /// This should always be safe to unwrap
    #[allow(clippy::pattern_type_mismatch)]
    fn build<S: UdfState>(&mut self) -> &ArgList<S> {
        let mut building = BuiltArgs::new();

        for arg in &self.unbuilt_args {
            // Add the attribute to this our build structure directly
            building
                .attributes
                .push(arg.attribute.as_str().as_ptr().cast());
            building
                .attribute_lengths
                .push(arg.attribute.len() as c_ulong);
            building.maybe_null.push(arg.maybe_null as c_char);
            building.arg_types.push(arg.value.as_item_result());

            // Args themselves are more difficult
            match &arg.value {
                MockArgData::String(v) | MockArgData::Decimal(v) => {
                    // Get a pointer to the buffer, or default to none
                    let v_ref = v.as_ref();
                    let buf_ptr = v_ref.map_or(ptr::null(), |s| s.as_ptr().cast());
                    let len = v.as_ref().map_or(0, |s| s.len() as c_ulong);
                    building.args.push(buf_ptr);
                    building.lengths.push(len);
                }
                MockArgData::Bytes(v) => {
                    let v_ref = v.as_ref();
                    let buf_ptr = v_ref.map_or(ptr::null(), |s| s.as_ptr().cast());
                    let len = v.as_ref().map_or(0, |s| s.len() as c_ulong);
                    building.args.push(buf_ptr);
                    building.lengths.push(len);
                }
                MockArgData::Int(v) => {
                    // Just add a pointer to the data
                    let v_ref = v.as_ref();
                    let data_ptr = v_ref.map_or(ptr::null(), |i| {
                        let ptr: *const i64 = i;
                        ptr.cast()
                    });
                    building.args.push(data_ptr);
                    building.lengths.push(0);
                }
                MockArgData::Real(v) => {
                    let v_ref = v.as_ref();
                    let data_ptr = v_ref.map_or(ptr::null(), |i| {
                        let ptr: *const f64 = i;
                        ptr.cast()
                    });
                    building.args.push(data_ptr);
                    building.lengths.push(0);
                }
            }
        }

        // Sanity check
        let arg_count = self.unbuilt_args.len();
        assert_eq!(building.arg_types.len(), arg_count);
        assert_eq!(building.args.len(), arg_count);
        assert_eq!(building.lengths.len(), arg_count);
        assert_eq!(building.maybe_null.len(), arg_count);
        assert_eq!(building.attributes.len(), arg_count);
        assert_eq!(building.attribute_lengths.len(), arg_count);

        self.built_args = Some(building);
        self.set_udf_args();
        let udf_args_ref = self.udf_args.as_mut().unwrap().get();

        unsafe { ArgList::from_raw_ptr(udf_args_ref) }

        // SAFETY: we created this data and so it should be sound
        // unsafe { Some(ArgList::from_udf_args(udf_args)) }
    }

    /// Just update our `udf_args` from our `built_args`
    fn set_udf_args(&mut self) {
        let built = self
            .built_args
            .as_mut()
            .expect("Library error: arguments unbuilt");

        let udf_args = UDF_ARGS {
            arg_count: built.args.len() as c_uint,
            arg_types: built.arg_types.as_mut_ptr(),
            args: built.args.as_ptr(),
            lengths: built.lengths.as_ptr(),
            maybe_null: built.maybe_null.as_ptr(),
            attributes: built.attributes.as_ptr(),
            attribute_lengths: built.attribute_lengths.as_ptr(),
            extension: ptr::null(),
        };

        self.udf_args = Some(UnsafeCell::new(udf_args));
    }

    /// Create a `&ArgList<Init>` for testing with the `init()` function call
    pub fn as_init(&mut self) -> &ArgList<Init> {
        self.build()
    }

    /// Create a `&ArgList<Process>` for testing with the `process()` function call
    pub fn as_process(&mut self) -> &ArgList<Process> {
        self.build()
    }

    // Need a flush method to back populate ArgList to data
}

impl<const N: usize> From<[MockArg; N]> for MockArgList {
    /// Create a [`MockArgList`] from an array of arguments
    fn from(value: [MockArg; N]) -> Self {
        let mut ret = Self::new();
        for arg in value {
            ret.push_arg(arg);
        }
        ret
    }
}

/// Helper macro to create a [`MockArgList`]
///
/// Use this macro to easily create a `MockArgList`
/// For example, to produce the following SQL arguments:
///
/// ```sql
/// -- Assuming `id` is the table column
/// select some_udf(id, "some string", 1000 as value, 1.234, NULL) from some_table;
/// ```
///
/// You could create mock arguments like the following:
///
/// ```
/// use udf::mock_args;
///
/// let mut arglist = mock_args![
///     // assuming id of 1
///     (1, "1", false),
///     // Type can be specified if desired
///     (Int 1, "1", false),
///     ("some string", "some string", false),
///     (1000, "value", false),
///     // By default anything in "" will be a string - you can specify the `MockArgData` type
///     (Decimal "1.234", "1.234", false),
///     // The type is required if `None` is specified
///     (Int None, "NULL", false),
/// ];
///
/// let init_args = arglist.as_init();
/// // Call to your `init` function with `init_args`
/// ```
#[macro_export]
macro_rules! mock_args {
    // Just use `From` to get a type
    (@internal $val:expr, $attr:expr, $nullable:expr) => {
        $crate::mock::MockArg::new($crate::mock::MockArgData::from($val), $attr, $nullable)
    };

    // Catch any `None`s
    (@internal $type_:ident None, $attr:expr, $nullable:expr) => {
        $crate::mock::MockArg::new($crate::mock::MockArgData::$type_(None), $attr, $nullable)
    };

    // String type needs to be owned
    (@internal String $val:expr, $attr:expr, $nullable:expr) => {
        $crate::mock::MockArg::new($crate::mock::MockArgData::String(Some($val.to_owned())), $attr, $nullable)
    };

    // Decimals also need an owned string
    (@internal Decimal $val:expr, $attr:expr, $nullable:expr) => {
        $crate::mock::MockArg::new($crate::mock::MockArgData::Decimal(Some($val.to_owned())), $attr, $nullable)
    };

    // Any generic type
    (@internal $type_:ident $val:expr, $attr:expr, $nullable:expr) => {
        $crate::mock::MockArg::new($crate::mock::MockArgData::$type_(Some($val)), $attr, $nullable)
    };

    // Match repeating (x y z), groups (optional trailling comma)
    ($( ($( $tt:tt )*) ),* $(,)?) => {
        $crate::mock::MockArgList::from([
            $(
                $crate::mock_args!(@internal $($tt)*)
            ),*
        ])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_args_macro() {
        let _args = mock_args![
            // assuming id of 1
            (1, "1", false),
            ("some string", "some string", false),
            (1000, "value", false),
            // By default anything in "" will be a string - you can specify the `MockArgData` type
            (Decimal "1.234", "1.234", false),
            (Int None, "NULL", false),
            (Decimal None, "NULL", false),
            (Decimal None, "NULL", true),
        ];
    }
}
