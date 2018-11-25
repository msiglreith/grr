#[macro_use]
extern crate bitflags;

mod __gl;

mod buffer;
mod command;
mod device;
mod error;
mod format;
mod framebuffer;
mod image;
mod pipeline;
mod sampler;
mod vertex;

pub use buffer::{Buffer, MappingFlags, MemoryFlags};
pub use command::Constant;
pub use device::Device;
pub use error::Error;
pub use format::{BaseFormat, Format, FormatLayout};
pub use framebuffer::{Attachment, AttachmentView, ClearAttachment, Framebuffer, Renderbuffer};
pub use image::{Image, ImageType, ImageView, ImageViewType, SubresourceRange};
pub use pipeline::{
    BlendChannel, BlendFactor, BlendOp, ColorBlend, ColorBlendAttachment, DepthStencil,
    GraphicsPipelineDesc, InputAssembly, Pipeline, Shader, ShaderStage, StencilFace, StencilOp,
};
pub use sampler::{Filter, Sampler, SamplerAddress, SamplerDesc};
pub use vertex::{InputRate, VertexArray, VertexAttributeDesc, VertexBufferView, VertexFormat};

///
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    /// Width
    pub w: f32,
    /// Height
    pub h: f32,
    // Near
    pub n: f64,
    // Far
    pub f: f64,
}

///
pub struct Region {
    pub x: i32,
    pub y: i32,
    /// Width
    pub w: i32,
    /// Height
    pub h: i32,
}

///
pub struct Offset {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

///
pub struct Extent {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

///
#[repr(u32)]
pub enum Primitive {
    Points = __gl::POINTS,
    Lines = __gl::LINES,
    LineStrip = __gl::LINE_STRIP,
    Triangles = __gl::TRIANGLES,
    TriangleStrip = __gl::TRIANGLE_STRIP,
    LinesAdjacency = __gl::LINES_ADJACENCY,
    LinesStripAdjacency = __gl::LINE_STRIP_ADJACENCY,
    TrianglesAdjacency = __gl::TRIANGLES_ADJACENCY,
    TrianglesStripAdjacency = __gl::TRIANGLE_STRIP_ADJACENCY,
    Patches = __gl::PATCHES,
}

///
#[repr(u32)]
pub enum IndexTy {
    U8 = __gl::UNSIGNED_BYTE,
    U16 = __gl::UNSIGNED_SHORT,
    U32 = __gl::UNSIGNED_INT,
}

///
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Compare {
    Less = __gl::LESS,
    LessEqual = __gl::LEQUAL,
    Greater = __gl::GREATER,
    GreaterEqual = __gl::GEQUAL,
    Equal = __gl::EQUAL,
    NotEqual = __gl::NOTEQUAL,
    Always = __gl::ALWAYS,
    Never = __gl::NEVER,
}
