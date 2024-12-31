//! Demonstrates showing a texture atlas-based image font text at both its
//! 'native' height and a scaled-up height, also demonstrating use of its
//! additional `color` value.

use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy_image_font::{ImageFont, ImageFontPlugin, ImageFontSpriteText, ImageFontText};

use crate::common::{FONT_WIDTH, TEXT};

mod common;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            ImageFontPlugin,
        ))
        .init_collection::<DemoAssets>()
        .add_systems(Startup, spawn_text)
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct DemoAssets {
    #[asset(path = "example_font.image_font.ron")]
    image_font: Handle<ImageFont>,
}

fn spawn_text(mut commands: Commands, assets: Res<DemoAssets>) {
    commands.spawn(Camera2d);

    commands.spawn((
        ImageFontSpriteText::default().color(tailwind::PINK_500),
        ImageFontText::default()
            .text(TEXT)
            .font(assets.image_font.clone())
            .font_height(36.0),
        Transform::from_translation(Vec3::new(0., 0., 0.)),
    ));
    commands.spawn((
        // This demonstrates using the `anchor` field on the `ImageFontSpriteText`.
        // To still center it horizontally with the text above, we shift it left by half its width.
        ImageFontSpriteText::default()
            .color(tailwind::EMERALD_500)
            .anchor(Anchor::CenterLeft),
        ImageFontText::default()
            .text(TEXT)
            .font(assets.image_font.clone()),
        #[allow(clippy::cast_precision_loss)]
        Transform::from_translation(Vec3::new(
            -((TEXT.chars().count() * FONT_WIDTH / 2) as f32),
            40.,
            0.,
        )),
    ));
}
