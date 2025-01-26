//! This module provides functionality for rendering text as individual sprites
//! using the Bevy engine, utilizing custom image fonts.
//!
//! It breaks down text into individual characters and represents them as
//! sprites in the game world. This approach allows precise positioning and
//! styling of text at the character level, suitable for scenarios where text
//! needs to be rendered dynamically or interactively.
//!
//! Key Features:
//! - `ImageFontSpriteText` component: Allows customization of text rendering,
//!   such as color and anchor point.
//! - Systems for rendering text to sprite entities and updating their
//!   configuration when text changes.
//! - Optional gizmo rendering for debugging purposes, available with the
//!   "gizmos" feature flag.
//!
//! This module is intended for advanced text rendering use cases, offering
//! fine-grained control over how text is displayed in the game world.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use derive_setters::Setters;

use crate::{filtered_string, LetterSpacing};
use crate::{sync_texts_with_font_changes, ImageFont, ImageFontSet, ImageFontText};

/// Internal plugin for conveniently organizing the code related to this
/// module's feature.
#[derive(Default)]
pub(crate) struct AtlasSpritesPlugin;

impl Plugin for AtlasSpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            set_up_sprites
                .after(sync_texts_with_font_changes)
                .in_set(ImageFontSet),
        );

        #[cfg(feature = "gizmos")]
        {
            app.add_systems(Update, render_sprite_gizmos);
        }
    }
}

/// Text rendered using an [`ImageFont`] as individual sprites.
///
/// This struct provides fields for customizing text rendering, such as
/// alignment, color, and scaling behavior.
///
/// - `anchor`: Specifies the alignment point of the text relative to its
///   position.
/// - `color`: Uniform tint applied to all glyphs.
/// - `scaling_mode`: Controls how scaling is applied to glyph dimensions.
#[derive(Debug, Clone, Reflect, Default, Component, Setters)]
#[setters(into)]
#[require(ImageFontText, Visibility)]
pub struct ImageFontSpriteText {
    /// The alignment point of the text relative to its position. For example,
    /// `Anchor::TopLeft` aligns the text's top-left corner to its position.
    pub anchor: Anchor,

    /// The color applied to the rendered text. This color affects all glyphs
    /// equally, allowing you to tint the text uniformly.
    pub color: Color,

    /// Determines how scaling is applied to the glyph dimensions when adjusting
    /// them to match the desired font height.
    ///
    /// This field allows control over how fractional scaling values are
    /// handled, using the [`ScalingMode`] enum. It provides options to
    /// truncate, round, or retain precise fractional values, depending on
    /// your rendering requirements.
    ///
    /// The default value is `ScalingMode::Rounded`.
    pub scaling_mode: ScalingMode,

    /// Determines a constant kerning between characters.
    pub letter_spacing: LetterSpacing,
}

/// Determines how scaling is applied when calculating the dimensions of a
/// character glyph.
///
/// This enum is used to control how fractional values are handled when scaling
/// glyph dimensions to fit a specified font height. It provides options for
/// truncating, rounding, or retaining precise values, offering flexibility
/// based on the rendering requirements.
#[derive(Debug, Clone, Copy, Reflect, Default)]
pub enum ScalingMode {
    /// Truncates fractional values during scaling.
    ///
    /// This mode ensures that the width and height of the glyph are always
    /// rounded down to the nearest whole number. It can be useful for
    /// pixel-perfect rendering where fractional dimensions could cause
    /// visual artifacts.
    Truncated,

    /// Rounds fractional values during scaling.
    ///
    /// This mode rounds the width and height of the glyph to the nearest whole
    /// number. It offers a balance between precision and consistency, often
    /// used when slight inaccuracies are acceptable but extreme rounding
    /// errors need to be avoided.
    ///
    /// This is the default scaling mode.
    #[default]
    Rounded,

    /// Retains precise fractional values during scaling.
    ///
    /// This mode avoids rounding entirely, keeping the scaled dimensions as
    /// floating-point values. It is ideal for high-precision rendering or
    /// cases where exact scaling is necessary, such as when performing
    /// sub-pixel positioning.
    Smooth,
}

