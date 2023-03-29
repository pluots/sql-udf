# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

### Changed

### Removed



## [0.5.3] - 2023-03-29

### Changed

- Fixed compilation for `Option<&'a [u8]>` types. Added example `mishmash` to
  complement this


## [0.5.2] - 2023-03-23

### Changed

- Bump dependencies to the latest version
- Update docs on feature flags
- [Internal] remove calls to `catch_unwind`
- [Internal] refactor feature flag calls
- [CI] refactor CI


## [0.5.1] - 2023-01-04

Changed licensing from 'Apache-2.0' to 'Apache-2.0 OR GPL-2.0-or-later'


## [0.5.0] - 2022-12-20

### Added

- Added feature `logging-debug-calls` for full debug printing of call parameters

### Changed

- Reworked behind the scenes for returning owned results (e.g. `String`) of any
  length, which nicely simplifies the API.
- Now using an internal type wrapper to solve `Miri`'s bad transmute suggestion


## [0.4.5] - 2022-12-10

### Added

- Added option to print full backtraces for buffer issues with `RUST_LIB_BACKTRACE=1`



## [0.4.4] - 2022-12-10

### Fixed

- Corrected issue with the size of `unsigned long` that would cause MSVC to not
  compile (`c_ulong` is 32 bits on MSVC, 64 bits everywhere else)


## [0.4.2] - 2022-12-08

### Added

- Feature `logging-debug` for potentially helpful output messages

### Changed

- Removed `MARIADB_ROOT_PASSWORD` from dockerfile
- Updated return buffer overflow action to return `NULL` instead copying some
  data.
- Added type names to printed output


## [0.4.1] - 2022-12-08

### Fixed

- Corrected dependency version for `udf-macros`


## [0.4.0] - 2022-12-08

This version is now yanked

### Added

- Mocks: added the `mock` module that provides ways to unit test UDFs. This is
  still a work in progress, and requires the feature `mock`.

### Changed

- Improved memory footprint of `SqlArg`
- (internal) Cleaned up `wrapper` internal structure to some extent

Unfortunately, this version brought some minor breaking changes. Luckily most of
these have little to no impact:

- Changed `SqlArg` `value` and `attribute` members to be methods
  instead. Migration: replace `.value` and `.attribute` with `.value()` and
  `.attribute()`
- `get_type_coercion` now returns a `SqlType` instead of `Option<SqlType>`


## [0.3.10] - 2022-11-13

### Added

- Added dockerfile to build examples
- Added preliminary integration test framework

### Changed

- [CI] cleaned up pipeline configuration
- Refactor wrappers to make use of the new `if let ... else` statements
- Changed wrapper structs to make use of `UnsafeCell` for better mutability
  controlability

### Removed



## [0.3.9] - 2022-11-09

### Changed

- Gave a transparant repr to `ArgList`



## [0.3.8] - 2022-11-04

### Changed

- Updated documentation



## [0.3.7] - 2022-11-03

### Changed

- Fixed broken link to `README.md` in `udf/src/lib.rs`



## [0.3.6] - 2022-11-03

### Changed

- Changed `SqlResult::Decimal(&'a [u8])` to be `SqlResult::Decimal(&'a str)`,
  since a decimal will always fall under ASCII.



## [0.3.5] - 2022-11-03

### Added

- Added `Copy`, `Clone`, `Debug`, and `PartialEq` implementations to
  `MaxLenOptions`

### Changed

### Removed



## [0.3.4] - 2022-11-03

### Changed

- Adjusted semantics of `udf_log` output


## [0.3.3] - 2022-11-03

\[nonpublished\]



## [0.3.2] - 2022-11-03

### Changed

- Updated documentation around `AggregateUdf::Remove`



## [0.3.1] - 2022-11-03

### Added

- Example for `lipsum` is now working properly
- Improved documentation


## [0.3.0] - 2022-10-26

### Added

- Completed basic support for `#[register]` procedural macro
- Added support for strings (buffers) as return type

### Changed

- Changed trait signatures to include a `UdfCfg`
- Changed config name to `UdfCfg`, added typestate

### Removed


<!-- next-url -->
[Unreleased]: https://github.com/pluots/sql-udf/compare/v0.5.3...HEAD
[0.5.3]: https://github.com/pluots/sql-udf/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/pluots/sql-udf/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/pluots/sql-udf/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/pluots/sql-udf/compare/v0.4.5...v0.5.0
[0.4.5]: https://github.com/pluots/sql-udf/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/pluots/sql-udf/compare/v0.4.2...v0.4.4
[0.4.2]: https://github.com/pluots/sql-udf/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/pluots/sql-udf/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/pluots/sql-udf/compare/v0.3.10...v0.4.0
[0.3.10]: https://github.com/pluots/sql-udf/compare/v0.3.9...v0.3.10
[0.3.9]: https://github.com/pluots/sql-udf/compare/v0.3.8...v0.3.9
[0.3.8]: https://github.com/pluots/sql-udf/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/pluots/sql-udf/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/pluots/sql-udf/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/pluots/sql-udf/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/pluots/sql-udf/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/pluots/sql-udf/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/pluots/sql-udf/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/pluots/sql-udf/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/pluots/sql-udf/compare/v0.0.1...v0.3.0
