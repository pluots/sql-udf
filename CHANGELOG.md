# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

### Changed

- Updated documentation

### Removed



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
[Unreleased]: https://github.com/pluots/stringmetrics/compare/v0.3.7...HEAD
[0.3.7]: https://github.com/pluots/stringmetrics/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/pluots/stringmetrics/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/pluots/stringmetrics/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/pluots/stringmetrics/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/pluots/stringmetrics/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/pluots/stringmetrics/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/pluots/stringmetrics/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/pluots/udf/compare/v0.0.1...v0.3.0
