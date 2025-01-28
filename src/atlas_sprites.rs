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

use crate::atlas_sprites::anchors::AnchorOffsets;
use crate::atlas_sprites::render_context::RenderContext;
use crate::{sync_texts_with_font_changes, ImageFont, ImageFontSet, ImageFontText};

mod anchors;
mod letter_spacing;
mod render_context;
mod scaling_mode;

pub use letter_spacing::*;
pub use scaling_mode::*;

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

        #[cfg(all(
            feature = "gizmos",
            not(feature = "DO_NOT_USE_internal_tests_disable_gizmos")
        ))]
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

    /// Determines a constant kerning between characters. The spacing is given
    /// at the font's native height and is scaled proportionally based on the
    /// current font height.
    pub letter_spacing: LetterSpacing,
}

/// Basically a map between character index and character sprite
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

        let Some(render_context) = RenderContext::new(
            &image_fonts,
            image_font_text,
            image_font_sprite_text,
            &texture_atlas_layouts,
        ) else {
            continue;
        };

        let mut sprite_context = SpriteContext {
            entity,
            image_font_text_data,
        };

        let x_pos = update_existing_sprites(&mut child_query, &mut sprite_context, &render_context);

        adjust_sprite_count(
            x_pos,
            &mut commands,
            &mut sprite_context,
            &render_context,
            image_font_sprite_text,
        );

        if let Some(new_image_font_text_data) = maybe_new_image_font_text_data {
            debug!("Inserted new ImageFontTextData for entity {:?}", entity);
            commands.entity(entity).insert(new_image_font_text_data);
        }
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
/// - `render_context`: Context providing rendering-related information and
///   operations.
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
    sprite_context: &mut SpriteContext,
    render_context: &RenderContext,
) -> f32 {
    let SpriteContext {
        ref mut image_font_text_data,
        ..
    } = *sprite_context;

    let mut x_pos = 0.;

    for (sprite_entity, character) in image_font_text_data
        .sprites
        .iter()
        .copied()
        .zip(render_context.text().filtered_chars())
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

        let sprite = &mut *sprite;
        let Some(sprite_texture) = sprite.texture_atlas.as_mut() else {
            error!(
                "An ImageFontSpriteText's child sprite was \
            unexpectedly missing a `texture_atlas`. This will likely cause rendering bugs."
            );
            continue;
        };

        render_context.update_sprite_values(character, sprite_texture, &mut sprite.color);

        *transform = render_context.transform(&mut x_pos, character);

        #[cfg(feature = "gizmos")]
        {
            let (width, height) = render_context.character_dimensions(character);
            gizmo_data.width = width;
            gizmo_data.height = height;
        }
    }

    x_pos
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
/// - `render_context`: Context providing rendering-related information and
///   operations.
/// - `sprite_text`: Component defining text appearance (e.g., color).
#[inline]
fn adjust_sprite_count(
    x_pos: f32,
    commands: &mut Commands,
    sprite_context: &mut SpriteContext,
    render_context: &RenderContext,
    sprite_text: &ImageFontSpriteText,
) {
    use std::cmp::Ordering;

    let char_count = render_context.text().filtered_chars().count();
    let sprite_count = sprite_context.image_font_text_data.sprites.len();

    match sprite_count.cmp(&char_count) {
        Ordering::Greater => {
            remove_excess_sprites(commands, sprite_context, char_count);
        }
        Ordering::Less => {
            add_missing_sprites(x_pos, commands, sprite_context, render_context, sprite_text);
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
    sprite_context: &mut SpriteContext,
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
/// - `commands`: Command buffer for spawning new sprite entities.
/// - `sprite_context`: Context for managing the entity and its sprite data.
/// - `render_context`: Context providing rendering-related information and
///   operations.
/// - `sprite_text`: Component defining text appearance (e.g., color).
fn add_missing_sprites(
    mut x_pos: f32,
    commands: &mut Commands,
    sprite_context: &mut SpriteContext,
    render_context: &RenderContext,
    sprite_text: &ImageFontSpriteText,
) {
    let SpriteContext {
        entity,
        ref mut image_font_text_data,
    } = *sprite_context;

    let current_sprite_count = image_font_text_data.sprites.len();

    commands.entity(entity).with_children(|parent| {
        for character in render_context
            .text()
            .filtered_chars()
            .skip(current_sprite_count)
        {
            let transform = render_context.transform(&mut x_pos, character);
            let sprite = Sprite {
                image: render_context.font_image(),
                texture_atlas: Some(render_context.font_texture_atlas(character)),
                color: sprite_text.color,
                ..Default::default()
            };

            let child = parent.spawn((sprite, transform));
            image_font_text_data.sprites.push(child.id());

            #[cfg(feature = "gizmos")]
            {
                let (width, height) = render_context.character_dimensions(character);
                let mut child = child;
                child.insert(ImageFontGizmoData { width, height });
            }
        }
    });
}

/// Represents the entity and its associated text sprites during rendering.
///
/// Manages the commands for modifying the entity, its sprite data, and the
/// filtered text to ensure the sprites match the text content.
struct SpriteContext<'data> {
    /// The entity associated with the text sprites.
    entity: Entity,
    /// The mutable text sprite data component for the entity.
    image_font_text_data: &'data mut ImageFontTextData,
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
#[cfg(all(
    feature = "gizmos",
    not(feature = "DO_NOT_USE_internal_tests_disable_gizmos")
))]
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
