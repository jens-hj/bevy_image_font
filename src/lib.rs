#![doc = include_str!("../README.md")]

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_image::Image;
use derive_setters::Setters;

pub mod loader;

#[cfg(feature = "rendered")]
pub mod rendered;
#[cfg(feature = "rendered")]
pub use rendered::*;

#[derive(Default)]
pub struct ImageFontPlugin;

impl Plugin for ImageFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ImageFont>()
            .init_asset_loader::<loader::ImageFontLoader>()
            .register_type::<ImageFont>()
            .register_type::<ImageFontText>();

        #[cfg(feature = "rendered")]
        app.add_systems(
            PostUpdate,
            (mark_changed_fonts_as_dirty, rendered::render_text_to_sprite)
                .chain()
                .in_set(ImageFontSet),
        );

        #[cfg(all(feature = "rendered", feature = "ui"))]
        {
            use bevy::ui::widget::update_image_content_size_system;
            app.add_systems(
                PostUpdate,
                rendered::render_text_to_image_node
                    .in_set(ImageFontSet)
                    .before(update_image_content_size_system)
                    .after(mark_changed_fonts_as_dirty),
            );
        }
    }
}

/// System set for systems related to [`ImageFontPlugin`].
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct ImageFontSet;

/// An image font as well as the mapping of characters to regions inside it.
#[derive(Debug, Clone, Reflect, Asset)]
pub struct ImageFont {
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
    /// The glyph used to render `c` is contained in the part of the image
    /// pointed to by `atlas.textures[atlas_character_map[c]]`.
    pub atlas_character_map: HashMap<char, usize>,
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
    ) -> Self {
        Self {
            atlas_layout,
            texture,
            atlas_character_map,
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
pub fn mark_changed_fonts_as_dirty(
    mut events: EventReader<AssetEvent<ImageFont>>,
    mut query: Query<&mut ImageFontText>,
) {
    let changed_fonts: HashSet<_> = events
        .read()
        .filter_map(|event| match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
                info!("Image font {id} finished loading; marking as dirty");
                Some(id)
            }
            _ => None,
        })
        .collect();
    for mut image_font_text in &mut query {
        if changed_fonts.contains(&image_font_text.font.id()) {
            image_font_text.set_changed();
        }
    }
}
