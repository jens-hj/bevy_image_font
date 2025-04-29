use std::marker::PhantomData;

use bevy::asset::AssetIndex;
use bevy::ecs::system::SystemState;

use super::*;

#[test]
fn added_and_changed_when_added() {
    let (mut app, mut system_state, _) = setup_app_system_state_and_entity();

    // Verify the initial change state of the component: both `is_added` and
    // `is_changed` should be true.
    with_image_font_text(&mut app, &mut system_state, |image_font_text| {
        assert!(image_font_text.is_added());
        assert!(image_font_text.is_changed());
    });
}

#[test]
fn unchanged_after_initial_update() {
    let (mut app, mut system_state, _) = setup_app_system_state_and_entity();

    clear_query_state(&mut app, &mut system_state);

    app.update();

    // Verify the change state of the component: both `is_added` and `is_changed`
    // should be false after `app.update()`.
    with_image_font_text(&mut app, &mut system_state, |image_font_text| {
        assert!(!image_font_text.is_added());
        assert!(!image_font_text.is_changed());
    });
}

#[test]
fn changed_after_modified_event() {
    let (mut app, mut system_state, font_handle) = setup_app_system_state_and_entity();

    clear_query_state(&mut app, &mut system_state);

    app.update();

    app.world_mut().send_event(AssetEvent::Modified {
        id: font_handle.id(),
    });

    app.update();

    // Verify the change state of the component: `is_changed` should be true after
    // `app.update()` with `AssetEvent::Modified` event.
    with_image_font_text(&mut app, &mut system_state, |image_font_text| {
        assert!(!image_font_text.is_added());
        assert!(image_font_text.is_changed());
    });
}

#[test]
fn changed_after_loaded_with_dependencies_event() {
    let (mut app, mut system_state, font_handle) = setup_app_system_state_and_entity();

    clear_query_state(&mut app, &mut system_state);

    app.update();

    app.world_mut()
        .send_event(AssetEvent::LoadedWithDependencies {
            id: font_handle.id(),
        });

    app.update();

    // Verify the change state of the component: `is_changed` should be true after
    // `app.update()` with `AssetEvent::LoadedWithDependencies` event.
    with_image_font_text(&mut app, &mut system_state, |image_font_text| {
        assert!(!image_font_text.is_added());
        assert!(image_font_text.is_changed());
    });
}

#[test]
fn not_changed_after_events_on_other_fonts() {
    let (mut app, mut system_state, _) = setup_app_system_state_and_entity();

    clear_query_state(&mut app, &mut system_state);

    app.update();

    let unrelated_font_id: AssetId<ImageFont> = AssetId::Index {
        index: AssetIndex::from_bits(42),
        marker: PhantomData,
    };
    app.world_mut().send_event(AssetEvent::Modified {
        id: unrelated_font_id,
    });

    app.update();

    // Verify the change state of the component: `is_changed` should be false after
    // `app.update()` with `AssetEvent::Modified` event on unrelated `ImageFont`.
    with_image_font_text(&mut app, &mut system_state, |image_font_text| {
        assert!(!image_font_text.is_added());
        assert!(!image_font_text.is_changed());
    });
}

#[test]
fn not_changed_on_irrelevant_events() {
    let (mut app, mut system_state, font_handle) = setup_app_system_state_and_entity();

    clear_query_state(&mut app, &mut system_state);

    app.update();

    app.world_mut().send_event(AssetEvent::Added {
        id: font_handle.id(),
    });

    app.world_mut().send_event(AssetEvent::Removed {
        id: font_handle.id(),
    });

    app.world_mut().send_event(AssetEvent::Unused {
        id: font_handle.id(),
    });

    app.update();

    // Verify the change state of the component: `is_changed` should be false after
    // `app.update()` with `AssetEvent`s other than `Modified` or
    // `LoadedWithDependencies`.
    with_image_font_text(&mut app, &mut system_state, |image_font_text| {
        assert!(!image_font_text.is_added());
        assert!(!image_font_text.is_changed());
    });
}

/// Helper function to set up the app, set up the `SystemState` we use for
/// validating change and spawn an `ImageFontText` entity.
fn setup_app_system_state_and_entity() -> (
    App,
    SystemState<Query<'static, 'static, Ref<'static, ImageFontText>>>,
    Handle<ImageFont>,
) {
    let mut app = App::new();
    app.add_event::<AssetEvent<ImageFont>>();
    app.add_systems(Update, sync_texts_with_font_changes);

    let font_handle = Handle::default();
    app.world_mut().spawn(ImageFontText {
        text: String::from("Hello"),
        font: font_handle.clone(),
        font_height: Some(36.0),
    });

    let system_state: SystemState<Query<Ref<ImageFontText>>> = SystemState::new(app.world_mut());

    (app, system_state, font_handle)
}

/// Helper function to run code on the `Ref<ImageFontText>`.
fn with_image_font_text(
    app: &mut App,
    system_state: &mut SystemState<Query<'static, 'static, Ref<'static, ImageFontText>>>,
    with_func: impl FnOnce(Ref<'_, ImageFontText>),
) {
    let query = system_state.get(app.world());
    let image_font_text = query
        .single()
        .expect("Entity with ImageFontText should exist");
    with_func(image_font_text);
}

fn clear_query_state(
    app: &mut App,
    system_state: &mut SystemState<Query<'static, 'static, Ref<'static, ImageFontText>>>,
) {
    with_image_font_text(app, system_state, |_| {});
}