#[derive(Debug, Clone, Default, Component)]
struct ImageFontTextData {
    /// Basically a map between character index and character sprite
    sprites: Vec<Entity>,
}

/// Debugging data for visualizing an `ImageFontSpriteText` in a scene, enabled
/// by the `gizmos` feature.
#[cfg(feature = "gizmos")]
#[derive(Debug, Clone, Default, Component)]
pub struct ImageFontGizmoData {
    /// The width of the gizmo, representing the rendered font's bounding box
    /// or visualized area in the scene.
    width: f32,

    /// The height of the gizmo, representing the rendered font's bounding box
    /// or visualized area in the scene.
    height: f32,
}

/// System that renders each [`ImageFontText`] as child [`Sprite`] entities
/// where each sprite represents a character in the text. That is to say, each
/// sprite gets positioned accordingly to its position in the text. This
/// system only runs when the `ImageFontText` or [`ImageFontSpriteText`]
/// changes.
#[expect(
    clippy::missing_panics_doc,
    reason = "expect() is only used on a newly created Some() value"
)]
#[expect(
    private_interfaces,
    reason = "Systems are only `pub` for the sake of allowing dependent crates to use them for ordering"
)]
pub fn set_up_sprites(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &ImageFontText,
            &ImageFontSpriteText,
            Option<&mut ImageFontTextData>,
        ),
        Or<(Changed<ImageFontText>, Changed<ImageFontSpriteText>)>,
    >,
    #[cfg(not(feature = "gizmos"))] mut child_query: Query<(&mut Sprite, &mut Transform)>,
    #[cfg(feature = "gizmos")] mut child_query: Query<(
        &mut Sprite,
        &mut Transform,
        &mut ImageFontGizmoData,
    )>,
    image_fonts: Res<Assets<ImageFont>>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for (entity, image_font_text, image_font_sprite_text, mut image_font_text_data) in &mut query {
        let mut maybe_new_image_font_text_data = None;
        let image_font_text_data = if let Some(image_font_text_data) = image_font_text_data.as_mut()
        {
            &mut *image_font_text_data
        } else {
            maybe_new_image_font_text_data = Some(ImageFontTextData::default());
            #[expect(clippy::expect_used, reason = "newly created Some() value")]
            maybe_new_image_font_text_data
                .as_mut()
                .expect("newly created Some() value")
        };

        let Some((image_font, layout)) =
            fetch_assets(&image_fonts, &image_font_text.font, &texture_atlas_layouts)
        else {
            continue;
        };

        let text = image_font.filter_string(&image_font_text.text);
        let max_height = calculate_text_height(&text, layout, image_font);
        let scale = calculate_scale(image_font_text.font_height, max_height);
        let total_width = calculate_text_width(
            image_font_text,
            image_font_sprite_text,
            image_font,
            layout,
            &text,
            max_height,
        );
        let anchors = calculate_anchors(image_font_sprite_text.anchor);

        let font_assets = FontAssets {
            layout,
            image_font,
            image_font_text,
        };

        let sprite_layout = SpriteLayout {
            max_height,
            total_width,
            scale,
            anchors,
        };

        let mut sprite_context = SpriteContext {
            entity,
            image_font_text_data,
            text: &text,
        };

        let x_pos = update_existing_sprites(
            &mut child_query,
            &mut sprite_context,
            &font_assets,
            &sprite_layout,
            image_font_text,
            image_font_sprite_text,
        );

        adjust_sprite_count(
            x_pos,
            &mut commands,
            &mut sprite_context,
            &font_assets,
            &sprite_layout,
            image_font_sprite_text,
        );

        if let Some(new_image_font_text_data) = maybe_new_image_font_text_data {
            debug!("Inserted new ImageFontTextData for entity {:?}", entity);
            commands.entity(entity).insert(new_image_font_text_data);
        }
    }
}

