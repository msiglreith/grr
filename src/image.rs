//!  Image storage and views.

use __gl;
use __gl::types::{GLenum, GLuint};

use std::ops::Range;

use debug::{Object, ObjectType};
use device::Device;
use error::Result;
use format::{BaseFormat, Format, FormatLayout};
use {Extent, Offset};

/// Image resource handle.
///
/// Images ares multisdimensional, formatted data arrays.
/// Images are, together with buffers, resources and are typed representations
/// of a memory slice.
///
/// Images are often also called textures. We only denote with an image
/// the actual **storage** of the data, meaning the memory with the
/// associated layout metadata.
///
/// The API only uses images directlt when the function call directly
/// affects the underlying memory (e.g copy operations).
pub struct Image {
    raw: GLuint,
    target: GLenum,
}

impl Object for Image {
    const TYPE: ObjectType = ObjectType::Image;
    fn handle(&self) -> GLuint {
        self.raw
    }
}

/// Image dimensionality type.
///
/// Layer, as in arrays or cube maps. don't affect
/// the dimensionality type.
pub enum ImageType {
    // One dimensional image.
    D1 {
        // Width.
        width: u32,

        // Number of layers.
        //
        // `1` for non-array textures.
        layers: u32,
    },
    // Two dimensional image.
    D2 {
        // Width.
        width: u32,

        // Height.
        height: u32,

        // Number of layers.
        //
        // `1` for non-array textures.
        layers: u32,

        samples: u32,
    },
    // Three dimensional image.
    D3 {
        // Width.
        width: u32,

        // Height.
        height: u32,

        // Depth.
        depth: u32,
    },
}

/// Image view handle.
///
/// Image Views denote subranges of an image storage. Pipelines will
/// only access image data via views. Views alias the memory of the associated
/// image.
#[repr(transparent)]
pub struct ImageView(pub(crate) GLuint);

///
pub enum ImageViewType {
    D1,
    D2,
    D3,
    Cube,
    D1Array,
    D2Array,
    CubeArray,
}

///
pub struct SubresourceRange {
    pub levels: Range<u32>,
    pub layers: Range<u32>,
}

impl Device {
    ///
    pub fn create_image(&self, ty: ImageType, format: Format, levels: u32) -> Result<Image> {
        let target = match ty {
            ImageType::D1 { layers: 1, .. } => __gl::TEXTURE_1D,
            ImageType::D1 { .. } => __gl::TEXTURE_1D_ARRAY,
            ImageType::D2 {
                layers: 1,
                samples: 1,
                ..
            } => __gl::TEXTURE_2D,
            ImageType::D2 {
                layers: 6,
                samples: 1,
                ..
            } => __gl::TEXTURE_CUBE_MAP,
            ImageType::D2 {
                layers, samples: 1, ..
            } if layers % 6 == 0 => __gl::TEXTURE_CUBE_MAP_ARRAY,
            ImageType::D2 { samples: 1, .. } => __gl::TEXTURE_2D_ARRAY,
            ImageType::D2 { layers: 1, .. } => __gl::TEXTURE_2D_MULTISAMPLE,
            ImageType::D2 { .. } => __gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
            ImageType::D3 { .. } => __gl::TEXTURE_3D,
        };

        let mut image = 0;
        unsafe {
            self.0.CreateTextures(target, 1, &mut image);
        }
        self.get_error()?;

        match ty {
            ImageType::D1 { width, layers: 1 } => unsafe {
                self.0
                    .TextureStorage1D(image, levels as _, format as _, width as _);
            },
            ImageType::D1 {
                width,
                layers: height,
            }
            | ImageType::D2 {
                width,
                height,
                layers: 1,
                samples: 1,
            }
            | ImageType::D2 {
                width,
                height,
                layers: 6,
                samples: 1,
            } => unsafe {
                self.0
                    .TextureStorage2D(image, levels as _, format as _, width as _, height as _);
            },
            ImageType::D2 {
                width,
                height,
                layers: depth,
                samples: 1,
            }
            | ImageType::D3 {
                width,
                height,
                depth,
            } => unsafe {
                self.0.TextureStorage3D(
                    image,
                    levels as _,
                    format as _,
                    width as _,
                    height as _,
                    depth as _,
                );
            },
            _ => unimplemented!(),
        }
        self.get_error()?;

        Ok(Image { raw: image, target })
    }

