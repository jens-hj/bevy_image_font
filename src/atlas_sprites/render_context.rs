#![allow(clippy::unwrap_used, reason = "test code panics to indicate errors")]

//! This module provides the `RenderContext` type, which encapsulates
//! rendering-related computations and state for text sprites.
//!
//! # Purpose
//! The `RenderContext` simplifies and centralizes text rendering logic by
//! managing:
//! - Font assets, texture atlases, and associated metadata.
//! - Cached computations for glyph dimensions, text width, and alignment
//!   offsets.
//! - Helper methods for transforming and configuring sprites based on text
//!   content.
//!
//! By consolidating these responsibilities, `RenderContext` reduces code
//! duplication and makes rendering logic more maintainable.
//!
//! # Key Features
//! - **Asset Management**: Fetches and stores references to the font and
//!   texture atlas assets.
//! - **Text Calculations**: Computes text dimensions, glyph dimensions, and
//!   scaling factors.
//! - **Sprite Configuration**: Provides methods for updating sprite transforms,
//!   colors, and textures.
//! - **Caching**: Uses `CacheCell` to lazily compute and store values like
//!   maximum height and anchor offsets.
//!
//! This module is intended for internal use within the text rendering system
//! and is designed to work seamlessly with other components, such as
//! `SpriteContext`.

use std::cell::Cell;
use std::fmt::Debug;

use bevy::prelude::*;

use crate::atlas_sprites::anchors::AnchorExt as _;
use crate::atlas_sprites::{AnchorOffsets, ImageFontSpriteText, ImageFontTextData};
use crate::filtered_string::FilteredString;
use crate::{ImageFont, ImageFontText};

/// Groups font-related assets and configuration for rendering text sprites.
///
/// Includes references to the texture atlas layout, font asset, and the
/// font text component that defines the text content and font height.
pub(crate) struct RenderContext<'assets> {
    /// The texture atlas layout defining glyph placements.
    atlas_layouts: Vec<&'assets TextureAtlasLayout>,
    /// The font asset containing glyph metadata.
    image_font: &'assets ImageFont,
    /// The text component defining the content and font height.
    image_font_text: &'assets ImageFontText,
    /// The sprite configuration for rendering the text, including letter
    /// spacing and scaling mode.
    image_font_sprite_text: &'assets ImageFontSpriteText,
    /// The text filtered to include only supported characters in the font
    /// atlas.
    filtered_text: FilteredString<'assets, &'assets String>,

    /// Cached maximum glyph height.
    max_height: CacheCell<u32>,
}

impl<'assets> RenderContext<'assets> {
    /// Fetches the font and texture atlas assets needed for rendering text.
    ///
    /// Ensures that both the `ImageFont` and its associated
    /// `TextureAtlasLayout` are available. Logs an error if any required
    /// asset is missing.
    ///
    /// # Parameters
    /// - `image_fonts`: The collection of loaded font assets.
    /// - `font_handle`: Handle to the `ImageFont` asset to fetch.
    /// - `texture_atlas_layouts`: The collection of loaded texture atlas
    ///   layouts.
    ///
    /// # Returns
    /// An `Option` containing a tuple `(image_font, layout)` if both assets are
    /// successfully retrieved, or `None` if any asset is missing.
    #[inline]
    pub(crate) fn new(
        image_fonts: &'assets Assets<ImageFont>,
        image_font_text: &'assets ImageFontText,
        image_font_sprite_text: &'assets ImageFontSpriteText,
        texture_atlas_layouts: &'assets Assets<TextureAtlasLayout>,
        image_font_text_data: &mut ImageFontTextData,
    ) -> Option<Self> {
        let font_handle = &image_font_text.font;
        let Some(image_font) = image_fonts.get(font_handle) else {
            if !image_font_text_data.has_reported_missing_font {
                let font_handle_detail: &dyn Debug = if let Some(font_path) = font_handle.path() {
                    font_path
                } else {
                    &font_handle.id()
                };
                error!(
                    "ImageFont asset {font_handle_detail:?} is not loaded; can't render text for entity: {}",
                    image_font_text_data.self_entity
                );
                image_font_text_data.has_reported_missing_font = true;
            }
            return None;
        };

        let atlas_layouts: Result<Vec<_>, _> = image_font
            .atlas_layouts
            .iter()
            .map(|texture_atlas_layout| {
                texture_atlas_layouts
                    .get(texture_atlas_layout)
                    .ok_or_else(|| {
                        format!("TextureAtlasLayout not loaded: {texture_atlas_layout:?}")
                    })
            })
            .collect();

        let atlas_layouts = match atlas_layouts {
            Ok(layout) => layout,
            Err(error) => {
                error!("{error}");
                return None;
            }
        };

        let filtered_text = image_font.filter_string(&image_font_text.text);

        Some(RenderContext {
            atlas_layouts,
            image_font,
            image_font_text,
            image_font_sprite_text,
            filtered_text,

            max_height: default(),
        })
    }