/// Fetches the font and texture atlas assets needed for rendering text.
///
/// Ensures that both the `ImageFont` and its associated `TextureAtlasLayout`
/// are available. Logs an error if any required asset is missing.
///
/// # Parameters
/// - `image_fonts`: The collection of loaded font assets.
/// - `font_handle`: Handle to the `ImageFont` asset to fetch.
/// - `texture_atlas_layouts`: The collection of loaded texture atlas layouts.
///
/// # Returns
/// An `Option` containing a tuple `(image_font, layout)` if both assets are
/// successfully retrieved, or `None` if any asset is missing.
#[inline]
fn fetch_assets<'assets>(
    image_fonts: &'assets Res<Assets<ImageFont>>,
    font_handle: &Handle<ImageFont>,
    texture_atlas_layouts: &'assets Res<Assets<TextureAtlasLayout>>,
) -> Option<(&'assets ImageFont, &'assets TextureAtlasLayout)> {
    let Some(image_font) = image_fonts.get(font_handle) else {
        error!("ImageFont asset not loaded: {:?}", font_handle);
        return None;
    };

    let Some(layout) = texture_atlas_layouts.get(&image_font.atlas_layout) else {
        error!(
            "TextureAtlasLayout not loaded: {:?}",
            image_font.atlas_layout
        );
        return None;
    };

    Some((image_font, layout))
}

/// Calculates the maximum height of the filtered text.
///
/// Iterates over the filtered text characters to determine the overall
/// height based on glyph sizes in the texture atlas.
///
/// # Parameters
/// - `text`: The filtered text to measure.
/// - `layout`: The texture atlas layout containing glyph sizes.
/// - `image_font`: The font asset mapping characters to glyph indices.
///
/// # Returns
/// The height of the tallest glyph
#[inline]
fn calculate_text_height(
    text: &filtered_string::FilteredString<'_, impl AsRef<str>>,
    layout: &TextureAtlasLayout,
    image_font: &ImageFont,
) -> u32 {
    let mut max_height = 1;

    for character in text.filtered_chars() {
        let rect = layout.textures[image_font.atlas_character_map[&character]];
        max_height = max_height.max(rect.height());
    }

    max_height
}

/// Calculates the total width of the rendered text based on the given font and
/// layout.
///
/// This function iterates through the filtered characters of the text and sums
/// their scaled widths to determine the total width of the text when rendered.
///
/// # Parameters
/// - `image_font_text`: The [`ImageFontText`] component containing the font
///   height and text to be rendered.
/// - `scaling_mode`: Determines how fractional widths are handled (truncated,
///   rounded, or precise).
/// - `image_font`: The [`ImageFont`] asset that maps characters to glyph
///   indices.
/// - `layout`: The [`TextureAtlasLayout`] defining the bounding rectangles for
///   each glyph.
/// - `text`: The filtered string representing the text to render, excluding
///   unsupported characters.
/// - `max_height`: The maximum glyph height in the text, used to calculate
///   scaling factors.
///
/// # Returns
/// The total width of the rendered text, in pixels, after applying the scaling
/// factor.
///
/// # Panics
/// This function assumes that all characters in `text` exist in the
/// `image_font`'s atlas and that `max_height` is non-zero. If this assumption
/// is violated, it may panic.
#[inline]
fn calculate_text_width(
    image_font_text: &ImageFontText,
    sprite_text: &ImageFontSpriteText,
    image_font: &ImageFont,
    layout: &TextureAtlasLayout,
    text: &filtered_string::FilteredString<'_, &String>,
    max_height: u32,
) -> f32 {
    let mut total_width = 0.;

    for character in text.filtered_chars() {
        let rect = layout.textures[image_font.atlas_character_map[&character]];
        let (width, _) = compute_dimensions(
            rect,
            image_font_text.font_height,
            max_height,
            sprite_text.letter_spacing,
            sprite_text.scaling_mode,
        );
        let letter_spacing: f32 = sprite_text.letter_spacing.into();
        total_width += width + letter_spacing;
    }
    total_width
}

