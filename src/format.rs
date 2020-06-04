use crate::__gl;

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Format {
    R8_UNORM = __gl::R8,
    R8G8B8A8_SRGB = __gl::RGBA8,
    R8G8B8A8_UNORM = __gl::SRGB8_ALPHA8,
    R16G16_SFLOAT = __gl::RG16F,
    R16G16B16_SFLOAT = __gl::RGB16F,
    R16G16B16A16_SFLOAT = __gl::RGBA16F,
    D32_SFLOAT = __gl::DEPTH_COMPONENT32F,
    R32F = __gl::R32F,
    RG32F = __gl::RG32F,
    RGB32F = __gl::RGB32F,
    RGBA32F = __gl::RGBA32F,
    // TODO
}

impl Format {
    /// Return the number of components of the pixel format.
    pub fn num_components(&self) -> u32 {
        use Format::*;
        match self {
            R8_UNORM | D32_SFLOAT | R32F => 1,
            R16G16_SFLOAT | RG32F => 2,
            R16G16B16_SFLOAT | RGB32F => 3,
            R8G8B8A8_SRGB | R8G8B8A8_UNORM | RGBA32F | R16G16B16A16_SFLOAT => 4,
        }
    }

    /// Return the corresponding base format for this format.
    pub fn base_format(&self) -> BaseFormat {
        match self {
            Format::R8_UNORM => BaseFormat::R,
            Format::R8G8B8A8_SRGB => BaseFormat::RGBA,
            Format::R8G8B8A8_UNORM => BaseFormat::RGBA,
            Format::R16G16_SFLOAT => BaseFormat::RG,
            Format::R16G16B16_SFLOAT => BaseFormat::RGB,
            Format::R16G16B16A16_SFLOAT => BaseFormat::RGBA,
            Format::D32_SFLOAT => BaseFormat::Depth,
            Format::R32F => BaseFormat::R,
            Format::RG32F => BaseFormat::RG,
            Format::RGB32F => BaseFormat::RGB,
            Format::RGBA32F => BaseFormat::RGBA,
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BaseFormat {
    R = __gl::RED,
    RG = __gl::RG,
    RGB = __gl::RGB,
    RGBA = __gl::RGBA,
    Depth = __gl::DEPTH_COMPONENT,
    DepthStencil = __gl::DEPTH_STENCIL,
    Stencil = __gl::STENCIL_INDEX,
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FormatLayout {
    U8 = __gl::UNSIGNED_BYTE,
    U16 = __gl::UNSIGNED_SHORT,
    U32 = __gl::UNSIGNED_INT,
    I8 = __gl::BYTE,
    I16 = __gl::SHORT,
    I32 = __gl::INT,
    F16 = __gl::HALF_FLOAT,
    F32 = __gl::FLOAT,
    // TODO
}
