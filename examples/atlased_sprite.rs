//! Demonstrates showing a texture atlas-based image font text at both its
//! 'native' height and a scaled-up height, also demonstrating use of its
//! additional `color` value.

#![expect(
    clippy::mod_module_files,
    reason = "if present as common.rs, cargo thinks it's an example binary"
)]
#![expect(clippy::expect_used, reason = "only used when panics can't happen")]

use std::time::Duration;

use bevy::color::palettes::tailwind;
use bevy::color::ColorCurve;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_asset_loader::prelude::AssetCollectionApp as _;
use bevy_image_font::atlas_sprites::ImageFontSpriteText;
use bevy_image_font::{ImageFontPlugin, ImageFontText, LetterSpacing};

use crate::common::{DemoAssets, FONT_WIDTH, RAINBOW, TEXT};

mod common;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            ImageFontPlugin,
        ))
        .init_collection::<DemoAssets>()
        .add_systems(Startup, spawn_text)
        .add_systems(Update, (animate_text, animate_color))
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .run();
}

/// A component for animating text content.
///
/// The `&'static str` field represents the text to animate, while the `usize`
/// field tracks the current animation state, i.e. how many characters of the
/// text is currenly being displayed.
#[derive(Component)]
struct AnimateText(&'static str, usize);

/// A component for animating the color of text.
///
/// The `ColorCurve` field defines the color gradient used for the animation.
#[derive(Component)]
struct AnimateColor(ColorCurve<Srgba>);

/// Spawns the text entities for the example.
///
/// This system creates two text entities:
/// 1. A text entity rendered at a scaled height with animated colors.
/// 2. A text entity rendered at its native height with animated content.
fn spawn_text(mut commands: Commands, assets: Res<DemoAssets>) {
    commands.spawn(Camera2d);

    commands.spawn((
        AnimateColor(ColorCurve::new(RAINBOW).expect("RAINBOW contains at least two colors")),
        ImageFontSpriteText::default(),
        ImageFontText::default()
            .text(TEXT)
            .font(assets.image_font.clone())
            .font_height(36.0),
        Transform::from_translation(Vec3::new(0., 0., 0.)),
    ));
    commands.spawn((
        AnimateText(TEXT, 0),
        // This demonstrates using the `anchor` field on the `ImageFontSpriteText`.
        // To still center it horizontally with the text above, we shift it left by half its width.
        ImageFontSpriteText::default()
            .color(tailwind::AMBER_500)
            .anchor(Anchor::CenterLeft),
        ImageFontText::default().font(assets.image_font.clone()),
        #[expect(
            clippy::cast_precision_loss,
            reason = "the magnitude of the numbers we're working on here are too small to lose \
            anything"
        )]
        Transform::from_translation(Vec3::new(
            -((TEXT.chars().count() * FONT_WIDTH / 2) as f32),
            40.,
            0.,
        )),
    ));

    commands.spawn((
        ImageFontSpriteText::default()
            .color(tailwind::ZINC_500)
            .letter_spacing(LetterSpacing::Pixel(2)),
        ImageFontText::default()
            .text(TEXT)
            .font(assets.image_font.clone())
            .font_height(36.0),
        Transform::from_translation(Vec3::new(0.5, -40., 0.)),
    ));
}

/// Animates the text content of entities with the `AnimateText` component.
///
/// This system modifies the `ImageFontText` component to display an animated
/// sequence of characters, cycling through the text content over time.
fn animate_text(
    mut query: Query<(&mut AnimateText, &mut ImageFontText)>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration().is_zero() || timer.mode() == TimerMode::Once {
        timer.set_duration(Duration::from_secs_f32(0.1));
        timer.set_mode(TimerMode::Repeating);
    }

    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }

    for (mut animated_text, mut image_font_text) in &mut query {
        let char_count = animated_text.0.chars().count();

        animated_text.1 += 1;
        if animated_text.1 > char_count * 2 {
            animated_text.1 = 0;
        }

        let show_count = if animated_text.1 > char_count - 1 {
            char_count - (animated_text.1 - char_count)
        } else {
            animated_text.1
        };
        image_font_text.text = animated_text.0.chars().take(show_count).collect::<String>();
    }
}

/// Animates the color of text entities with the `AnimateColor` component.
///
/// This system modifies the `color` field of the `ImageFontSpriteText`
/// component to cycle through the colors defined in the `RAINBOW` palette.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "the magnitude of the numbers we're working on here are too small to lose anything"
)]
fn animate_color(mut query: Query<(&AnimateColor, &mut ImageFontSpriteText)>, time: Res<Time>) {
    for (animate_color, mut image_sprite_font_text) in &mut query {
        let animation_progress = time.elapsed_secs() / RAINBOW.len() as f32;
        let len = (RAINBOW.len() - 1) as f32;
        if animation_progress.trunc() as u32 % 2 == 0 {
            image_sprite_font_text.color = animate_color
                .0
                .sample(animation_progress.fract() * len)
                .expect("fract() is [0,1) and `len` will always be within bounds")
                .into();
        } else {
            image_sprite_font_text.color = animate_color
                .0
                .sample(len - (animation_progress.fract() * len))
                .expect("fract() is [0,1) and `len` will always be within bounds")
                .into();
        }
    }
}