/// Computes the uniform scaling factor for text glyphs.
///
/// Determines the scaling factor to apply to glyph dimensions based on
/// the specified font height and the maximum glyph height.
///
/// # Parameters
/// - `font_height`: Optional target font height. Defaults to no scaling.
/// - `max_height`: The maximum glyph height in the text.
///
/// # Returns
/// A `Vec3` representing the uniform scaling factor for text sprites.
#[expect(
    clippy::cast_precision_loss,
    reason = "`max_height` won't ever be particularly large"
)]
#[inline]
fn calculate_scale(font_height: Option<f32>, max_height: u32) -> Vec3 {
    let scale = font_height.map_or(1.0, |font_height| font_height / max_height as f32);
    Vec3::new(scale, scale, 0.0)
}

/// Calculates anchor offsets for aligning text and glyphs.
///
/// Computes the offsets needed to align the entire text block (`whole`)
/// and individual glyphs (`individual`) based on the provided `Anchor`.
///
/// # Parameters
/// - `anchor`: The alignment configuration for positioning text.
///
/// # Returns
/// An `Anchors` struct containing:
/// - `whole`: Offset for aligning the entire text block.
/// - `individual`: Offset for aligning each individual glyph.
#[inline]
fn calculate_anchors(anchor: Anchor) -> Anchors {
    let anchor_vec = anchor.as_vec();
    Anchors {
        whole: -(anchor_vec + Vec2::new(0.5, 0.0)),
        individual: Vec2::new(0.5, 0.0),
    }
}

/// Updates existing sprites to match the filtered text content.
///
/// Adjusts the position, scale, and appearance of each sprite to reflect
/// the corresponding glyph in the text and texture atlas.
///
/// # Parameters
/// - `child_query`: Query for accessing child sprite components.
/// - `sprite_context`: Context for managing the entity and its sprite data.
/// - `font_assets`: Font-related assets and configuration.
/// - `sprite_layout`: Precomputed layout and scaling information.
/// - `sprite_text`: Component defining text appearance (e.g., color).
///
/// # Returns
/// The x-position to the right of the last processed sprite.
fn update_existing_sprites(
    #[cfg(not(feature = "gizmos"))] child_query: &mut Query<(&mut Sprite, &mut Transform)>,
    #[cfg(feature = "gizmos")] child_query: &mut Query<(
        &mut Sprite,
        &mut Transform,
        &mut ImageFontGizmoData,
    )>,
    sprite_context: &mut SpriteContext<impl AsRef<str>>,
    font_assets: &FontAssets,
    sprite_layout: &SpriteLayout,
    font_text: &ImageFontText,
    sprite_text: &ImageFontSpriteText,
) -> f32 {
    let SpriteLayout {
        max_height,
        total_width,
        scale,
        anchors:
            Anchors {
                individual: anchor_vec_individual,
                whole: anchor_vec_whole,
            },
    } = *sprite_layout;

    let FontAssets {
        layout, image_font, ..
    } = *font_assets;

    let SpriteContext {
        ref mut image_font_text_data,
        text,
        ..
    } = *sprite_context;

    let mut x_pos = 0.;

    for (sprite_entity, character) in image_font_text_data
        .sprites
        .iter()
        .copied()
        .zip(text.filtered_chars())
    {
        #[cfg(not(feature = "gizmos"))]
        let (mut sprite, mut transform) = match child_query.get_mut(sprite_entity) {
            Ok(result) => result,
            Err(error) => {
                error!("An ImageFontSpriteText unexpectedly failed: {error}. This will likely cause rendering bugs.");
                continue;
            }
        };

        #[cfg(feature = "gizmos")]
        let (mut sprite, mut transform, mut gizmo_data) = match child_query.get_mut(sprite_entity) {
            Ok(result) => result,
            Err(error) => {
                error!("An ImageFontSpriteText unexpectedly failed: {error}. This will likely cause rendering bugs.");
                continue;
            }
        };

        let Some(sprite_texture) = sprite.texture_atlas.as_mut() else {
            error!("An ImageFontSpriteText's child sprite was unexpectedly missing a `texture_atlas`. This will likely cause rendering bugs.");
            continue;
        };

        sprite_texture.index = image_font.atlas_character_map[&character];
        sprite.color = sprite_text.color;

        let rect = layout.textures[image_font.atlas_character_map[&character]];
        let (width, _height) = compute_dimensions(
            rect,
            font_text.font_height,
            max_height,
            sprite_text.letter_spacing,
            sprite_text.scaling_mode,
        );

        *transform = compute_transform(
            x_pos,
            total_width,
            width,
            max_height,
            scale,
            anchor_vec_whole,
            anchor_vec_individual,
        );

        x_pos += width;

        #[cfg(feature = "gizmos")]
        #[expect(
            clippy::used_underscore_binding,
            reason = "we're using an underscore binding here because it's unused when the \
            `gizmos` feature is not enabled."
        )]
        {
            gizmo_data.width = width;
            gizmo_data.height = _height;
        }
    }

    x_pos
}

