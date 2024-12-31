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

use crate::{mark_changed_fonts_as_dirty, ImageFont, ImageFontSet, ImageFontText};

#[derive(Default)]
pub(crate) struct AtlasSpritesPlugin;

impl Plugin for AtlasSpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            set_up_sprites
                .after(mark_changed_fonts_as_dirty)
                .in_set(ImageFontSet),
        );

        #[cfg(feature = "gizmos")]
        {
            app.add_systems(Update, render_sprite_gizmos);
        }
    }
}

/// Text rendered using an [`ImageFont`] as individual sprites.
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
    width: u32,

    /// The height of the gizmo, representing the rendered font's bounding box
    /// or visualized area in the scene.
    height: u32,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
/// System that renders each [`ImageFontText`] as child [`Sprite`] entities
/// where each sprite represents a character in the text. That is to say, each
/// sprite gets positioned accordingly to its position in the text. This
/// system only runs when the `ImageFontText` or [`ImageFontSpriteText`]
/// changes.
#[allow(clippy::missing_panics_doc)] // Panics should be impossible
#[allow(private_interfaces)]
#[allow(clippy::too_many_lines)] // TODO: Only temporarily!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
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
    mut child_query: Query<(&mut Sprite, &mut Transform)>,
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
            maybe_new_image_font_text_data.as_mut().unwrap()
        };

        let Some(image_font) = image_fonts.get(&image_font_text.font) else {
            error!(
                "Error when setting up image font text {:?}: ImageFont asset not loaded",
                image_font_text
            );
            return;
        };
        let Some(layout) = texture_atlas_layouts.get(&image_font.atlas_layout) else {
            error!(
                "Error when setting up image font text {:?}: Font texture asset not loaded",
                image_font_text
            );
            return;
        };

        let text = image_font.filter_string(&image_font_text.text);

        let max_height = text
            .chars()
            .map(|c| layout.textures[image_font.atlas_character_map[&c]].height())
            .reduce(u32::max)
            .unwrap_or(1);
        let total_width = text
            .chars()
            .map(|c| layout.textures[image_font.atlas_character_map[&c]].width())
            .reduce(|a, b| a + b)
            .unwrap_or(0);

        let scale: Vec3 = (
            Vec2::splat(
                image_font_text.font_height.unwrap_or(max_height as f32) / max_height as f32,
            ),
            0.0,
        )
            .into();

        let anchor_vec = image_font_sprite_text.anchor.as_vec();
        let anchor_vec_individual = -anchor_vec;
        let anchor_vec_whole = -(anchor_vec + Vec2::new(0.5, 0.0));

        // First, let's set and move any existing sprites we've got
        let mut x_pos = 0;
        for (sprite_entity, c) in image_font_text_data
            .sprites
            .iter()
            .copied()
            .zip(text.chars())
        {
            let (mut sprite, mut transform) = child_query.get_mut(sprite_entity).unwrap();
            sprite.texture_atlas.as_mut().unwrap().index = image_font.atlas_character_map[&c];
            sprite.color = image_font_sprite_text.color;

            let rect = layout.textures[image_font.atlas_character_map[&c]];
            let (width, _height) =
                image_font_text
                    .font_height
                    .map_or((rect.width(), rect.height()), |fh| {
                        (
                            (rect.width() as f32 * fh / max_height as f32) as u32,
                            (rect.height() as f32 * fh / max_height as f32) as u32,
                        )
                    });

            *transform = Transform::from_translation(Vec3::new(
                x_pos as f32
                    + total_width as f32 * anchor_vec_whole.x * scale.x
                    + width as f32 * anchor_vec_individual.x,
                max_height as f32 * anchor_vec_whole.y * scale.y,
                0.,
            ))
            .with_scale(scale);

            x_pos += width;
        }

        // If it isn't an exact match, we have two potential cases that require addition
        // work: too many sprites or too few sprites. With too many, we remove
        // them, and with too few, we add them.
        let char_count = text.chars().count();
        let sprite_count = image_font_text_data.sprites.len();

        #[allow(clippy::comparison_chain)]
        if sprite_count == char_count {
            // Exact match, nothing to do
            trace!("Exact match, nothing to do");
        } else if sprite_count > char_count {
            // Too many sprites; remove excess
            debug!("Removing excess sprites; have {sprite_count}, only need {char_count}");
            for e in image_font_text_data.sprites.drain(char_count..) {
                trace!("Despawning {e}");
                commands.entity(e).despawn();
            }
        } else {
            // Too few sprites; add missing
            debug!("Adding missing sprites; have {sprite_count}, need {char_count}; e={entity}");

            let mut entity_commands = commands.entity(entity);
            entity_commands.with_children(|parent| {
                for c in text.chars().skip(sprite_count) {
                    let rect = layout.textures[image_font.atlas_character_map[&c]];
                    let (width, _height) =
                        image_font_text
                            .font_height
                            .map_or((rect.width(), rect.height()), |fh| {
                                (
                                    (rect.width() as f32 * fh / max_height as f32) as u32,
                                    (rect.height() as f32 * fh / max_height as f32) as u32,
                                )
                            });

                    let transform = Transform::from_translation(Vec3::new(
                        x_pos as f32
                            + total_width as f32 * anchor_vec_whole.x * scale.x
                            + width as f32 * anchor_vec_individual.x,
                        max_height as f32 * anchor_vec_whole.y * scale.y,
                        0.,
                    ))
                    .with_scale(scale);

                    x_pos += width;

                    let child = parent.spawn((
                        Sprite {
                            image: image_font.texture.clone_weak(),
                            texture_atlas: Some(TextureAtlas {
                                layout: image_font.atlas_layout.clone_weak(),
                                index: image_font.atlas_character_map[&c],
                            }),
                            color: image_font_sprite_text.color,
                            ..default()
                        },
                        transform,
                    ));
                    image_font_text_data.sprites.push(child.id());

                    #[cfg(feature = "gizmos")]
                    #[allow(clippy::used_underscore_binding)]
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

        if let Some(new_image_font_text_data) = maybe_new_image_font_text_data {
            commands.entity(entity).insert(new_image_font_text_data);
        }
    }
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
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
#[cfg(feature = "gizmos")]
pub fn render_sprite_gizmos(
    mut gizmos: Gizmos,
    query: Query<(&GlobalTransform, &Children), With<ImageFontText>>,
    child_query: Query<(&GlobalTransform, &ImageFontGizmoData), Without<ImageFontText>>,
) {
    for (global_transform, children) in &query {
        for &child in children {
            if let Ok((child_global_transform, image_font_gizmo_data)) = child_query.get(child) {
                gizmos.rect_2d(
                    Isometry2d::from_translation(child_global_transform.translation().truncate()),
                    Vec2::new(
                        image_font_gizmo_data.width as f32,
                        image_font_gizmo_data.height as f32,
                    ),
                    bevy::color::palettes::css::PURPLE,
                );
            }
        }

        gizmos.cross_2d(
            Isometry2d::from_translation(global_transform.translation().truncate()),
            10.,
            bevy::color::palettes::css::RED,
        );
    }
}
