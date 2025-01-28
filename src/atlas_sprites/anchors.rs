//! This module provides functionality for managing anchor-based positioning
//! and alignment in text rendering.
//!
//! It extends Bevy's [`Anchor`] type with additional methods and structures
//! for calculating offsets and transforms used to position and scale text
//! glyphs. This is achieved through the [`AnchorExt`] trait and the
//! [`AnchorOffsets`] struct, which encapsulate logic for handling:
//! - Whole alignment offsets: Used to position the entire text block.
//! - Individual alignment offsets: Used to align individual glyphs within the
//!   text block.
//!
//! # Key Components
//!
//! - [`AnchorExt`]: An extension trait for [`Anchor`] that calculates anchor
//!   offsets as an [`AnchorOffsets`] struct.
//! - [`AnchorOffsets`]: A structure representing precomputed offsets for whole
//!   and individual alignments, with methods for computing text glyph
//!   transforms.
//!
//! # Testing
//! The module includes comprehensive unit tests to ensure the correctness
//! of offsets and transform calculations. See the `tests` module for details.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Extension trait to provide additional functionality for `Anchor`.
pub(crate) trait AnchorExt {
    /// Computes the `Anchors` struct directly for the `Anchor` value.
    ///
    /// # Returns
    /// `Anchors` containing:
    /// - `whole`: Offset for aligning the entire text block.
    /// - `individual`: Offset for aligning each individual glyph.
    fn to_anchor_offsets(self) -> AnchorOffsets;
}

impl AnchorExt for Anchor {
    fn to_anchor_offsets(self) -> AnchorOffsets {
        let anchor_vec = self.as_vec();
        AnchorOffsets {
            whole: -(anchor_vec + Vec2::new(0.5, 0.0)),
            individual: Vec2::new(0.5, 0.0),
        }
    }
}

/// Represents anchor-related offsets for text alignment and glyph positioning.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AnchorOffsets {
    /// Offset for aligning the entire text block.
    pub whole: Vec2,
    /// Offset for aligning individual glyphs.
    pub individual: Vec2,
}

impl AnchorOffsets {
    /// Computes the transform for positioning and scaling a text sprite.
    ///
    /// This method calculates the translation and scaling for a sprite based
    /// on:
    /// - The `whole` offset: Used for aligning the entire text block.
    /// - The `individual` offset: Used for aligning each glyph relative to the
    ///   text block.
    ///
    /// Intermediate calculations use `Vec2`, as the `z` component is
    /// unnecessary until constructing the final `Transform`.
    ///
    /// # Parameters
    /// - `x_pos`: Current x-position of the sprite.
    /// - `text_width`: Total width of the text block.
    /// - `width`: Width of the current glyph.
    /// - `max_height`: Maximum height of the text block.
    /// - `scale`: Scaling factor for glyph dimensions.
    /// - `whole_anchor`: Offset for aligning the entire text block.
    /// - `individual_anchor`: Offset for aligning the individual glyph.
    ///
    /// # Returns
    /// A `Transform` object representing the sprite's position and scale.
    #[inline]
    pub(crate) fn compute_transform(
        self,
        x_pos: f32,
        text_width: f32,
        width: f32,
        max_height: u32,
        scale: f32,
    ) -> Transform {
        // Step 1: Start with the base x_pos translation
        let mut translation = base_translation(x_pos);

        // Step 2: Apply the whole offset contribution
        apply_whole_offset(&mut translation, text_width, max_height, scale, self.whole);

        // Step 3: Apply the individual offset contribution
        apply_individual_offset(&mut translation, width, self.individual);

        // Step 4: Finalize the transform
        finalize_transform(translation, scale)
    }
}

/// Creates the initial base translation for a sprite.
///
/// # Parameters
/// - `x_pos`: The x-position of the sprite.
///
/// # Returns
/// A `Vec2` representing the base translation with only the x-component set.
fn base_translation(x_pos: f32) -> Vec2 {
    Vec2::new(x_pos, 0.0)
}

/// Adjusts the translation based on the whole anchor offset.
///
/// # Parameters
/// - `translation`: A mutable reference to the translation vector to modify.
/// - `text_width`: Total width of the text block.
/// - `max_height`: Maximum height of the text block.
/// - `scale_height`: Scaling factor for glyph dimensions.
/// - `whole`: The `whole` offset vector for aligning the entire text block.
///
/// # Side Effects
/// Modifies the `translation` vector to include the whole offset contribution.
#[expect(
    clippy::cast_precision_loss,
    reason = "we're working on numbers small enough not to be affected"
)]
fn apply_whole_offset(
    translation: &mut Vec2,
    text_width: f32,
    max_height: u32,
    scale_height: f32,
    whole: Vec2,
) {
    *translation += Vec2::new(
        text_width * whole.x,
        max_height as f32 * whole.y * scale_height,
    );
}

/// Adjusts the translation based on the individual anchor offset.
///
/// # Parameters
/// - `translation`: A mutable reference to the translation vector to modify.
/// - `width`: The width of the current glyph.
/// - `individual`: The `individual` offset vector for aligning a single glyph.
///
/// # Side Effects
/// Modifies the `translation` vector to include the individual offset
/// contribution.
fn apply_individual_offset(translation: &mut Vec2, width: f32, individual: Vec2) {
    *translation += Vec2::new(width * individual.x, 0.0);
}