/// Computes the dimensions of a glyph rectangle, scaled if a specific font
/// height is provided.
///
/// This function calculates the width and height of a glyph rectangle based on
/// its raw size (from the texture atlas) and optionally scales it to match a
/// specified font height. If no `font_height` is provided, the raw dimensions
/// are returned.
///
/// # Parameters
/// - `rect`: The bounding rectangle of the glyph in the texture atlas. The
///   width and height of this rectangle represent the raw glyph dimensions in
///   pixels.
/// - `font_height`: An optional scaling target for the height of the glyph. If
///   provided, the glyph's dimensions are scaled proportionally to match this
///   height.
/// - `max_height`: The maximum glyph height in the text, used to calculate the
///   scaling factor when `font_height` is provided.
/// - `scaling_mode`: Specifies how scaling is applied (e.g., truncation,
///   rounding, or precise).
///
/// # Returns
/// A tuple `(width, height)` representing the scaled or raw dimensions of the
/// glyph.
///
/// # Panics
/// This function assumes that `max_height` is non-zero when `font_height` is
/// provided. If `max_height` is zero, behavior is undefined and may result in a
/// panic.
#[expect(
    clippy::cast_precision_loss,
    reason = "the magnitude of the numbers we're working on here are too small to lose anything"
)]
#[inline]
fn compute_dimensions(
    rect: URect,
    font_height: Option<f32>,
    max_height: u32,
    letter_spacing: LetterSpacing,
    scaling_mode: ScalingMode,
) -> (f32, f32) {
    let letter_spacing: f32 = letter_spacing.into();
    let width = rect.width() as f32 + letter_spacing;
    let height = rect.height() as f32;
    let max_height = max_height as f32;
    font_height.map_or((width, height), |fh| match scaling_mode {
        ScalingMode::Truncated => (
            (width * fh / max_height).trunc(),
            (height * fh / max_height).trunc(),
        ),
        ScalingMode::Rounded => (
            (width * fh / max_height).round(),
            (height * fh / max_height).round(),
        ),
        ScalingMode::Smooth => ((width * fh / max_height), (height * fh / max_height)),
    })
}

/// Computes the transform for positioning and scaling a text sprite.
///
/// Calculates the sprite's translation and scaling based on its position
/// within the text block, the total dimensions, scaling factors, and anchor
/// offsets.
///
/// # Parameters
/// - `x_pos`: Current x-position of the sprite.
/// - `total_width`: Total width of the text block.
/// - `width`: Width of the current glyph.
/// - `max_height`: Maximum height of the text block.
/// - `scale`: Scaling factor for glyph dimensions.
/// - `whole_anchor`: Offset for aligning the entire text block.
/// - `individual_anchor`: Offset for aligning the individual glyph.
///
/// # Returns
/// A `Transform` object representing the sprite's position and scale.
#[expect(
    clippy::cast_precision_loss,
    reason = "we're working on numbers small enough not to be affected"
)]
#[inline]
fn compute_transform(
    x_pos: f32,
    total_width: f32,
    width: f32,
    max_height: u32,
    scale: Vec3,
    anchor_vec_whole: Vec2,
    anchor_vec_individual: Vec2,
) -> Transform {
    Transform::from_translation(Vec3::new(
        x_pos + total_width * anchor_vec_whole.x + width * anchor_vec_individual.x,
        max_height as f32 * anchor_vec_whole.y * scale.y,
        0.0,
    ))
    .with_scale(scale)
}