    /// Computes the uniform scaling factor for text glyphs.
    ///
    /// Determines the scaling factor to apply to glyph dimensions based on
    /// the specified font height and the maximum glyph height.
    ///
    /// # Returns
    /// An `f32` representing the uniform scaling factor for text sprites.
    #[expect(
        clippy::cast_precision_loss,
        reason = "`max_height` won't ever be particularly large"
    )]
    #[inline]
    pub(crate) fn scale(&self) -> f32 {
        let max_height = self.max_height();
        self.image_font_text
            .font_height
            .map_or(1.0, |font_height| font_height / max_height as f32)
    }

    /// Calculates the maximum height of the filtered text.
    ///
    /// Iterates over the filtered text characters to determine the overall
    /// height based on glyph sizes in the texture atlas.
    ///
    /// # Returns
    /// The height of the tallest glyph
    #[inline]
    pub(crate) fn max_height(&self) -> u32 {
        self.max_height.get_or_insert_with(|| {
            let mut max_height = 1;

            for character in self.filtered_text.filtered_chars() {
                let image_font_character = &self.image_font.atlas_character_map[&character];
                let rect = self.atlas_layouts[image_font_character.page_index].textures
                    [image_font_character.character_index];
                max_height = max_height.max(rect.height());
            }

            max_height
        })
    }

    /// Calculates the total width of the rendered text based on the filtered
    /// characters and glyph dimensions stored in the context.
    ///
    /// # Returns
    /// The total width of the rendered text, in pixels, after applying the
    /// scaling factor.
    #[inline]
    pub(crate) fn text_width(&self) -> f32 {
        let mut text_width = 0.;

        for character in self.filtered_text.filtered_chars() {
            let (width, _) = self.character_dimensions(character);
            text_width += width;
        }
        text_width
    }

    /// Computes the dimensions of a glyph for a given character, scaled if a
    /// specific font height is provided.
    ///
    /// The dimensions are calculated based on the character's bounding
    /// rectangle in the texture atlas, along with the scaling mode and
    /// letter spacing settings configured in the context.
    ///
    /// # Parameters
    /// - `character`: The character whose dimensions are being computed.
    ///
    /// # Returns
    /// A tuple `(width, height)` representing the scaled or raw dimensions of
    /// the glyph.
    #[expect(
        clippy::cast_precision_loss,
        reason = "the magnitude of the numbers we're working on here are too small to lose anything"
    )]
    #[inline]
    pub(crate) fn character_dimensions(&self, character: char) -> (f32, f32) {
        let image_font_character = &self.image_font.atlas_character_map[&character];
        let rect = self.atlas_layouts[image_font_character.page_index].textures
            [image_font_character.character_index];
        let letter_spacing = self.image_font_sprite_text.letter_spacing.to_f32();
        let width = rect.width() as f32 + letter_spacing;
        let height = rect.height() as f32;
        let max_height = self.max_height() as f32;

        if let Some(font_height) = self.image_font_text.font_height {
            let scaling_mode = self.image_font_sprite_text.scaling_mode;
            let scale_factor = font_height / max_height;
            (
                scaling_mode.apply_scale(width, scale_factor),
                scaling_mode.apply_scale(height, scale_factor),
            )
        } else {
            (width, height)
        }
    }

    /// Retrieves the handle to the font texture image.
    ///
    /// This handle is used to assign the appropriate image to a text sprite.
    pub(crate) fn font_image(&self, character: char) -> Handle<Image> {
        let image_font_character = &self.image_font.atlas_character_map[&character];

        self.image_font.textures[image_font_character.page_index].clone_weak()
    }

    /// Constructs the texture atlas entry for a specific character.
    ///
    /// This method provides the texture atlas layout and the character's index
    /// within the atlas.
    ///
    /// # Parameters
    /// - `character`: The character whose texture atlas entry is needed.
    ///
    /// # Returns
    /// A [`TextureAtlas`] structure containing the layout and character index.
    pub(crate) fn font_texture_atlas(&self, character: char) -> TextureAtlas {
        let image_font_character = &self.image_font.atlas_character_map[&character];
        TextureAtlas {
            layout: self.image_font.atlas_layouts[image_font_character.page_index].clone_weak(),
            index: image_font_character.character_index,
        }
    }

    /// Returns the filtered text, which includes only characters supported by
    /// the font.
    ///
    /// The filtered text excludes unsupported or invalid characters, ensuring
    /// that only renderable glyphs are processed.
    pub(crate) fn text(&self) -> &FilteredString<'_, &String> {
        &self.filtered_text
    }

    /// Updates the sprite's texture and color values based on a specific
    /// character.
    ///
    /// # Parameters
    /// - `character`: The character associated with the sprite.
    /// - `texture_atlas`: The sprite's texture atlas, which is updated with the
    ///   character's index.
    /// - `color`: The sprite's color, which is set to the configured font
    ///   color.
    pub(crate) fn update_sprite_values(
        &self,
        character: char,
        texture_atlas: &mut TextureAtlas,
        color: &mut Color,
    ) {
        texture_atlas.index = self.image_font.atlas_character_map[&character].character_index;
        *color = self.image_font_sprite_text.color;
    }

    /// Computes or retrieves the cached anchor offsets for the text and glyph
    /// alignment.
    ///
    /// The anchor offsets include:
    /// - `whole`: Alignment for the entire text block.
    /// - `individual`: Alignment for each glyph relative to the text block.
    ///
    /// The offsets are computed lazily and cached for reuse.
    pub(crate) fn anchor_offsets(&self) -> AnchorOffsets {
        self.image_font_sprite_text.anchor.to_anchor_offsets()
    }

    /// Computes the transform for positioning and scaling a text sprite.
    ///
    /// This method calculates the sprite's translation and scale based on:
    /// - The x-position of the sprite.
    /// - The dimensions of the character's glyph.
    /// - The alignment offsets and scaling configuration.
    ///
    /// # Parameters
    /// - `x_pos`: A mutable reference to the current x-position of the sprite.
    ///   This value is updated to reflect the position of the next sprite.
    /// - `character`: The character associated with the sprite.
    ///
    /// # Returns
    /// A [`Transform`] representing the position and scale of the sprite.
    pub(crate) fn transform(&self, x_pos: &mut f32, character: char) -> Transform {
        let x = *x_pos;
        let (width, _) = self.character_dimensions(character);
        *x_pos += width;
        self.anchor_offsets().compute_transform(
            x,
            self.text_width(),
            width,
            self.max_height(),
            self.scale(),
        )
    }
}

