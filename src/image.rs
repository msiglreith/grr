//!  Image storage and views.

use crate::__gl;
use crate::__gl::types::{GLenum, GLuint};

use std::ops::Range;

use crate::debug::{Object, ObjectType};
use crate::device::Device;
use crate::error::Result;
use crate::format::{BaseFormat, Format, FormatLayout};
use crate::{Extent, Offset};

/// Image resource handle.
///
/// Images ares multidimensional, formatted data arrays.
/// Images are, together with buffers, resources and are typed representations
/// of a memory slice.
///
/// Images are often also called textures. We only denote with an image
/// the actual **storage** of the data, meaning the memory with the
/// associated layout metadata.
///
/// The API only uses images directly when the function call
/// affects the underlying memory (e.g copy operations).
#[derive(Clone, Copy)]
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
/// Layer, as in arrays or cube maps, don't affect the dimensionality type.
#[derive(Clone, Copy)]
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

impl ImageType {
    /// Return the number of texels in a single layer of the texture.
    pub fn num_texels(&self) -> usize {
        match *self {
            ImageType::D1 { width, .. } => width as usize,
            ImageType::D2 { width, height, .. } => width as usize * height as usize,
            ImageType::D3 {
                width,
                height,
                depth,
                ..
            } => width as usize * height as usize * depth as usize,
        }
    }
}

/// Image view handle.
///
/// Image Views denote subranges of an image storage. Pipelines will
/// only access image data via views. Views alias the memory of the associated
/// image.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ImageView(pub(crate) GLuint);

impl Object for ImageView {
    const TYPE: ObjectType = ObjectType::Image; // internally it's an image
    fn handle(&self) -> GLuint {
        self.0
    }
}

/// Image View type.
///
/// An `ImageViewType` maps roughly to OpenGL texture targets.
#[derive(Clone, Copy)]
pub enum ImageViewType {
    D1,
    D2,
    D3,
    Cube,
    D1Array,
    D2Array,
    CubeArray,
}

/// Subresource of an image.
pub struct SubresourceRange {
    /// Range of mip levels.
    pub levels: Range<u32>,
    /// Range of array layers.
    pub layers: Range<u32>,
}

pub struct SubresourceLevel {
    /// Mip level.
    pub level: u32,
    /// Range of array layers.
    pub layers: Range<u32>,
}

///
pub struct SubresourceLayout {
    pub base_format: BaseFormat,
    pub format_layout: FormatLayout,
    pub row_pitch: u32,
    pub image_height: u32,
    pub alignment: u32,
}

impl Device {
    ///
    pub unsafe fn create_image(&self, ty: ImageType, format: Format, levels: u32) -> Result<Image> {
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
        self.0.CreateTextures(target, 1, &mut image);
        self.get_error()?;

        match ty {
            ImageType::D1 { width, layers: 1 } => {
                self.0
                    .TextureStorage1D(image, levels as _, format as _, width as _);
            }
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
            } => {
                self.0
                    .TextureStorage2D(image, levels as _, format as _, width as _, height as _);
            }
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
            } => {
                self.0.TextureStorage3D(
                    image,
                    levels as _,
                    format as _,
                    width as _,
                    height as _,
                    depth as _,
                );
            }
            _ => unimplemented!(),
        }
        self.get_error()?;

        Ok(Image { raw: image, target })
    }

    /// Delete an images.
    pub unsafe fn delete_image(&self, image: Image) {
        self.delete_images(&[image]);
    }

    /// Delete multiple images.
    pub unsafe fn delete_images(&self, images: &[Image]) {
        let images = images.iter().map(|i| i.raw).collect::<Vec<_>>();

        self.0
            .DeleteTextures(images.len() as _, images.as_ptr() as *const _);
    }

    /// Copy image data from host memory to device memory.
    pub unsafe fn copy_host_to_image<T>(
        &self,
        image: Image,
        subresource: SubresourceLevel,
        offset: Offset,
        extent: Extent,
        data: &[T],
        layout: SubresourceLayout,
    ) {
        self.0
            .PixelStorei(__gl::UNPACK_ALIGNMENT, layout.alignment as _);
        match image.target {
            __gl::TEXTURE_2D if subresource.layers == (0..1) => self.0.TextureSubImage2D(
                image.raw,
                subresource.level as _,
                offset.x,
                offset.y,
                extent.width as _,
                extent.height as _,
                layout.base_format as _,
                layout.format_layout as _,
                data.as_ptr() as *const _,
            ),
            _ => unimplemented!(), // panic!("Invalid target image: {}", image.target),
        }
    }

    /// Create an image view from an image.
    pub unsafe fn create_image_view(
        &self,
        image: Image,
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
        self.0.GenTextures(1, &mut view);
        self.get_error()?;

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
        self.get_error()?;

        Ok(ImageView(view))
    }

    /// Delete an image views.
    pub unsafe fn delete_image_view(&self, view: ImageView) {
        self.delete_image_views(&[view]);
    }

    /// Delete multipe image views.
    pub unsafe fn delete_image_views(&self, views: &[ImageView]) {
        self.0.DeleteTextures(
            views.len() as _,
            views.as_ptr() as *const _, // newtype
        );
    }

    /// Bind image views to texture units.
    pub unsafe fn bind_image_views(&self, first: u32, views: &[ImageView]) {
        let views = views.iter().map(|view| view.0).collect::<Vec<_>>();
        self.0.BindTextures(first, views.len() as _, views.as_ptr());
    }

    /// Bind image views to storage image units.
    pub unsafe fn bind_storage_image_views(&self, first: u32, views: &[ImageView]) {
        let views = views.iter().map(|view| view.0).collect::<Vec<_>>();
        self.0
            .BindImageTextures(first, views.len() as _, views.as_ptr());
    }

    /// Generate mipmaps.
    ///
    /// This generates the remaining mipmap levels using the base layer
    /// via downscaling. The number of levels are determined on resource
    /// creation.
    ///
    /// The downscaling filter is implementation dependent!
    pub unsafe fn generate_mipmaps(&self, image: Image) {
        self.0.GenerateTextureMipmap(image.raw);
    }
}