/// Ensures the number of sprites matches the number of characters in the text.
///
/// Adds missing sprites or removes excess sprites to maintain consistency
/// between the text content and the entity's children.
///
/// # Parameters
/// - `x_pos`: x-position of where the next sprite should go.
/// - `commands`: A command buffer for spawning or despawning sprites to
///   synchronize with the text content.
/// - `sprite_context`: Context for managing the entity and its sprite data.
/// - `font_assets`: Font-related assets and configuration.
/// - `sprite_layout`: Precomputed layout and scaling information.
/// - `sprite_text`: Component defining text appearance (e.g., color).
#[inline]
fn adjust_sprite_count(
    x_pos: f32,
    commands: &mut Commands,
    sprite_context: &mut SpriteContext<impl AsRef<str>>,
    font_assets: &FontAssets,
    sprite_layout: &SpriteLayout,
    sprite_text: &ImageFontSpriteText,
) {
    use std::cmp::Ordering;

    let char_count = sprite_context.text.filtered_chars().count();
    let sprite_count = sprite_context.image_font_text_data.sprites.len();

    match sprite_count.cmp(&char_count) {
        Ordering::Greater => {
            remove_excess_sprites(commands, sprite_context, char_count);
        }
        Ordering::Less => {
            add_missing_sprites(
                x_pos,
                commands,
                sprite_context,
                font_assets,
                sprite_layout,
                sprite_text,
            );
        }
        Ordering::Equal => {}
    }
}

/// Removes excess sprites from the text entity to match the new character
/// count.
///
/// # Parameters
/// - `commands`: Command buffer for despawning entities.
/// - `sprite_context`: Context for managing the entity and its sprite data.
/// - `char_count`: The number of characters in the filtered text.
///
/// # Side Effects
/// Excess sprites are despawned from the ECS.
#[inline]
fn remove_excess_sprites(
    commands: &mut Commands,
    sprite_context: &mut SpriteContext<impl AsRef<str>>,
    char_count: usize,
) {
    for entity in sprite_context
        .image_font_text_data
        .sprites
        .drain(char_count..)
    {
        commands.entity(entity).despawn();
    }
}

/// Adds missing sprites to the text entity to match the new character count.
///
/// If the number of sprites is less than the number of characters in the text,
/// this function spawns new sprites for the remaining characters and updates
/// the sprite data accordingly.
///
/// # Parameters
/// - `x_pos`: x-position of where the next sprite should go.
/// - `sprite_context`: Context for managing the entity and its sprite data.
/// - `font_assets`: Font-related assets and configuration.
/// - `sprite_layout`: Precomputed layout and scaling information.
/// - `sprite_text`: Component defining text appearance (e.g., color).
///
/// # Side Effects
/// New sprites are spawned as children of the entity, and the sprite data is
/// updated.
fn add_missing_sprites(
    mut x_pos: f32,
    commands: &mut Commands,
    sprite_context: &mut SpriteContext<impl AsRef<str>>,
    font_assets: &FontAssets,
    sprite_layout: &SpriteLayout,
    sprite_text: &ImageFontSpriteText,
) {
    let SpriteLayout {
        max_height,
        total_width,
        scale,
        anchors:
            Anchors {
                individual: anchor_vec_individual,
                whole: anchor_vec_whole,
            },
    } = *sprite_layout;

    let FontAssets {
        layout,
        image_font,
        image_font_text,
    } = *font_assets;

    let SpriteContext {
        entity,
        ref mut image_font_text_data,
        text,
    } = *sprite_context;

    let current_sprite_count = image_font_text_data.sprites.len();

    commands.entity(entity).with_children(|parent| {
        for character in text.filtered_chars().skip(current_sprite_count) {
            let rect = layout.textures[image_font.atlas_character_map[&character]];
            let (width, _height) = compute_dimensions(
                rect,
                image_font_text.font_height,
                max_height,
                sprite_text.letter_spacing,
                sprite_text.scaling_mode,
            );

            let transform = compute_transform(
                x_pos,
                total_width,
                width,
                max_height,
                scale,
                anchor_vec_whole,
                anchor_vec_individual,
            );

            debug!("Rendering {character} @ {x_pos} with size {width}");

            x_pos += width;

            let sprite = Sprite {
                image: image_font.texture.clone_weak(),
                texture_atlas: Some(TextureAtlas {
                    layout: image_font.atlas_layout.clone_weak(),
                    index: image_font.atlas_character_map[&character],
                }),
                color: sprite_text.color,
                ..Default::default()
            };

            let child = parent.spawn((sprite, transform));
            image_font_text_data.sprites.push(child.id());

            #[cfg(feature = "gizmos")]
            #[expect(
                clippy::used_underscore_binding,
                reason = "we're using an underscore binding here because it's unused when the \
                `gizmos` feature is not enabled."
            )]
            {
                let mut child = child;
                child.insert(ImageFontGizmoData {
                    width,
                    height: _height,
                });
            }
        }
    });
}

