use __gl::types::GLuint;

use buffer::Buffer;

///
#[repr(transparent)]
pub struct VertexArray(pub(crate) GLuint);

///
pub struct VertexBufferView<'a> {
    pub buffer: &'a Buffer,
    pub offset: u64,
    pub stride: u32,
    pub input_rate: InputRate,
}

///
pub struct VertexAttributeDesc {
    pub location: u32,
    pub binding: u32,
    pub format: VertexFormat,
    pub offset: u32,
}

///
pub enum InputRate {
    Vertex,
    Instance { divisor: usize },
}

///
pub enum VertexFormat {
    X8Int,
    X8Uint,
    X8Unorm,
    X8Inorm,
    X8Uscaled,
    X8Iscaled,

    Xy8Int,
    Xy8Uint,
    Xy8Unorm,
    Xy8Inorm,
    Xy8Uscaled,
    Xy8Iscaled,

    Xyz8Int,
    Xyz8Uint,
    Xyz8Unorm,
    Xyz8Inorm,
    Xyz8Uscaled,
    Xyz8Iscaled,

    Xyzw8Int,
    Xyzw8Uint,
    Xyzw8Unorm,
    Xyzw8Inorm,
    Xyzw8Uscaled,
    Xyzw8Iscaled,

    X16Int,
    X16Uint,
    X16Float,
    X16Unorm,
    X16Inorm,
    X16Uscaled,
    X16Iscaled,

    Xy16Int,
    Xy16Uint,
    Xy16Float,
    Xy16Unorm,
    Xy16Inorm,
    Xy16Uscaled,
    Xy16Iscaled,

    Xyz16Int,
    Xyz16Uint,
    Xyz16Float,
    Xyz16Unorm,
    Xyz16Inorm,
    Xyz16Uscaled,
    Xyz16Iscaled,

    Xyzw16Int,
    Xyzw16Uint,
    Xyzw16Float,
    Xyzw16Unorm,
    Xyzw16Inorm,
    Xyzw16Uscaled,
    Xyzw16Iscaled,

    X32Int,
    X32Uint,
    X32Float,
    X32Unorm,
    X32Inorm,
    X32Uscaled,
    X32Iscaled,

    Xy32Int,
    Xy32Uint,
    Xy32Float,
    Xy32Unorm,
    Xy32Inorm,
    Xy32Uscaled,
    Xy32Iscaled,

    Xyz32Int,
    Xyz32Uint,
    Xyz32Float,
    Xyz32Unorm,
    Xyz32Inorm,
    Xyz32Uscaled,
    Xyz32Iscaled,

    Xyzw32Int,
    Xyzw32Uint,
    Xyzw32Float,
    Xyzw32Unorm,
    Xyzw32Inorm,
    Xyzw32Uscaled,
    Xyzw32Iscaled,

    X64Float,
    Xy64Float,
    Xyz64Float,
    Xyzw64Float,
}
