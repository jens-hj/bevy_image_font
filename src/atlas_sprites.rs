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
    pub anchor: Anchor,
    pub color: Color,
}

#[cfg(feature = "gizmos")]
#[derive(Debug, Clone, Default, Component)]
pub struct ImageFontGizmoData {
    width: u32,
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
pub fn set_up_sprites(
    mut commands: Commands,
    mut query: Query<(Entity, &ImageFontText, &ImageFontSpriteText), Changed<ImageFontText>>,
    image_fonts: Res<Assets<ImageFont>>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for (entity, image_font_text, image_font_sprite_text) in &mut query {
        // Remove existing sprites
        let mut entity_commands = commands.entity(entity);
        entity_commands.despawn_descendants();

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

        if text.is_empty() {
            // nothing to render
            continue;
        }

        let max_height = text
            .chars()
            .map(|c| layout.textures[image_font.atlas_character_map[&c]].height())
            .reduce(u32::max)
            .unwrap();
        let total_width = text
            .chars()
            .map(|c| layout.textures[image_font.atlas_character_map[&c]].width())
            .reduce(|a, b| a + b)
            .unwrap();

        let scale: Vec3 = (
            Vec2::splat(
                image_font_text.font_height.unwrap_or(max_height as f32) / max_height as f32,
            ),
            0.0,
        )
            .into();

        entity_commands.with_children(|parent| {
            let mut x = 0;
            for c in text.chars() {
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

                let anchor_vec = image_font_sprite_text.anchor.as_vec();
                let anchor_vec_individual = -anchor_vec;
                let anchor_vec_whole = -(anchor_vec + Vec2::new(0.5, 0.0));
                let transform = Transform::from_translation(Vec3::new(
                    x as f32
                        + total_width as f32 * anchor_vec_whole.x * scale.x
                        + width as f32 * anchor_vec_individual.x,
                    max_height as f32 * anchor_vec_whole.y * scale.y,
                    0.,
                ))
                .with_scale(scale);
                let _child = parent.spawn((
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

                #[cfg(feature = "gizmos")]
                #[allow(clippy::used_underscore_binding)]
                {
                    let mut child = _child;
                    child.insert(ImageFontGizmoData {
                        width,
                        height: _height,
                    });
                }

                x += width;
            }
        });
    }
}

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
