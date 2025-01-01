# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Introduced the `rendered` feature, allowing text rendering to `Sprite` and `ImageNode` components using `ImageFontPreRenderedText` and `ImageFontPreRenderedUiText`.
- **`atlas_sprites` Feature**: Allows text rendering with individual sprites for each character using a texture atlas.
  - Added `ImageFontSpriteText` component for text rendering via sprite atlases with configurable color and anchor.
  - Added support for optional gizmo rendering via the `gizmos` feature.
  - `atlased_sprite.rs` example: Demonstrates rendering text with the `atlas_sprites` feature.
- Added `ImageFontLoaderSettings` for specifying a custom `ImageSampler` to be used by the font asset loader.
- Enabled documentation for all features when building on `docs.rs` as well as enhanced documentation using the `doc_cfg` and `doc_auto_cfg` features.
- Expanded inline documentation for:
  - `ImageFont` and its fields (`atlas_layout`, `texture`, `atlas_character_map`).
  - `ImageFontLayout` variants, with detailed descriptions of layout methods.
  - `ImageFontSettings`, including details on `image` paths and `layout` mappings.
  - Error enums (`ImageFontLoadError` and `ImageFontRenderError`) with comprehensive explanations of possible errors.
- Added detailed comments to the `AtlasSpritesPlugin` and `ImageFontSpriteText` structs to describe their functionality.
- Added more Rust lint configuration to `Cargo.toml`.
- Added `ImageFontTextData` to track sprite-entity relationships and optimize sprite updates.
- Introduced a `validate` method to `ImageFontSettings` to ensure that the `image` path is non-empty and the layout string for `Automatic` is valid.
- Added `EmptyImagePath` and `EmptyLayoutString` variants to `ImageFontLoadError` for clearer error diagnostics.
- Added the `camino` crate (version 1.1.9) with `serde1` feature for robust UTF-8 path handling.
- Introduced stricter linting rules in `Cargo.toml` to enforce higher code quality standards.

### Changed

- Made rendering text function no longer `pub`; this is an internal implementation detail.
- Renamed `ImageFontSpriteText` to `ImageFontPreRenderedText` and `ImageFontUiText` to `ImageFontPreRenderedUiText`.
- Refactored text rendering systems into the `rendered` module, making them conditional on the `rendered` feature.
- Updated `Cargo.toml` to make the `image` dependency optional, activated only with the `rendered` feature.
- Updated `ImageFont` to include an `ImageSampler` field for enhanced texture sampling control.
- Renamed examples for clarity with the new `atlas_sprites` feature:
  - `sprite.rs` → `rendered_sprite.rs`
  - `bevy_ui.rs` → `rendered_ui.rs`
- Improved documentation for `ImageFontPlugin` to include usage examples and feature descriptions.
- Changed `atlased_sprite.rs` to demonstrate animated text and colors.
- Added detailed debug logs to monitor sprite creation, updates, and removals for `ImageFontSpriteText` and enhanced comments for better code clarity and maintainability.
- Enhanced `set_up_sprites` system to update, add, or remove sprites dynamically for `ImageFontSpriteText`.
- Replaced `PathBuf` with `Utf8PathBuf` in `ImageFontSettings` for stricter validation and compatibility with non-ASCII paths.
- Updated error messages in `ImageFontLoader` to provide more context when paths are invalid or images fail to load.

### Fixed

- Addressed inconsistencies in type annotations, parameter naming, and error handling across core systems and modules.

### Removed

- Redundant font setup logic in individual examples; replaced with reusable components in the `common` module.

### Notes

- This release introduces breaking changes.

## [0.6.0] - 2024-12-31

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

[unreleased]: https://github.com/ilyvion/bevy_image_font/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/ilyvion/bevy_image_font/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/ilyvion/bevy_image_font/compare/HEAD...v0.5.1
[0.5.0]: https://github.com/ilyvion/bevy_image_font/compare/c98d7a05c78be9e1bc8ce46145a2559754ff2924...v0.5.0
[0.4.0]: https://github.com/ilyvion/bevy_image_font/releases/tag/v0.4.0