/// Stores precomputed layout and scaling information for rendering text
/// sprites.
///
/// Includes the maximum glyph height, total text width, scaling factor, and
/// anchor offsets for aligning individual glyphs and the entire text block.
struct SpriteLayout {
    /// Maximum glyph height in the text.
    max_height: u32,
    /// Total width of the text.
    total_width: f32,
    /// Scaling factor applied to glyph dimensions.
    scale: Vec3,
    /// Precomputed anchor offsets for alignment.
    anchors: Anchors,
}

/// Represents anchor-related offsets for text alignment and glyph positioning.
struct Anchors {
    /// Offset for aligning the entire text block.
    whole: Vec2,
    /// Offset for aligning individual glyphs.
    individual: Vec2,
}

/// Groups font-related assets and configuration for rendering text sprites.
///
/// Includes references to the texture atlas layout, font asset, and the
/// font text component that defines the text content and font height.
struct FontAssets<'assets> {
    /// The texture atlas layout defining glyph placements.
    layout: &'assets TextureAtlasLayout,
    /// The font asset containing glyph metadata.
    image_font: &'assets ImageFont,
    /// The text component defining the content and font height.
    image_font_text: &'assets ImageFontText,
}

/// Represents the entity and its associated text sprites during rendering.
///
/// Manages the commands for modifying the entity, its sprite data, and the
/// filtered text to ensure the sprites match the text content.
struct SpriteContext<'data, S: AsRef<str>> {
    /// The entity associated with the text sprites.
    entity: Entity,
    /// The mutable text sprite data component for the entity.
    image_font_text_data: &'data mut ImageFontTextData,
    /// The filtered text to be rendered as sprites.
    text: &'data filtered_string::FilteredString<'data, S>,
}

/// Renders gizmos for debugging `ImageFontText` and its associated glyphs in
/// the scene.
///
/// This function draws 2D rectangles and crosshairs to visualize the bounding
/// boxes and positions of rendered glyphs, aiding in debugging and alignment.
///
/// ### Gizmo Details
/// - Each child glyph is visualized as a purple rectangle using its dimensions
///   and position.
/// - The `ImageFontText` position is marked with a red cross for easier
///   identification.
///
/// ### Notes
/// This function is enabled only when the `gizmos` feature is active and
/// leverages the Bevy gizmo system for runtime visualization.
#[cfg(feature = "gizmos")]
pub fn render_sprite_gizmos(
    mut gizmos: Gizmos,
    query: Query<(&GlobalTransform, &Children), With<ImageFontText>>,
    child_query: Query<(&GlobalTransform, &ImageFontGizmoData), Without<ImageFontText>>,
) {
    use bevy::color::palettes::css;

    for (global_transform, children) in &query {
        for &child in children {
            if let Ok((child_global_transform, image_font_gizmo_data)) = child_query.get(child) {
                gizmos.rect_2d(
                    Isometry2d::from_translation(child_global_transform.translation().truncate()),
                    Vec2::new(image_font_gizmo_data.width, image_font_gizmo_data.height),
                    css::PURPLE,
                );
                gizmos.cross_2d(
                    Isometry2d::from_translation(child_global_transform.translation().truncate()),
                    5.,
                    css::GREEN,
                );
            }
        }

        gizmos.cross_2d(
            Isometry2d::from_translation(global_transform.translation().truncate()),
            10.,
            css::RED,
        );
    }
}
