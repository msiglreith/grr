#[macro_use]
extern crate bitflags;

mod __gl;

pub mod buffer;
pub mod command;
pub mod device;
pub mod error;
pub mod format;
pub mod framebuffer;
pub mod image;
pub mod pipeline;
pub mod sampler;
pub mod sync;
pub mod vertex;

pub use buffer::{Buffer, MappingFlags, MemoryFlags};
pub use command::{Constant, IndexTy, Primitive, Viewport};
pub use device::{Debug, DebugReport, DebugSource, DebugType, Device};
pub use error::{Error, Result};
pub use format::{BaseFormat, Format, FormatLayout};
pub use framebuffer::{Attachment, AttachmentView, ClearAttachment, Framebuffer, Renderbuffer};
pub use image::{Image, ImageType, ImageView, ImageViewType, SubresourceRange};
pub use pipeline::{
    BlendChannel, BlendFactor, BlendOp, ColorBlend, ColorBlendAttachment, DepthStencil,
    GraphicsPipelineDesc, InputAssembly, Pipeline, Shader, ShaderStage, StencilFace, StencilOp,
};
pub use sampler::{Filter, Sampler, SamplerAddress, SamplerDesc};
pub use sync::{Barrier, RegionBarrier};
pub use vertex::{InputRate, VertexArray, VertexAttributeDesc, VertexBufferView, VertexFormat};

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

/// Comparison operator.
///
/// Used in depth test, stencil test and sampling depth textures.
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
