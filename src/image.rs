use __gl;
use __gl::types::{GLenum, GLuint};

use std::ops::Range;

use device::Device;
use format::{BaseFormat, Format, FormatLayout};
use {Extent, Offset};

///
pub struct Image {
    raw: GLuint,
    target: GLenum,
}

pub enum ImageType {
    D1 {
        width: u32,
        layers: u32,
    },
    D2 {
        width: u32,
        height: u32,
        layers: u32,
        samples: u32,
    },
    D3 {
        width: u32,
        height: u32,
        depth: u32,
    },
}

///
#[repr(transparent)]
pub struct ImageView(GLuint);

pub enum ImageViewType {
    D1,
    D2,
    D3,
    Cube,
    D1Array,
    D2Array,
    CubeArray,
}

pub struct SubresourceRange {
    pub levels: Range<u32>,
    pub layers: Range<u32>,
}

impl Device {
    pub fn create_image(&self, ty: ImageType, format: Format, levels: u32) -> Image {
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
            } if layers % 6 == 0 =>
            {
                __gl::TEXTURE_CUBE_MAP_ARRAY
            }
            ImageType::D2 { samples: 1, .. } => __gl::TEXTURE_2D_ARRAY,
            ImageType::D2 { layers: 1, .. } => __gl::TEXTURE_2D_MULTISAMPLE,
            ImageType::D2 { .. } => __gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
            ImageType::D3 { .. } => __gl::TEXTURE_3D,
        };

        let mut image = 0;
        unsafe {
            self.0.CreateTextures(target, 1, &mut image);
        }
        self.get_error("CreateTextures");

        match ty {
            ImageType::D1 { width, layers: 1 } => unsafe {
                self.0
                    .TextureStorage1D(image, levels as _, format as _, width as _);
                self.get_error("TextureStorage1D");
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
            } => unsafe {
                self.0
                    .TextureStorage2D(image, levels as _, format as _, width as _, height as _);
                self.get_error("TextureStorage2D");
            },
            ImageType::D2 { .. } => unimplemented!(),
            ImageType::D3 { .. } => unimplemented!(),
        }

        Image { raw: image, target }
    }

    pub fn copy_host_to_image(
        &self,
        image: &Image,
        level: u32,
        layers: Range<u32>,
        offset: Offset,
        extent: Extent,
        data: &[u8],
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

    pub fn create_image_view(
        &self,
        image: &Image,
        ty: ImageViewType,
        format: Format,
        range: SubresourceRange,
    ) -> ImageView {
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
        self.get_error("GenTextures");

        unsafe {
            self.0.TextureView(
                view,
                target,
                image.raw,
                format as _,
                range.levels.start,
                range.levels.end,
                range.layers.start,
                range.layers.end,
            );
        }
        self.get_error("TextureView");

        ImageView(view)
    }

    pub fn bind_image_views(&self, first: u32, views: &[&ImageView]) {
        let views = views.iter().map(|view| view.0).collect::<Vec<_>>();
        unsafe {
            self.0.BindTextures(first, views.len() as _, views.as_ptr());
        }
        self.get_error("BindTextures");
    }

    pub fn generate_mipmaps(&self, image: &Image) {
        unsafe {
            self.0.GenerateTextureMipmap(image.raw);
        }
        self.get_error("GenerateTextureMipmap");
    }
}
