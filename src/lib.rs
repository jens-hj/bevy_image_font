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

pub mod loader;

#[cfg(feature = "rendered")]
pub mod rendered;
#[cfg(feature = "rendered")]
pub use rendered::*;

#[cfg(feature = "atlas_sprites")]
pub mod atlas_sprites;
#[cfg(feature = "atlas_sprites")]
pub use atlas_sprites::*;

#[derive(Default)]
pub struct ImageFontPlugin;

impl Plugin for ImageFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ImageFont>()
            .init_asset_loader::<loader::ImageFontLoader>()
            .register_type::<ImageFont>()
            .register_type::<ImageFontText>()
            .add_systems(PostUpdate, mark_changed_fonts_as_dirty);

        #[cfg(feature = "rendered")]
        app.add_plugins(rendered::RenderedPlugin);

        #[cfg(feature = "atlas_sprites")]
        app.add_plugins(atlas_sprites::AtlasSpritesPlugin);
    }
}

/// System set for systems related to [`ImageFontPlugin`].
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct ImageFontSet;

/// An image font as well as the mapping of characters to regions inside it.
#[derive(Debug, Clone, Reflect, Asset)]
#[reflect(opaque)]
pub struct ImageFont {
    pub atlas_layout: Handle<TextureAtlasLayout>,
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

    #[allow(dead_code)]
    fn filter_string(&self, s: impl AsRef<str>) -> String {
        s.as_ref()
            .chars()
            .filter(|c| self.atlas_character_map.contains_key(c))
            .collect()
    }
}

/// Text rendered using an [`ImageFont`].
#[derive(Debug, Clone, Reflect, Default, Component, Setters)]
#[setters(into)]
pub struct ImageFontText {
    pub text: String,
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
