use crate::__gl;

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The `Format` enum represents a sized internal format of texture
/// storage. The naming convention closely follows that of Vulkan, but
/// it has a close correspondence with OpenGL.
///
/// * The '_UNORM' suffix denotes unsigned normalized formats. They are
/// represented as unsigned integers internally, remapped to the [0.0,
/// 1.0] floating point range in shaders. These are equivalent to the
/// constants with no suffix in OpenGL.
///
/// * The '_SNORM' suffix denotes signed normalized formats. They are
/// represented as signed integers, remapped to the [-1.0, 1.0]
/// floating point range in shaders. This suffix is the same as in
/// OpenGL.
///
/// * The '_SFLOAT' suffix denotes floating point formats, equivalent
/// to the OpenGL 'F' suffix.
///
/// * The '_INT' suffix denotes signed integer formats, exposed to
/// shaders unmodified as integers. This is equivalent to the OpenGL
/// 'I' suffix.
///
/// * The '_UINT' suffix denotes unsigned integer formats, exposed to
/// shaders as unsigned integers. This is equivalent to the OpenGL
/// 'UI' suffix.
///
/// * The '_SRGB' suffix denotes sRGB formats, which are all unsigned
/// normalized integers. Textures in this format are assumed to be in
/// the sRGB color space. Shaders reading from this format will
/// automatically convert the color components to a linear color
/// space, so the shader will only see linear values.  Because
/// `GL_FRAMEBUFFER_SRGB` is enabled by default in `grr`, when
/// outputting from a shader to a render target with an '_SRGB'
/// format, OpenGL will convert the color components to an sRGB color
/// space automatically.  Alpha components, if they exist, are treated
/// as linear throughout.
///
/// Each component is followed by the number of bits used to represent
/// it.
pub enum Format {
    // unsigned normalized integer formats
    R8_UNORM = __gl::R8,
    R8G8_UNORM = __gl::RG8,
    R8G8B8_UNORM = __gl::RGB8,
    R8G8B8A8_UNORM = __gl::RGBA8,

    R16_UNORM = __gl::R16,
    R16G16_UNORM = __gl::RG16,
    R16G16B16_UNORM = __gl::RGB16,
    R16G16B16A16_UNORM = __gl::RGBA16,

    // signed normalized integer formats
    R8_SNORM = __gl::R8_SNORM,
    R8G8_SNORM = __gl::RG8_SNORM,
    R8G8B8_SNORM = __gl::RGB8_SNORM,
    R8G8B8A8_SNORM = __gl::RGBA8_SNORM,

    R16_SNORM = __gl::R16_SNORM,
    R16G16_SNORM = __gl::RG16_SNORM,
    R16G16B16_SNORM = __gl::RGB16_SNORM,
    R16G16B16A16_SNORM = __gl::RGBA16_SNORM,

    // floating point formats
    R16_SFLOAT = __gl::R16F,
    R16G16_SFLOAT = __gl::RG16F,
    R16G16B16_SFLOAT = __gl::RGB16F,
    R16G16B16A16_SFLOAT = __gl::RGBA16F,

    R32_SFLOAT = __gl::R32F,
    R32G32_SFLOAT = __gl::RG32F,
    R32G32B32_SFLOAT = __gl::RGB32F,
    R32G32B32A32_SFLOAT = __gl::RGBA32F,

    // signed integer formats
    R8_INT = __gl::R8I,
    R8G8_INT = __gl::RG8I,
    R8G8B8_INT = __gl::RGB8I,
    R8G8B8A8_INT = __gl::RGBA8I,

    R16_INT = __gl::R16I,
    R16G16_INT = __gl::RG16I,
    R16G16B16_INT = __gl::RGB16I,
    R16G16B16A16_INT = __gl::RGBA16I,

    R32_INT = __gl::R32I,
    R32G32_INT = __gl::RG32I,
    R32G32B32_INT = __gl::RGB32I,
    R32G32B32A32_INT = __gl::RGBA32I,

