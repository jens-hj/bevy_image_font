[![Crates.io](https://img.shields.io/crates/v/bevy_image_font)](https://crates.io/crates/bevy_image_font)
[![Crates.io](https://img.shields.io/crates/l/bevy_image_font)](https://crates.io/crates/bevy_image_font)
[![Crates.io](https://img.shields.io/crates/d/bevy_image_font)](https://crates.io/crates/bevy_image_font)
[![Docs.io](https://docs.rs/bevy_image_font/badge.svg)](https://docs.rs/bevy_image_font)
[![Docs master](https://img.shields.io/static/v1?label=docs&message=master&color=5479ab)](https://ilyvion.github.io/bevy_image_font/)
[![Rust](https://github.com/ilyvion/bevy_image_font/actions/workflows/CI.yml/badge.svg)](https://github.com/ilyvion/bevy_image_font/actions/workflows/CI.yml)

`bevy_image_font` allows rendering fonts that are stored as a single image (typically PNG), with each letter at a given location. This is common in game development, especially for pixel art fonts, since it allows the use of colors and creating a font can be done using any image editor as opposed to specialized software. These are also sometimes known as 'pixel fonts', but I choose the name 'image font' to be more precise (since bitmap fonts stored in OTB could also be called 'pixel fonts').

## Features

**Supported**

- Unicode (anything that fits in a single codepoint)
- Specifying the coordinates with a string containing the letters in proper order (see the example asset)
- Manually specifying the rects (including non-uniform sizes)

**Future work**

- Padding and offsets for automatic texture layout
- Newlines embedding in strings

**Out of scope**

- Rendering from 'actual' bitmap fonts
- Automatic line wrapping

### Caveats

- You need to have a portion of the texture that's just blank and 'map' the space character to it.
- Newlines are not currently supported.

## Usage

```toml
[dependencies]
bevy = "0.14"
bevy_image_font = "0.6"
```

### How to use

If your text sprites aren't pixel-accurate, note that Bevy anchors sprites at the center by default. This causes sprites with odd pixel dimensions to land on non-integer positions, resulting in blurry rendering. To fix this, use a non-`Center` anchor like `Anchor::TopLeft` or adjust the translation of centered sprites. See [the sprite example] for more details.

Just take any entity with a `Handle<Image>` or `UiImage` component, such as something created with a `SpriteBundle` or `ImageBundle`, and add a `ImageFontText` component to it.

See [the bevy_ui example] for sample usage using the `bevy_asset_loader` crate to construct handles to the texture layout and image, or [the sprite example] if you want to use pixel fonts 'in the world' (such as for flying damage text).

[the sprite example]: https://github.com/ilyvion/bevy_image_font/blob/main/examples/sprite.rs
[the bevy_ui example]: https://github.com/ilyvion/bevy_image_font/blob/main/examples/bevy_ui.rs

If you're not using `bevy_ui`, you can disable the `bevy_ui` feature (enabled by default) to avoid taking a dependency on that.

This crate uses the `image` crate to load images, but only enables PNG support by default. If you need some other format, add your own dependency on (the same version of) `image` and enable the relevant features.

## Bevy Version Support

I intend to track the latest release version of Bevy. PRs supporting this are welcome!

| bevy | bevy_image_font |
| ---- | --------------- |
| 0.14 | 0.5             |

## Contributing

Please run `git config --local core.hooksPath .githooks` after you have cloned the repo to make sure your local Git repo is configured to run our Git hooks, which takes care of things like not allowing you to commit code that doesn't follow our coding standards. These hooks require the following additional tools:

- `cargo-hack`: `cargo install cargo-hack --locked`

## Credits

The sample font is by [gnsh](https://opengameart.org/content/bitmap-font-0).

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
