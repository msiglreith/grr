use __gl;
use __gl::types::GLuint;

use device::Device;
use format::Format;

///
#[repr(transparent)]
pub struct Image(GLuint);

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
            }
                if layers % 6 == 0 =>
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
            } => unsafe {
                self.0
                    .TextureStorage2D(image, levels as _, format as _, width as _, height as _);
            },
            ImageType::D2 {
                width,
                height,
                layers,
                samples,
            } => unimplemented!(),
            ImageType::D3 {
                width,
                height,
                depth,
            } => unimplemented!(),
        }

        Image(image)
    }
}
