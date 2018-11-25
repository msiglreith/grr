use __gl;

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Format {
    R8_UNORM = __gl::R8,
    R8G8B8A8_SRGB = __gl::RGBA8,
    R8G8B8A8_UNORM = __gl::SRGB8_ALPHA8,
    R16G16_SFLOAT = __gl::RG16F,
    R16G16B16_SFLOAT = __gl::RGB16F,
    // TODO
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BaseFormat {
    R = __gl::RED,
    RG = __gl::RG,
    RGB = __gl::RGB,
    RGBA = __gl::RGBA,
    // TODO
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FormatLayout {
    U8 = __gl::UNSIGNED_BYTE,
    I8 = __gl::BYTE,
    F16 = __gl::HALF_FLOAT,
    F32 = __gl::FLOAT,
    // TODO
}
