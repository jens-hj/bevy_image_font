# bevy_image_font

[![Crates.io](https://img.shields.io/crates/v/bevy_image_font)](https://crates.io/crates/bevy_image_font)
[![Crates.io](https://img.shields.io/crates/l/bevy_image_font)](https://crates.io/crates/bevy_image_font)
[![Crates.io](https://img.shields.io/crates/d/bevy_image_font)](https://crates.io/crates/bevy_image_font)
[![Docs.rs](https://docs.rs/bevy_image_font/badge.svg)](https://docs.rs/bevy_image_font)
[![Docs main](https://img.shields.io/static/v1?label=docs&message=main&color=5479ab)](https://ilyvion.github.io/bevy_image_font/)
[![Build Status](https://github.com/ilyvion/bevy_image_font/actions/workflows/CI.yml/badge.svg)](https://github.com/ilyvion/bevy_image_font/actions/workflows/CI.yml)

`bevy_image_font` enables rendering fonts stored as single images (e.g., PNG), with each letter at a predefined position. This crate focuses specifically on image-based fonts, often called "pixel fonts," used in game development. The term "image font" was chosen for precision, as bitmap fonts in formats like OTB might also be referred to as "pixel fonts."

## Features

### Supported

- Unicode (single codepoints)
- Defining character coordinates via strings (see example asset)
- Manual specification of rectangles (including non-uniform sizes)

### Planned Enhancements

- Padding and offsets for texture layouts
- Inline newlines in strings

### Out of Scope

- Rendering from traditional bitmap fonts
- Automatic line wrapping

### Known Limitations

- Space characters require a blank texture region.
- Newlines are currently unsupported.

## Getting Started

Add the following to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.15"
bevy_image_font = "0.6"
```

### Usage

Add an `ImageFontText` component to an entity with either a `Sprite` or `ImageNode` component. This will render the text onto the associated texture.

#### Minimal Example

Here's a minimal example of using `bevy_image_font` to render text:

```rust,no_run
use bevy::prelude::*;
use bevy_image_font::{ImageFontPlugin, ImageFontText, ImageFontPreRenderedText};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ImageFontPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("path/to/font_layout.image_font.ron");

    commands.spawn((
        ImageFontPreRenderedText,
        ImageFontText::default()
            .text("Hello, world!")
            .font(font_handle.clone()),
    ));
}
```

This example sets up a simple Bevy application with an `ImageFontText` component, rendering "Hello, world!" using a specified font image and layout.

See examples for more details:

- [Sprite example](https://github.com/ilyvion/bevy_image_font/blob/main/examples/sprite.rs): Using pixel fonts for in-world text like damage numbers.
- [Bevy UI example](https://github.com/ilyvion/bevy_image_font/blob/main/examples/bevy_ui.rs): Using `bevy_asset_loader` for texture and font handling.

#### Note on Pixel Accuracy

Bevy anchors sprites at the center by default, which may cause odd-dimensioned sprites to appear blurry. To avoid this, use non-`Center` anchors like `Anchor::TopLeft` or adjust sprite translations. Refer to the [sprite example](https://github.com/ilyvion/bevy_image_font/blob/main/examples/sprite.rs) for details.

### Optional Features

- Disable the default `bevy_ui` feature if unused to minimize dependencies.
- The `image` crate is already a dependency of `bevy_image_font`. If your project depends on this crate and you need support for non-PNG formats, add your own dependency on the same version of `image` and enable the relevant features.

## Bevy Version Compatibility

| Bevy Version | Crate Version |
| ------------ | ------------- |
| 0.15         | 0.6           |
| 0.14         | 0.5           |

## Changelog

For detailed changes across versions, see the [Changelog](CHANGELOG.md). Each GitHub Release which is created each time the crate is published also includes the relevant section of the changelog in its release notes for easy reference.

## Contributing

1. Configure Git hooks after cloning:
   ```bash
   git config --local core.hooksPath .githooks
   ```
2. Install required tools:
   ```bash
   cargo install cargo-hack --locked
   ```

PRs to support the latest Bevy releases are welcome!

## Credits

Sample font by [gnsh](https://opengameart.org/content/bitmap-font-0).

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