    // unsigned integer formats
    R8_UINT = __gl::R8UI,
    R8G8_UINT = __gl::RG8UI,
    R8G8B8_UINT = __gl::RGB8UI,
    R8G8B8A8_UINT = __gl::RGBA8UI,

    R16_UINT = __gl::R16UI,
    R16G16_UINT = __gl::RG16UI,
    R16G16B16_UINT = __gl::RGB16UI,
    R16G16B16A16_UINT = __gl::RGBA16UI,

    R32_UINT = __gl::R32UI,
    R32G32_UINT = __gl::RG32UI,
    R32G32B32_UINT = __gl::RGB32UI,
    R32G32B32A32_UINT = __gl::RGBA32UI,

    // sRGB normalized integer formats.
    R8G8B8_SRGB = __gl::SRGB8,

    /// sRGB8 color space with a linear alpha
    R8G8B8A8_SRGB = __gl::SRGB8_ALPHA8,

    // depth and stencil formats
    D16_UNORM = __gl::DEPTH_COMPONENT16,
    D24_UNORM = __gl::DEPTH_COMPONENT24,
    D32_UNORM = __gl::DEPTH_COMPONENT32,
    D32_SFLOAT = __gl::DEPTH_COMPONENT32F,

    S8_UINT = __gl::STENCIL_INDEX8,

    D24_UNORM_S8_UINT = __gl::DEPTH24_STENCIL8,
    D32_SFLOAT_S8_UINT = __gl::DEPTH32F_STENCIL8,
}

impl Format {
    /// Return the number of components of the pixel format.
    pub fn num_components(self) -> u32 {
        self.base_format().num_components()
    }

    /// Return the corresponding base format for this format.
    pub fn base_format(self) -> BaseFormat {
        use Format::*;
        match self {
            R8_UNORM | R16_UNORM | R8_SNORM | R16_SNORM | R8_INT | R16_INT | R32_INT | R8_UINT
            | R16_UINT | R32_UINT | R16_SFLOAT | R32_SFLOAT => BaseFormat::R,

            R8G8_UNORM | R16G16_UNORM | R8G8_SNORM | R16G16_SNORM | R8G8_INT | R16G16_INT
            | R32G32_INT | R8G8_UINT | R16G16_UINT | R32G32_UINT | R16G16_SFLOAT
            | R32G32_SFLOAT => BaseFormat::RG,

            R8G8B8_UNORM | R16G16B16_UNORM | R8G8B8_SNORM | R16G16B16_SNORM | R8G8B8_INT
            | R16G16B16_INT | R32G32B32_INT | R8G8B8_UINT | R16G16B16_UINT | R32G32B32_UINT
            | R16G16B16_SFLOAT | R32G32B32_SFLOAT => BaseFormat::RGB,

            R8G8B8A8_UNORM | R16G16B16A16_UNORM | R8G8B8A8_SNORM | R16G16B16A16_SNORM
            | R8G8B8A8_INT | R16G16B16A16_INT | R32G32B32A32_INT | R8G8B8A8_UINT
            | R16G16B16A16_UINT | R32G32B32A32_UINT | R16G16B16A16_SFLOAT | R32G32B32A32_SFLOAT => {
                BaseFormat::RGBA
            }

            R8G8B8_SRGB => BaseFormat::RGB,

            R8G8B8A8_SRGB => BaseFormat::RGBA,

            D32_SFLOAT | D16_UNORM | D24_UNORM | D32_UNORM => BaseFormat::Depth,

            S8_UINT => BaseFormat::Stencil,

            D32_SFLOAT_S8_UINT | D24_UNORM_S8_UINT => BaseFormat::DepthStencil,
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

impl BaseFormat {
    /// Return the number of components that compose this format.
    pub fn num_components(self) -> u32 {
        use BaseFormat::*;
        match self {
            R => 1,
            RG => 2,
            RGB => 3,
            RGBA => 4,
            Depth => 1,
            DepthStencil => 2,
            Stencil => 1,
        }
    }
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
    U24U8 = __gl::UNSIGNED_INT_24_8,
    F32U8 = __gl::FLOAT_32_UNSIGNED_INT_24_8_REV,
    // TODO
}
