//!  Image storage and views.

use crate::__gl;
use crate::__gl::types::{GLenum, GLuint};

use std::ops::Range;

use crate::buffer::BufferRange;
use crate::debug::{Object, ObjectType};
use crate::device::Device;
use crate::error::Result;
use crate::format::Format;
use crate::Extent;

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
    pub(crate) raw: GLuint,
    pub(crate) target: GLenum,
}

impl Object for Image {
    const TYPE: ObjectType = ObjectType::Image;
    fn handle(&self) -> GLuint {
        self.raw
    }
}

impl Image {
    /// Get image view from current image.
    ///
    /// # Valid Usage
    ///
    /// - These image view **must** not be deleted.
    pub fn as_view(&self) -> ImageView {
        ImageView(self.raw)
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
    /// Return the number of texels in the top layer of the image.
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

    /// Return the width of the image.
    pub fn width(&self) -> u32 {
        match *self {
            ImageType::D1 { width, .. }
            | ImageType::D2 { width, .. }
            | ImageType::D3 { width, .. } => width,
        }
    }

    /// Return the height of the image.
    pub fn height(&self) -> u32 {
        match *self {
            ImageType::D1 { .. } => 1,
            ImageType::D2 { height, .. } | ImageType::D3 { height, .. } => height,
        }
    }

    /// Return the height of the image.
    pub fn depth(&self) -> u32 {
        match *self {
            ImageType::D1 { .. } | ImageType::D2 { .. } => 1,
            ImageType::D3 { depth, .. } => depth,
        }
    }

    /// Full extent of the image dimensions for a single layer.
    pub fn full_extent(&self) -> Extent {
        Extent {
            width: self.width(),
            height: self.height(),
            depth: self.depth(),
        }
    }

    /// Return the number of samples in a texel of the image.
    pub fn samples(&self) -> u32 {
        match *self {
            ImageType::D1 { .. } | ImageType::D3 { .. } => 1,
            ImageType::D2 { samples, .. } => samples,
        }
    }

    /// Return the number of layers in the texutre.
    pub fn layers(&self) -> u32 {
        match *self {
            ImageType::D1 { layers, .. } | ImageType::D2 { layers, .. } => layers,
            ImageType::D3 { .. } => 1,
        }
    }

    fn view_ty(&self) -> ImageViewType {
        match *self {
            ImageType::D1 { layers: 1, .. } => ImageViewType::D1,
            ImageType::D1 { .. } => ImageViewType::D1Array,
            ImageType::D2 { layers: 1, .. } => ImageViewType::D2,
            ImageType::D2 { .. } => ImageViewType::D2Array,
            ImageType::D3 { .. } => ImageViewType::D3,
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone)]
pub struct SubresourceRange {
    /// Range of mip levels.
    pub levels: Range<u32>,
    /// Range of array layers.
    pub layers: Range<u32>,
}

#[derive(Debug, Clone)]
pub struct SubresourceLayers {
    /// Mipmap level.
    pub level: u32,
    /// Range of array layers.
    pub layers: Range<u32>,
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

    /// Create a texel buffer.
    pub unsafe fn create_texel_buffer(&self, buffer: BufferRange, format: Format) -> Result<Image> {
        let mut image = 0;
        self.0.CreateTextures(__gl::TEXTURE_BUFFER, 1, &mut image);
        self.get_error()?;

        self.0.TextureBufferRange(image, format as _, buffer.buffer.0, buffer.offset as _, buffer.size as _);
        self.get_error()?;

        Ok(Image { raw: image, target: __gl::TEXTURE_BUFFER })
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

    /// Create an image and an associated view.
    ///
    /// The image view type is derived from the `ImageType`.
    /// It creates either non-arrayed or arrayed view types.
    pub unsafe fn create_image_and_view(
        &self,
        ty: ImageType,
        format: Format,
        levels: u32,
    ) -> Result<(Image, ImageView)> {
        let image = self.create_image(ty, format, levels)?;
        let view_ty = ty.view_ty();
        let image_view = self.create_image_view(
            image,
            view_ty,
            format,
            SubresourceRange {
                levels: 0..levels,
                layers: 0..ty.layers(),
            },
        )?;

        Ok((image, image_view))
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
