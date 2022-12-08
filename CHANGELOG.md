# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

### Changed

### Removed



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
[Unreleased]: https://github.com/pluots/stringmetrics/compare/v0.4.2...HEAD
[0.4.2]: https://github.com/pluots/stringmetrics/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/pluots/stringmetrics/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/pluots/stringmetrics/compare/v0.3.10...v0.4.0
[0.3.10]: https://github.com/pluots/stringmetrics/compare/v0.3.9...v0.3.10
[0.3.9]: https://github.com/pluots/stringmetrics/compare/v0.3.8...v0.3.9
[0.3.8]: https://github.com/pluots/stringmetrics/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/pluots/stringmetrics/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/pluots/stringmetrics/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/pluots/stringmetrics/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/pluots/stringmetrics/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/pluots/stringmetrics/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/pluots/stringmetrics/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/pluots/stringmetrics/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/pluots/udf/compare/v0.0.1...v0.3.0
