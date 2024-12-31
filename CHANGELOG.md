# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `ImageFontSpriteText` and `ImageFontUiText` were added as replacements for `ImageFontBundle` and `ImageFontUiBundle`. These now use Bevy 0.15's new required components to ensure that the entity has the components required to show the image font text.

### Changed

- Crate updated to target Bevy 0.14.

### Removed

- `ImageFontBundle` and `ImageFontUiBundle` have been removed.

## [0.5.1] - 2024-12-30

### Fixed

- Accidentally left behind some old repo links to the previous maintainer's repo.

## [0.5.0] - 2024-12-30

### Changed

- Crate renamed from `extol_image_font` to `bevy_image_font` and has a new maintainer.
- Crate updated to target Bevy 0.14.

### Fixed

- Only declare `ImageFontUiBundle` when feature `ui` is enabled, as it's useless otherwise and because `ImageBundle` is unavailable without `bevy`'s `bevy_ui` feature enabled.
- Clarify why text might sometimes render blurry in README. Update `sprite` example to illustrate work-arounds.

## [0.4.0] - 2024-04-04

- First public release; prior versions are not on Cargo.

[unreleased]: https://github.com/ilyvion/bevy_image_font/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/ilyvion/bevy_image_font/compare/HEAD...v0.5.1
[0.5.0]: https://github.com/ilyvion/bevy_image_font/compare/c98d7a05c78be9e1bc8ce46145a2559754ff2924...v0.5.0
[0.4.0]: https://github.com/ilyvion/bevy_image_font/releases/tag/v0.4.0
