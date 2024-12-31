#![doc = include_str!("../README.md")]

pub mod loader;

#[cfg(feature = "ui")]
use bevy::ui::widget::update_image_content_size_system;
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    utils::{HashMap, HashSet},
};
use bevy_image::{Image, ImageSampler};
use derive_setters::Setters;
use image::{
    imageops::{self, FilterType},
    GenericImage, GenericImageView, ImageBuffer, ImageError, Rgba,
};
use thiserror::Error;

#[derive(Default)]
pub struct ImageFontPlugin;

impl Plugin for ImageFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ImageFont>()
            .add_systems(
                PostUpdate,
                (mark_changed_fonts_as_dirty, render_sprites)
                    .chain()
                    .in_set(ImageFontSet),
            )
            .init_asset_loader::<loader::ImageFontLoader>()
            .register_type::<ImageFont>()
            .register_type::<ImageFontText>();
        #[cfg(feature = "ui")]
        app.add_systems(
            PostUpdate,
            render_ui_images
                .in_set(ImageFontSet)
                .before(update_image_content_size_system)
                .after(mark_changed_fonts_as_dirty),
        );
    }
}

/// System set for systems related to [`ImageFontPlugin`].
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct ImageFontSet;

/// An image font as well as the mapping of characters to regions inside it.
#[derive(Debug, Clone, Reflect, Asset)]
pub struct ImageFont {
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
    /// The glyph used to render `c` is contained in the part of the image
    /// pointed to by `atlas.textures[atlas_character_map[c]]`.
    pub atlas_character_map: HashMap<char, usize>,
}

impl ImageFont {
    fn mapped_atlas_layout_from_char_map(
        size: UVec2,
        char_rect_map: &HashMap<char, URect>,
    ) -> (HashMap<char, usize>, TextureAtlasLayout) {
        let mut atlas_character_map = HashMap::new();
        let mut atlas_layout = TextureAtlasLayout::new_empty(size);
        for (&c, &rect) in char_rect_map.iter() {
            atlas_character_map.insert(c, atlas_layout.add_texture(rect));
        }

        (atlas_character_map, atlas_layout)
    }

    fn from_mapped_atlas_layout(
        texture: Handle<Image>,
        atlas_character_map: HashMap<char, usize>,
        atlas_layout: Handle<TextureAtlasLayout>,
    ) -> Self {
        Self {
            atlas_layout,
            texture,
            atlas_character_map,
        }
    }

    fn filter_string(&self, s: impl AsRef<str>) -> String {
        s.as_ref()
            .chars()
            .filter(|c| self.atlas_character_map.contains_key(c))
            .collect()
    }
}

/// Text rendered using an [`ImageFont`].
#[derive(Debug, Clone, Reflect, Default, Component, Setters)]
#[setters(into)]
pub struct ImageFontText {
    pub text: String,
    pub font: Handle<ImageFont>,
    /// If set, overrides the height the font is rendered at. This should be an
    /// integer multiple of the 'native' height if you want pixel accuracy,
    /// but we allow float values for things like animations.
    pub font_height: Option<f32>,
}

/// All the components you need to render image font text 'in the world'. If you
/// want to use this with `bevy_ui`, use [`ImageFontUiText`] instead.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[require(ImageFontText, Sprite)]
pub struct ImageFontSpriteText;

/// All the components you need to render image font text in the UI. If you want
/// to display text as an entity in the world, use [`ImageFontSpriteText`]
/// instead.
#[derive(Component, Debug, Default, Clone, Reflect)]
#[cfg(feature = "ui")]
#[require(ImageFontText, ImageNode)]
pub struct ImageFontUiText;

/// System that renders each [`ImageFontText`] into the corresponding
/// `Handle<Image>`. This is mainly for use with sprites.
pub fn render_sprites(
    mut query: Query<(&ImageFontText, &mut Sprite), Changed<ImageFontText>>,
    image_fonts: Res<Assets<ImageFont>>,
    mut images: ResMut<Assets<Image>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    render_text_to_image(
        query.iter_mut().map(|(a, b)| (a, b.into_inner())),
        &image_fonts,
        &mut images,
        &layouts,
    );
}

#[cfg(feature = "ui")]
/// System that renders each [`ImageFontText`] into the corresponding
/// [`UiImage`].
pub fn render_ui_images(
    mut query: Query<(&ImageFontText, &mut ImageNode), Changed<ImageFontText>>,
    image_fonts: Res<Assets<ImageFont>>,
    mut images: ResMut<Assets<Image>>,
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    render_text_to_image(
        query.iter_mut().map(|(a, b)| (a, b.into_inner())),
        &image_fonts,
        &mut images,
        &layouts,
    );
}

