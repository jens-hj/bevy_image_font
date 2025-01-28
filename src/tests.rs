#![allow(clippy::unwrap_used, reason = "test code panics to indicate errors")]

use std::any::TypeId;
use std::path::PathBuf;

use bevy::log::LogPlugin;
use bevy_image::{CompressedImageFormats, ImageLoader};

use super::*;

mod sync_texts_with_font_changes;

#[test]
fn mapped_atlas_layout_from_char_map_creates_correct_character_map_and_layout() {
    let size = UVec2::new(256, 256);
    let mut char_rect_map = HashMap::new();
    char_rect_map.insert('A', URect::new(0, 0, 16, 16));
    char_rect_map.insert('B', URect::new(16, 0, 32, 16));

    let (atlas_character_map, atlas_layout) =
        ImageFont::mapped_atlas_layout_from_char_map(size, &char_rect_map);

    assert_eq!(atlas_character_map.len(), 2);
    assert!(atlas_character_map.contains_key(&'A'));
    assert!(atlas_character_map.contains_key(&'B'));
    assert_eq!(atlas_layout.textures.len(), 2);
    assert_eq!(
        atlas_layout.textures[atlas_character_map[&'A']],
        char_rect_map[&'A']
    );
    assert_eq!(
        atlas_layout.textures[atlas_character_map[&'B']],
        char_rect_map[&'B']
    );
}

#[test]
#[cfg_attr(
    all(
        feature = "gizmos",
        not(feature = "DO_NOT_USE_internal_tests_disable_gizmos")
    ),
    ignore
)]
fn image_font_plugin_initialization() {
    let mut app = App::new();

    app.add_plugins((MinimalPlugins, AssetPlugin::default(), LogPlugin::default()));
    app.add_plugins(ImageFontPlugin);

    app.register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE))
        .init_asset::<TextureAtlasLayout>()
        .init_asset::<Image>();

    // Verify that `ImageFont` is registered as an asset by attempting to load one
    let asset_server = app.world().resource::<AssetServer>();
    let font_path = PathBuf::from("example_font.image_font.ron");

    let handle: Handle<ImageFont> = asset_server.load(font_path.clone());
    let load_state = asset_server.get_load_state(handle.id());
    assert!(
        load_state.is_some(),
        "The `ImageFontPlugin` should allow loading `.image_font.ron` files; load state was \
        {load_state:?}, expected Some(_)."
    );

    // Verify that `ImageFont` and related types are registered with the reflection
    // system
    {
        let type_registry = app.world().resource::<AppTypeRegistry>().read();
        assert!(
            type_registry.contains(TypeId::of::<ImageFont>()),
            "The `ImageFontPlugin` should register `ImageFont` with the reflection system."
        );
        assert!(
            type_registry.contains(TypeId::of::<ImageFontText>()),
            "The `ImageFontPlugin` should register `ImageFontText` with the reflection system."
        );
    }

    // Verify that the app updates without errors (systems from the plugin are
    // functional)
    app.update();
}

// This is mostly here for the sake of coverage.
#[test]
fn creating_image_font_works() {
    ImageFont::from_mapped_atlas_layout(default(), default(), default(), default());
}
