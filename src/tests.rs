#![allow(clippy::unwrap_used, reason = "test code panics to indicate errors")]
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
