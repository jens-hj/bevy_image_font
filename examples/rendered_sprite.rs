//! Demonstrates rendering image font text at both its 'native' height and a
//! scaled-up height.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy_image_font::{ImageFont, ImageFontPlugin, ImageFontPreRenderedText, ImageFontText};

use crate::common::{FONT_WIDTH, TEXT};

mod common;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ImageFontPlugin))
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
        ImageFontPreRenderedText,
        ImageFontText::default()
            .text(TEXT)
            .font(assets.image_font.clone())
            .font_height(36.0),
        // Our font is 45 pixels wide per character, and with an odd number of characters, the
        // text aligns to the middle of a pixel, causing imperfect rendering. Shifting the
        // position by 0.5 pixels ensures alignment to pixel boundaries, resulting in crisp,
        // pixel-perfect rendering.
        Transform::from_translation(Vec3::new(0.5, 0., 0.)),
    ));
    commands.spawn((
        ImageFontPreRenderedText,
        ImageFontText::default()
            .text(TEXT)
            .font(assets.image_font.clone()),
        // Instead of shifting the character by 0.5 pixels when the text lands in the middle of
        // a pixel, we can anchor the sprite to an edge and move it by a whole number of pixels.
        // To center it with the text above, we shift it left by half its width.
        Sprite {
            anchor: Anchor::CenterLeft,
            ..default()
        },
        #[allow(clippy::cast_precision_loss)]
        Transform::from_translation(Vec3::new(
            -((TEXT.chars().count() * FONT_WIDTH / 2) as f32),
            40.,
            0.,
        )),
    ));
}
