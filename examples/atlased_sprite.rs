//! Demonstrates showing a texture atlas-based image font text at both its
//! 'native' height and a scaled-up height, also demonstrating use of its
//! additional `color` value.

use std::time::Duration;

use bevy::color::palettes::tailwind;
use bevy::color::ColorCurve;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy_image_font::{ImageFont, ImageFontPlugin, ImageFontSpriteText, ImageFontText};

use crate::common::{FONT_WIDTH, RAINBOW, TEXT};

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

#[derive(AssetCollection, Resource)]
struct DemoAssets {
    #[asset(path = "example_font.image_font.ron")]
    image_font: Handle<ImageFont>,
}

#[derive(Component)]
struct AnimateText(&'static str, usize);

#[derive(Component)]
struct AnimateColor(ColorCurve<Srgba>);

fn spawn_text(mut commands: Commands, assets: Res<DemoAssets>) {
    commands.spawn(Camera2d);

    commands.spawn((
        AnimateColor(ColorCurve::new(RAINBOW).unwrap()),
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
        #[allow(clippy::cast_precision_loss)]
        Transform::from_translation(Vec3::new(
            -((TEXT.chars().count() * FONT_WIDTH / 2) as f32),
            40.,
            0.,
        )),
    ));
}

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

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
fn animate_color(mut query: Query<(&AnimateColor, &mut ImageFontSpriteText)>, time: Res<Time>) {
    for (animate_color, mut image_sprite_font_text) in &mut query {
        let animation_progress = time.elapsed_secs() / RAINBOW.len() as f32;
        let len = (RAINBOW.len() - 1) as f32;
        if animation_progress.trunc() as u32 % 2 == 0 {
            image_sprite_font_text.color = animate_color
                .0
                .sample(animation_progress.fract() * len)
                .unwrap()
                .into();
        } else {
            image_sprite_font_text.color = animate_color
                .0
                .sample(len - (animation_progress.fract() * len))
                .unwrap()
                .into();
        }
    }
}
