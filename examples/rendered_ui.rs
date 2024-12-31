//! Shows use of the plugin with `bevy_ui`.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy_image_font::{ImageFont, ImageFontPlugin, ImageFontPreRenderedUiText, ImageFontText};

#[derive(Default, Debug, Resource)]
struct VowsJudged(u32);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ImageFontPlugin))
        .init_collection::<DemoAssets>()
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .init_resource::<VowsJudged>()
        .add_systems(Startup, spawn_ui)
        .add_systems(
            Update,
            (
                judge.run_if(input_just_pressed(KeyCode::Space)),
                update_vows_node,
            )
                .chain(),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct DemoAssets {
    #[asset(path = "example_font.image_font.ron")]
    image_font: Handle<ImageFont>,
}

#[derive(Component)]
struct VowsNode;

fn spawn_ui(mut commands: Commands, assets: Res<DemoAssets>) {
    commands.spawn(Camera2d);

    // root node
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                ImageNode::default(),
                Node {
                    position_type: PositionType::Relative,
                    ..default()
                },
                ImageFontPreRenderedUiText,
                ImageFontText::default()
                    .text("Press SPACE to judge!")
                    .font(assets.image_font.clone())
                    .font_height(72.0),
            ));
        });

    commands.spawn((
        VowsNode,
        ImageNode::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Auto,
            right: Val::Px(0.0),
            ..default()
        },
        ImageFontText::default()
            .font(assets.image_font.clone())
            .font_height(72.0),
    ));
}

fn judge(mut vows: ResMut<VowsJudged>) {
    vows.0 += 1;
}

fn update_vows_node(vows: Res<VowsJudged>, mut node: Query<&mut ImageFontText, With<VowsNode>>) {
    if vows.is_changed() {
        node.single_mut().text = format!("Vows judged: {}", vows.0);
    }
}
