//! Bare metal OpenGL 4.5+ wrapper
//!
//! ## Overview
//!
//! `grr` aims at providing a modern and clean looking API with focus on
//! direct state access. The terminology used follows mostly Vulkan and OpenGL.
//!
//! ## Initialization
//!
//! The main entry point for working with the library is a [`Device`](struct.Device.html).
//! A device wraps an OpenGL context which needs to be created and managed externally.
//! The documentation and examples will all use `glutin` for window and context management.
//!
//! ```rust
//! extern crate glutin;
//! extern crate grr;
//!
//! use glutin::GlContext;
//!
//! fn main() -> grr::Result<()> {
//!     let mut events_loop = glutin::EventsLoop::new();
//!     let window = glutin::WindowBuilder::new()
//!         .with_title("Hello grr!")
//!         .with_dimensions(1024, 768);
//!     let context = glutin::ContextBuilder::new()
//!         .with_vsync(true)
//!         .with_srgb(true)
//!         .with_gl_debug_flag(true);
//!
//!     let window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
//!     unsafe {
//!         window.make_current().unwrap();
//!     }
//!
//!     let grr = grr::Device::new(
//!         |symbol| window.get_proc_address(symbol) as *const _,
//!         grr::Debug::Enable {
//!             callback: |_, _, _, _, msg| {
//!                 println!("{:?}", msg);
//!             },
//!             flags: grr::DebugReport::all(),
//!         },
//!     );
//!
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! The API has multiple concepts which interplay with each other. The main of interacting with the library is via
//! function calls on a [`Device`](struct.Device.html) object. The calls often translate directly to one GL call.
//! All other objects created are opaque handles!
//!
//! * **Resource**: Objects with associated memory. Can be a [`Buffer`](struct.Buffer.html) (untyped) or an [`Image`](struct.Image.html).
//! * **Pipeline**: There currently are two sort of pipelines suppoerted:
//!       [*Graphics*](struct.Device.html#method.create_graphics_pipeline) and
//!       [*Compute*](struct.Device.html#method.create_compute_pipeline)
//! * **Framebuffer**: Assembles the attachments ([`ImageView`](struct.ImageView.html) or [`RenderBuffer`](struct.RenderBuffer.html)) for draw calls
//! * **Sampler**: Configures image filtering. An [`Image`](struct.Image.html) is bound together with a [`Sampler`](struct.Sampler.html) to a texture
//!       unit for access in a shader stage.
//! * **Vertex Array**: Specifies the vertex attributes and bindings for the input assembler stage. Buffers are bound to a [`VertexArray`](struct.VertexArray.html)
//!       to declare the memory region to fetch attribute data from.

#[macro_use]
extern crate bitflags;

mod __gl;

mod buffer;
mod command;
mod debug;
mod device;
mod error;
mod format;
mod framebuffer;
mod image;
mod pipeline;
mod sampler;
mod sync;
mod vertex;

pub use buffer::{Buffer, MappingFlags, MemoryFlags};
pub use command::{Constant, IndexTy, Primitive, Viewport};
pub use debug::{DebugReport, DebugSource, DebugType};
pub use device::{Debug, Device};
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