    /// Delete an images.
    pub fn delete_image(&self, image: Image) {
        self.delete_images(&[image]);
    }

    /// Delete multiple images.
    pub fn delete_images(&self, images: &[Image]) {
        let images = images.iter().map(|i| i.raw).collect::<Vec<_>>();
        unsafe {
            self.0
                .DeleteTextures(images.len() as _, images.as_ptr() as *const _);
        }
    }

    /// Copy image data from host memory to device memory.
    pub fn copy_host_to_image<T>(
        &self,
        image: &Image,
        level: u32,
        layers: Range<u32>,
        offset: Offset,
        extent: Extent,
        data: &[T],
        base_format: BaseFormat,
        format_layout: FormatLayout,
    ) {
        match image.target {
            __gl::TEXTURE_2D if layers == (0..1) => unsafe {
                self.0.TextureSubImage2D(
                    image.raw,
                    level as _,
                    offset.x,
                    offset.y,
                    extent.width as _,
                    extent.height as _,
                    base_format as _,
                    format_layout as _,
                    data.as_ptr() as *const _,
                )
            },
            _ => unimplemented!(), // panic!("Invalid target image: {}", image.target),
        }
    }

    /// Create an image view from an image.
    pub fn create_image_view(
        &self,
        image: &Image,
        ty: ImageViewType,
        format: Format,
        range: SubresourceRange,
    ) -> Result<ImageView> {
        let target = match ty {
            ImageViewType::D1 => __gl::TEXTURE_1D,
            ImageViewType::D2 if image.target == __gl::TEXTURE_2D_MULTISAMPLE => {
                __gl::TEXTURE_2D_MULTISAMPLE
            }
            ImageViewType::D2 => __gl::TEXTURE_2D,
            ImageViewType::D3 => __gl::TEXTURE_3D,
            ImageViewType::Cube => __gl::TEXTURE_CUBE_MAP,
            ImageViewType::D1Array => __gl::TEXTURE_1D_ARRAY,
            ImageViewType::D2Array if image.target == __gl::TEXTURE_2D_MULTISAMPLE_ARRAY => {
                __gl::TEXTURE_2D_MULTISAMPLE_ARRAY
            }
            ImageViewType::D2Array => __gl::TEXTURE_2D_ARRAY,
            ImageViewType::CubeArray => __gl::TEXTURE_CUBE_MAP_ARRAY,
        };
        let mut view = 0;
        unsafe {
            self.0.GenTextures(1, &mut view);
        }
        self.get_error()?;

        unsafe {
            self.0.TextureView(
                view,
                target,
                image.raw,
                format as _,
                range.levels.start,
                range.levels.end - range.levels.start,
                range.layers.start,
                range.layers.end - range.layers.start,
            );
        }
        self.get_error()?;

        Ok(ImageView(view))
    }

    /// Delete an image views.
    pub fn delete_image_view(&self, view: ImageView) {
        self.delete_image_views(&[view]);
    }

    /// Delete multipe image views.
    pub fn delete_image_views(&self, views: &[ImageView]) {
        unsafe {
            self.0.DeleteTextures(
                views.len() as _,
                views.as_ptr() as *const _, // newtype
            );
        }
    }

    /// Bind image views to texture units.
    pub fn bind_image_views(&self, first: u32, views: &[&ImageView]) {
        let views = views.iter().map(|view| view.0).collect::<Vec<_>>();
        unsafe {
            self.0.BindTextures(first, views.len() as _, views.as_ptr());
        }
    }

    /// Generate mipmaps.
    ///
    /// This generates the remaining mipmap levels using the base layer
    /// via downscaling. The number of levels are determined on resource
    /// creation.
    ///
    /// The downscaling filter is implementation dependent!
    pub fn generate_mipmaps(&self, image: &Image) {
        unsafe {
            self.0.GenerateTextureMipmap(image.raw);
        }
    }
}