/// Converts a `Vec2` translation into a `Transform` with scaling.
///
/// # Parameters
/// - `translation`: The final 2D translation vector.
/// - `scale`: The scaling factor for both x and y dimensions.
///
/// # Returns
/// A `Transform` representing the 3D translation and scale.
fn finalize_transform(translation: Vec2, scale: f32) -> Transform {
    Transform::from_translation(translation.extend(0.0)).with_scale(Vec2::splat(scale).extend(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_translation_sets_right_value() {
        assert_eq!(base_translation(10.0), Vec2::new(10.0, 0.0));
    }

    #[test]
    fn apply_whole_offset_applies_correct_offset() {
        // Initial translation starts at (10.0, 0.0)
        let mut translation = Vec2::new(10.0, 0.0);

        // Apply a whole offset:
        // - `text_width` = 20.0 contributes 20.0 * 1.0 = 20.0 to the x-component.
        // - `max_height` = 30, `scale_height` = 0.5 contributes 30 * -1.0 * 0.5 = -15.0
        //   to the y-component.
        // - The whole offset vector (1.0, -1.0) determines these directions.
        apply_whole_offset(&mut translation, 20.0, 30, 0.5, Vec2::new(1.0, -1.0));

        // After the adjustment:
        // - x-component: 10.0 (initial) + 20.0 (whole.x contribution) = 30.0.
        // - y-component: 0.0 (initial) + (-15.0) (whole.y contribution) = -15.0.
        assert_eq!(translation, Vec2::new(30.0, -15.0));
    }

    #[test]
    fn apply_whole_offset_with_zero() {
        let mut translation = Vec2::new(10.0, 5.0);
        apply_whole_offset(&mut translation, 0.0, 0, 1.0, Vec2::ZERO);
        assert_eq!(translation, Vec2::new(10.0, 5.0)); // No change
    }

    #[test]
    fn test_apply_whole_offset_edge_cases() {
        let mut translation = Vec2::new(0.0, 0.0);

        // Zero scaling factor should result in no y adjustment
        apply_whole_offset(&mut translation, 20.0, 30, 0.0, Vec2::new(1.0, -1.0));
        assert_eq!(translation, Vec2::new(20.0, 0.0)); // Only x affected

        // Negative offsets
        apply_whole_offset(&mut translation, 50.0, 40, 1.0, Vec2::new(-1.0, 1.0));
        assert_eq!(translation, Vec2::new(-30.0, 40.0)); // Moves left and up

        // Large dimensions
        apply_whole_offset(&mut translation, 1e6, 100_000, 1.0, Vec2::new(1.0, -1.0));
        assert_eq!(translation, Vec2::new(1e6 - 30.0, -99_960.0));
    }

    #[test]
    fn apply_individual_offset_applies_the_correct_offset() {
        // Initial translation starts at (10.0, 0.0)
        let mut translation = Vec2::new(10.0, 0.0);

        // Apply an individual offset:
        // - `width` = 5.0 contributes 5.0 * 2.0 = 10.0 to the x-component.
        // - The individual offset vector (2.0, 0.0) determines these directions.
        apply_individual_offset(&mut translation, 5.0, Vec2::new(2.0, 0.0));

        // After the adjustment:
        // - x-component: 10.0 (initial) + 10.0 (individual.x contribution) = 20.0.
        // - y-component: 0.0 remains unchanged since individual.y = 0.0.
        assert_eq!(translation, Vec2::new(20.0, 0.0));
    }

    #[test]
    fn finalize_transform_produces_expected_transform() {
        // Case 1: Normal translation and scale
        let translation = Vec2::new(10.0, -5.0);
        let scale = 2.0;
        let transform = finalize_transform(translation, scale);

        assert_eq!(transform.translation, Vec3::new(10.0, -5.0, 0.0));
        assert_eq!(transform.scale, Vec3::new(2.0, 2.0, 0.0));

        // Case 2: Zero scale
        let translation = Vec2::new(-3.0, 8.0);
        let scale = 0.0;
        let transform = finalize_transform(translation, scale);

        assert_eq!(transform.translation, Vec3::new(-3.0, 8.0, 0.0));
        assert_eq!(transform.scale, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn compute_transform_produces_expected_transform() {
        let offsets = AnchorOffsets {
            whole: Vec2::new(1.0, -1.0),
            individual: Vec2::new(0.5, 0.0),
        };

        // Case 1: Normal parameters
        let transform = offsets.compute_transform(10.0, 20.0, 5.0, 30, 1.5);
        assert_eq!(transform.translation, Vec3::new(32.5, -45.0, 0.0));
        assert_eq!(transform.scale, Vec3::new(1.5, 1.5, 0.0));

        // Case 2: Zero scaling factor
        let transform = offsets.compute_transform(0.0, 10.0, 2.0, 50, 0.0);
        assert_eq!(transform.translation, Vec3::new(11.0, 0.0, 0.0));
        assert_eq!(transform.scale, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn to_anchor_offsets_are_as_expected() {
        // Case 1: Center anchor
        let anchor = Anchor::Center;
        let offsets = anchor.to_anchor_offsets();
        assert_eq!(offsets.whole, Vec2::new(-0.5, 0.0));
        assert_eq!(offsets.individual, Vec2::new(0.5, 0.0));

        // Case 2: Top-left anchor
        let anchor = Anchor::TopLeft;
        let offsets = anchor.to_anchor_offsets();
        assert_eq!(offsets.whole, Vec2::new(0.0, -0.5));
        assert_eq!(offsets.individual, Vec2::new(0.5, 0.0));

        // Case 3: Bottom-right anchor
        let anchor = Anchor::BottomRight;
        let offsets = anchor.to_anchor_offsets();
        assert_eq!(offsets.whole, Vec2::new(-1.0, 0.5));
        assert_eq!(offsets.individual, Vec2::new(0.5, 0.0));
    }
}
