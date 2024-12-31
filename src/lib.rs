#![doc = include_str!("../README.md")]
//
// only enables the `doc_cfg` feature when
// the `docsrs` configuration attribute is defined
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_image::{Image, ImageSampler};
use derive_setters::Setters;

#[cfg(any(feature = "rendered", feature = "atlas_sprites"))]
mod filtered_string;

pub mod loader;

#[cfg(feature = "rendered")]
pub mod rendered;
#[cfg(feature = "rendered")]
pub use rendered::*;

#[cfg(feature = "atlas_sprites")]
pub mod atlas_sprites;
#[cfg(feature = "atlas_sprites")]
pub use atlas_sprites::*;

/// A Bevy plugin for rendering image-based fonts.
///
/// This plugin enables support for fonts stored as single images (e.g., PNG),
/// where each glyph is represented by a section of the image. It handles:
///
/// - Loading `ImageFont` assets, which describe the glyph layout.
/// - Registering the `ImageFont` and `ImageFontText` types for use in your app.
/// - Marking updated fonts as dirty, ensuring proper re-rendering.
///
/// ### Features
/// The plugin conditionally includes additional functionality based on enabled
/// features:
/// - `rendered`: Enables support for rendering image fonts.
/// - `atlas_sprites`: Enables support for more advanced atlas-based sprite
///   functionality.
///
/// ### Usage
/// To use this plugin, add it to your Bevy app:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_image_font::ImageFontPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(ImageFontPlugin)
///     .run();
/// ```
///
/// Ensure that `ImageFont` assets are properly loaded and configured using the
/// asset system, and consider the relevant examples in the documentation for
/// advanced use cases.

#[derive(Debug, Default)]
pub struct ImageFontPlugin;

impl Plugin for ImageFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ImageFont>()
            .init_asset_loader::<loader::ImageFontLoader>()
            .register_type::<ImageFont>()
            .register_type::<ImageFontText>()
            .add_systems(PostUpdate, mark_changed_fonts_as_dirty);

        #[cfg(feature = "rendered")]
        app.add_plugins(RenderedPlugin);

        #[cfg(feature = "atlas_sprites")]
        app.add_plugins(AtlasSpritesPlugin);
    }
}

/// System set for systems related to [`ImageFontPlugin`].
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct ImageFontSet;

/// An image font as well as the mapping of characters to regions inside it.
#[derive(Debug, Clone, Reflect, Asset)]
#[reflect(opaque)]
pub struct ImageFont {
    /// The layout of the texture atlas, describing the positioning and sizes
    /// of glyphs within the image font. This layout is used to map character
    /// regions within the texture.
    pub atlas_layout: Handle<TextureAtlasLayout>,
    /// The image that contains the font glyphs. Each glyph is a section of
    /// this texture, defined by the `atlas_layout` and `atlas_character_map`.
    pub texture: Handle<Image>,
    /// The glyph used to render `c` is contained in the part of the image
    /// pointed to by `atlas.textures[atlas_character_map[c]]`.
    pub atlas_character_map: HashMap<char, usize>,
    /// The [`ImageSampler`] to use during font image rendering. The default is
    /// `nearest`, which scales an image without blurring, keeping the text
    /// crisp and pixellated.
    pub image_sampler: ImageSampler,
}

impl ImageFont {
    fn mapped_atlas_layout_from_char_map(
        size: UVec2,
        char_rect_map: &HashMap<char, URect>,
    ) -> (HashMap<char, usize>, TextureAtlasLayout) {
        let mut atlas_character_map = HashMap::new();
        let mut atlas_layout = TextureAtlasLayout::new_empty(size);
        for (&c, &rect) in char_rect_map {
            atlas_character_map.insert(c, atlas_layout.add_texture(rect));
        }

        (atlas_character_map, atlas_layout)
    }

    fn from_mapped_atlas_layout(
        texture: Handle<Image>,
        atlas_character_map: HashMap<char, usize>,
        atlas_layout: Handle<TextureAtlasLayout>,
        image_sampler: ImageSampler,
    ) -> Self {
        Self {
            atlas_layout,
            texture,
            atlas_character_map,
            image_sampler,
        }
    }

    #[cfg(any(feature = "rendered", feature = "atlas_sprites"))]
    fn filter_string<S: AsRef<str>>(&self, s: S) -> filtered_string::FilteredString<'_, S> {
        filtered_string::FilteredString::new(s, &self.atlas_character_map)
    }
}

/// Text rendered using an [`ImageFont`].
#[derive(Debug, Clone, Reflect, Default, Component, Setters)]
#[setters(into)]
pub struct ImageFontText {
    /// The string of text to be rendered. Each character in the string is
    /// mapped to a corresponding glyph in the associated [`ImageFont`].
    pub text: String,
    /// The handle to the [`ImageFont`] used to render this text. The font's
    /// texture and atlas mapping determine how characters are displayed.
    pub font: Handle<ImageFont>,
    /// If set, overrides the height the font is rendered at. This should be an
    /// integer multiple of the 'native' height if you want pixel accuracy,
    /// but we allow float values for things like animations.
    pub font_height: Option<f32>,
}

/// Marks any text where the underlying [`ImageFont`] asset has changed as
/// dirty, which will cause it to be rerendered.
#[allow(private_interfaces)]
pub fn mark_changed_fonts_as_dirty(
    mut events: EventReader<AssetEvent<ImageFont>>,
    mut query: Query<&mut ImageFontText>,
    mut changed_fonts: Local<CachedHashSet>,
) {
    changed_fonts.extend(events.read().copied().filter_map(|event| match event {
        AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
            info!("Image font {id} finished loading; marking as dirty");
            Some(id)
        }
        _ => None,
    }));

    for mut image_font_text in &mut query {
        if changed_fonts.contains(&image_font_text.font.id()) {
            image_font_text.set_changed();
        }
    }

    changed_fonts.clear();
}

#[derive(Default, Deref, DerefMut)]
struct CachedHashSet(HashSet<AssetId<ImageFont>>);
