# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2024-12-30

### Fixed

- Only declare `ImageFontUiBundle` when feature `ui` is enabled, as it's useless otherwise and because `ImageBundle` is unavailable without `bevy`'s `bevy_ui` feature enabled.
- Clarify why text might sometimes render blurry in README. Update `sprite` example to illustrate work-arounds.

## [0.4.0] - 2024-04-04

- First public release; prior versions are not on Cargo.

[0.4.0]: https://github.com/ilyvion/bevy_image_font/releases/tag/v0.4.0
[unreleased]: https://github.com/ilyvion/bevy_image_font/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/ilyvion/bevy_image_font/compare/c98d7a05c78be9e1bc8ce46145a2559754ff2924...v0.5.0