/// A lightweight wrapper around a [`Cell<Option<T>>`] for caching values.
///
/// This utility type provides a mechanism to lazily compute and cache a value
/// using a factory function. It is ideal for scenarios where a value is
/// expensive to compute but may not always be needed.
///
/// # Features
/// - Lazily initializes the cached value when accessed for the first time.
/// - Ensures the cached value is reused across multiple accesses.
/// - Supports types that implement the [`Copy`] trait.
struct CacheCell<T>(Cell<Option<T>>);

impl<T> CacheCell<T> {
    /// Retrieves the cached value, computing it if necessary.
    ///
    /// If the value is already cached, this method returns it directly. If not,
    /// it invokes the provided factory function to compute the value, caches
    /// it, and then returns it.
    ///
    /// # Parameters
    /// - `factory`: A closure that computes the value if it is not already
    ///   cached.
    ///
    /// # Returns
    /// The cached value, or the newly computed value if the cache was empty.
    fn get_or_insert_with<F>(&self, factory: F) -> T
    where
        T: Copy,
        F: FnOnce() -> T,
    {
        if let Some(value) = self.0.get() {
            value
        } else {
            let value = factory();
            self.0.set(Some(value));
            value
        }
    }
}

impl<T> Default for CacheCell<T> {
    /// Creates a new, empty `CacheCell`.
    ///
    /// The cache will initialize with no value and will compute the value on
    /// first access via `get_or_insert_with`.
    fn default() -> Self {
        Self(default())
    }
}

#[cfg(test)]
mod tests;
