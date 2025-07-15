# Changelog

## 0.3.0 (future)

- Breaking: change `Segment::segment_id() -> Segment::id()`

## 0.2.1

- Use `Into<Color>` for methods that used to take a `Color`
- Add a few more docs
- Remove `async_trait` dependency

## 0.2.0

- Breaking: change `Command` struct and method names to be more consistent.
- Breaking: replaced `data()` method by several methods that return that data instead.
  - ie `Controller::data().name()` -> `Controller::name()`
- Most methods now take a `impl Iterator<Item = Color>`
- Fix some broken links in docs

## 0.1.0

Initial version, see the [original repo](https://github.com/nicoulaj/openrgb-rs) for the full history.