/// Renders text into images and assigns the resulting image handles to their
/// holders.
///
/// This function is designed to work with any type that implements
/// [`ImageHandleHolder`], allowing it to be used with multiple components, such
/// as sprites and UI elements.
///
/// # Parameters
/// - `font_text_to_image_iter`: An iterator over pairs of [`ImageFontText`] and
///   mutable references to objects implementing [`ImageHandleHolder`]. Each
///   item in the iterator represents a text-to-image mapping to be rendered.
/// - `image_fonts`: A reference to the font assets used for rendering.
/// - `images`: A mutable reference to the collection of image assets. This is
///   used to store the newly rendered images.
/// - `layouts`: A reference to the collection of texture atlas assets.
///
/// The function iterates over the provided items, renders the text for each
/// [`ImageFontText`], and updates the corresponding [`ImageHandleHolder`] with
/// the handle to the newly created image.
///
/// # Errors
/// If text rendering fails for an item, an error message is logged, and the
/// corresponding holder is not updated.
fn render_text_to_image<'a>(
    font_text_to_image_iter: impl Iterator<
        Item = (&'a ImageFontText, &'a mut (impl ImageHandleHolder + 'a)),
    >,
    image_fonts: &Assets<ImageFont>,
    images: &mut Assets<Image>,
    layouts: &Assets<TextureAtlasLayout>,
) {
    for (image_font_text, image_handle_holder) in font_text_to_image_iter {
        debug!("Rendering [{}]", image_font_text.text);
        match render_text(image_font_text, image_fonts, images, layouts) {
            Ok(image) => {
                image_handle_holder.set_image_handle(images.add(image));
            }
            Err(e) => {
                error!(
                    "Error when rendering image font text {:?}: {}",
                    image_font_text, e
                );
            }
        }
    }
}

/// Errors that can show up during rendering.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ImageFontRenderError {
    #[error("failed to convert image to DynamicImage: {0}")]
    ImageConversion(String),
    #[error("ImageFont asset not loaded")]
    MissingImageFontAsset,
    #[error("Font texture asset not loaded")]
    MissingTextureAsset,
    #[error("internal error")]
    UnknownError,
    #[error("failed to copy from atlas")]
    CopyFailure(#[from] ImageError),
}

/// Renders the text inside the [`ImageFontText`] to a single output image. You
/// don't need to use this if you're using the built-in functionality, but if
/// you want to use this for some other custom plugin/system, you can call this.
#[allow(clippy::result_large_err)]
pub fn render_text(
    image_font_text: &ImageFontText,
    image_fonts: &Assets<ImageFont>,
    images: &Assets<Image>,
    layouts: &Assets<TextureAtlasLayout>,
) -> Result<Image, ImageFontRenderError> {
    let image_font = image_fonts
        .get(&image_font_text.font)
        .ok_or(ImageFontRenderError::MissingImageFontAsset)?;
    let font_texture = images
        .get(&image_font.texture)
        .ok_or(ImageFontRenderError::MissingTextureAsset)?;
    let layout = layouts
        .get(&image_font.atlas_layout)
        .expect("handle is kept alive by ImageFont");

    let text = image_font.filter_string(&image_font_text.text);

    if text.is_empty() {
        // can't make a 0x0 image, so make a 1x1 transparent black pixel
        return Ok(Image::new(
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        ));
    }

    // as wide as the sum of all characters, as tall as the tallest one
    let height = text
        .chars()
        .map(|c| layout.textures[image_font.atlas_character_map[&c]].height())
        .reduce(u32::max)
        .unwrap();
    let width = text
        .chars()
        .map(|c| layout.textures[image_font.atlas_character_map[&c]].width())
        .reduce(|a, b| a + b)
        .unwrap();

    let mut output_image = image::RgbaImage::new(width, height);
    let font_texture: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
        font_texture.width(),
        font_texture.height(),
        font_texture.data.as_slice(),
    )
    .ok_or(ImageFontRenderError::UnknownError)?;

    let mut x = 0;
    for c in text.chars() {
        let rect = layout.textures[image_font.atlas_character_map[&c]];
        let width = rect.width();
        let height = rect.height();
        output_image.copy_from(
            &*font_texture.view(rect.min.x, rect.min.y, width, height),
            x,
            0,
        )?;
        x += width;
    }

    if let Some(font_height) = image_font_text.font_height {
        let width = output_image.width() as f32 * font_height / output_image.height() as f32;
        output_image = imageops::resize(
            &output_image,
            width as u32,
            font_height as u32,
            FilterType::Nearest,
        );
    }

    let mut bevy_image = Image::new(
        Extent3d {
            // these might have changed because of the resize
            width: output_image.width(),
            height: output_image.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        output_image.into_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    bevy_image.sampler = ImageSampler::nearest();
    Ok(bevy_image)
}

/// Marks any text where the underlying [`ImageFont`] asset has changed as
/// dirty, which will cause it to be rerendered.
pub fn mark_changed_fonts_as_dirty(
    mut events: EventReader<AssetEvent<ImageFont>>,
    mut query: Query<&mut ImageFontText>,
) {
    let changed_fonts: HashSet<_> = events
        .read()
        .filter_map(|event| match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
                info!("Image font {id} finished loading; marking as dirty");
                Some(id)
            }
            _ => None,
        })
        .collect();
    for mut image_font_text in &mut query {
        if changed_fonts.contains(&image_font_text.font.id()) {
            image_font_text.set_changed();
        }
    }
}

/// A helper trait that represents types that can hold an image handle.
///
/// This is used to abstract over different components (e.g., [`Sprite`] and
/// [`ImageNode`]) that need to store a handle to an image rendered from text.
trait ImageHandleHolder {
    /// Sets the handle for the image that this holder represents.
    ///
    /// This method is called after rendering text into an image
    /// to update the holder with the new image handle.
    ///
    /// # Parameters
    /// - `image`: The handle to the newly rendered image.
    fn set_image_handle(&mut self, image: Handle<Image>);
}

impl ImageHandleHolder for Sprite {
    fn set_image_handle(&mut self, image: Handle<Image>) {
        self.image = image;
    }
}

#[cfg(feature = "ui")]
impl ImageHandleHolder for ImageNode {
    fn set_image_handle(&mut self, image: Handle<Image>) {
        self.image = image;
    }
}
