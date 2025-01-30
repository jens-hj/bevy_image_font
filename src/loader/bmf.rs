// #![allow(
//     clippy::missing_docs_in_private_items,
//     missing_docs,
//     unused,
//     clippy::unwrap_used,
//     reason = "dev work"
// )]

//! Code for parsing an [`ImageFont`] off of an on-disk representation in `fnt`
//! format.

use bevy::log::warn;
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    image::Image,
    math::{URect, UVec2},
    sprite::TextureAtlasLayout,
    utils::HashMap,
};
use thiserror::Error;

use crate::ImageFontCharacter;
use crate::{
    loader::{ImageFontLoadError, ImageFontLoaderSettings},
    ImageFont,
};

/// Loader for [`ImageFont`]s.
#[derive(Debug, Default)]
pub struct BmFontLoader;

/// An error type representing issues that may arise during the loading of BMF
/// fonts.
///
/// This includes errors encountered when processing the font file, handling
/// associated assets, or enforcing constraints specific to the
/// [`BmFontLoader`]. Some errors originate from lower-level font loading
/// mechanisms and are wrapped transparently.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BmFontLoadError {
    /// Errors that can occur when loading an image font.
    #[error(transparent)]
    ImageFontLoadError(#[from] ImageFontLoadError),

    /// Errors that occur while parsing the BMF file format.
    #[error(transparent)]
    BmFontError(#[from] bmfont_rs::Error),

    /// [`BmFontLoader`] only supports Unicode fonts, but the loaded font is
    /// not.
    #[error("BmFontLoader only supports unicode fonts")]
    CharsetUnsupported,

    /// BMF fonts must not be packed, but the loaded font is packed.
    #[error("BmFontLoader only supports non-packed fonts")]
    PackedUnsupported,
}

impl AssetLoader for BmFontLoader {
    type Asset = ImageFont;

    type Settings = ImageFontLoaderSettings;

    type Error = BmFontLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let from_bytes = match get_extension(load_context) {
            Some("txt.fnt") => bmfont_rs::text::from_bytes,
            Some("xml.fnt") => bmfont_rs::xml::from_bytes,
            Some("bin.fnt") => bmfont_rs::binary::from_bytes,
            extension => unreachable!("{extension:?}"),
        };

        // Read data
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .await
            .map_err(ImageFontLoadError::Io)?;

        // Decode BMF format
        let bm_font = from_bytes(&data)?;

        if !bm_font.info.unicode {
            return Err(BmFontLoadError::CharsetUnsupported);
        }

        if bm_font.common.packed {
            return Err(BmFontLoadError::PackedUnsupported);
        }

        let mut atlas_character_map = HashMap::new();
        for char in &bm_font.chars {
            let x = u32::from(char.x);
            let y = u32::from(char.y);

            let rect = URect {
                min: UVec2 {
                    x: u32::from(char.x),
                    y: u32::from(char.y),
                },
                max: UVec2 {
                    x: x + u32::from(char.width),
                    y: y + u32::from(char.height),
                },
            };

            if let Some(character) = char::from_u32(char.id) {
                atlas_character_map.insert(character, (char.page, rect, None));
            } else {
                warn!(
                    "Skipping invalid character id {}. Full char definition: {char:?} ",
                    char.id
                );
            }
        }

        // Load the font images
        let mut image_handles = Vec::new();
        let mut atlas_layout_handles = Vec::new();
        for (page_no, page) in bm_font.pages.iter().enumerate() {
            let image_path = load_context
                .path()
                .parent()
                .ok_or(ImageFontLoadError::MissingParentPath)?
                .join(page);

            let Some(mut image) = load_context
                .loader()
                .immediate()
                .with_unknown_type()
                .load(image_path.clone())
                .await
                .map_err(|error| ImageFontLoadError::LoadDirect(Box::new(error)))?
                .take::<Image>()
            else {
                return Err(ImageFontLoadError::NotAnImage(page.into()).into());
            };

            image.sampler = settings.image_sampler.clone();

            let size = image.size();
            let image_handle = load_context.add_labeled_asset(format!("texture_{page_no}"), image);
            image_handles.push(image_handle);

            let mut atlas_layout = TextureAtlasLayout::new_empty(size);
            for (_, &mut (_, rect, ref mut texture_handle)) in atlas_character_map
                .iter_mut()
                .filter(|&(_, &mut (page, _, _))| page as usize == page_no)
            {
                texture_handle.replace(atlas_layout.add_texture(rect));
            }

            let layout_handle =
                load_context.add_labeled_asset(format!("layout_{page_no}"), atlas_layout);
            atlas_layout_handles.push(layout_handle);
        }

        let image_font = ImageFont {
            textures: image_handles,
            atlas_character_map: atlas_character_map
                .into_iter()
                .map(|(char, (page, _, atlas_index))| {
                    (
                        char,
                        #[expect(clippy::unwrap_used, reason = "all Nones replaced above")]
                        ImageFontCharacter {
                            character_index: atlas_index.unwrap(),
                            page_index: page as usize,
                        },
                    )
                })
                .collect(),
            atlas_layouts: atlas_layout_handles,
            image_sampler: settings.image_sampler.clone(),
        };
        Ok(image_font)
    }

    fn extensions(&self) -> &[&str] {
        &["txt.fnt", "bin.fnt", "xml.fnt"]
    }
}

/// Extracts the file extension from a given load context path.
///
/// This function attempts to retrieve the file extension for assets being
/// loaded, ensuring that it correctly identifies the correct BMF font format.
///
/// # Parameters
/// - `load_context`: The `LoadContext` containing the path of the asset being
///   loaded.
///
/// # Returns
/// An `Option<&str>` containing the file extension if it can be determined.
fn get_extension<'load_context>(
    load_context: &'load_context mut LoadContext<'_>,
) -> Option<&'load_context str> {
    load_context.path().to_str().and_then(|path| {
        path.char_indices()
            .rev()
            .nth(7 - 1)
            .map(|(i, _)| &path[i..])
    })
}
