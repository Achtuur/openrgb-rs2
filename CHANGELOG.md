# Changelog

This project uses [semantic versioning](https://semver.org/).

One caveat is that this project is still young, so some smaller breaking changes can be expected between updates, especially in newer API's.

<!--
  ### Added
  ### Changed
  ### Deprecated
  ### Removed
  ### Fixed
  ### Security
 -->

## [0.3.0] - Unreleased

### Added

- `ControllerMode` and `ControllerModeBuilder` to access and configure controller modes (#14).
- Added `Led` struct and API to access controller LEDs.
- [internal] Added (basic) plugin API
  - Added OpenRGBEffects plugin support
- [internal] Added more lints

### Changed

- Breaking: changed `Segment::segment_id() -> Segment::id()`
- Breaking: change `Zone::get_all_segments() -> Zone::segment_iter()`


## Fixed

- `Controller::init()` sets the brightness to max value. Some controllers were initialised with a brightness of 0 before.

## [0.2.1]

### Added

- Addded more documentation

### Changed

- Use `Into<Color>` for methods that used to take a `Color`
- Remove `async_trait` dependency

## [0.2.0]

### Changed

- Breaking: change `Command` struct and method names to be more consistent.
- Breaking: replaced `data()` method by several methods that return that data instead.
  - ie `Controller::data().name()` -> `Controller::name()`
- Most methods now take a `impl Iterator<Item = Color>`

### Fixed

- Fix some broken links in docs

## [0.1.0]

### Changed

- Changed the way parsing works. In short, the entire message is read to a buffer and then parsed, as opposed to being read byte-by-byte from stream.

## [0.1.0]

### Changed

- Changed the way parsing works. In short, the entire message is read to a buffer and then parsed, as opposed to being read byte-by-byte from stream.

Initial version, see the [original repo](https://github.com/nicoulaj/openrgb-rs) for the full history.
