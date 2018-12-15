use __gl;
use __gl::types::{GLenum, GLuint};
use device::Device;

bitflags!(
    /// Debug report flags.
    ///
    /// Denotes which events will trigger a debug report.
    pub struct DebugReport: GLenum {
        const NOTIFICATION = __gl::DEBUG_SEVERITY_NOTIFICATION;
        const WARNING = __gl::DEBUG_SEVERITY_MEDIUM;
        const ERORR = __gl::DEBUG_SEVERITY_HIGH;
        const PERFORMANCE_WARNING = __gl::DEBUG_SEVERITY_LOW;
    }
);

/// Debug message source.
#[repr(u32)]
pub enum DebugSource {
    Api = __gl::DEBUG_SOURCE_API,
    ShaderCompiler = __gl::DEBUG_SOURCE_SHADER_COMPILER,
    Wsi = __gl::DEBUG_SOURCE_WINDOW_SYSTEM,
    ThirdParty = __gl::DEBUG_SOURCE_THIRD_PARTY,
    Application = __gl::DEBUG_SOURCE_APPLICATION,
    Other = __gl::DEBUG_SOURCE_OTHER,
}

/// Debug message type.
#[repr(u32)]
pub enum DebugType {
    Error = __gl::DEBUG_TYPE_ERROR,
    Deprecated = __gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR,
    UndefinedBehavior = __gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR,
    Performance = __gl::DEBUG_TYPE_PERFORMANCE,
    Portability = __gl::DEBUG_TYPE_PORTABILITY,
    Marker = __gl::DEBUG_TYPE_MARKER,
    PushGroup = __gl::DEBUG_TYPE_PUSH_GROUP,
    PopGroup = __gl::DEBUG_TYPE_POP_GROUP,
    Other = __gl::DEBUG_TYPE_OTHER,
}

///
pub type DebugCallback = fn(DebugReport, DebugSource, DebugType, u32, &str);

///
#[repr(u32)]
pub enum ObjectType {
    Buffer = __gl::BUFFER,
    Shader = __gl::SHADER,
    Image = __gl::TEXTURE,
    VertexArray = __gl::VERTEX_ARRAY,
    Pipeline = __gl::PROGRAM,
    Framebuffer = __gl::FRAMEBUFFER,
    Renderbuffer = __gl::RENDERBUFFER,
    Sampler = __gl::SAMPLER,
}

pub trait Object {
    const TYPE: ObjectType;

    fn handle(&self) -> GLuint;
}

impl Device {
    /// Associate a name with an object.
    pub fn object_name<T: Object>(&self, object: &T, name: &str) {
        let label = name.as_bytes();
        unsafe {
            self.0.ObjectLabel(
                T::TYPE as _,
                object.handle(),
                label.len() as _,
                label.as_ptr() as *const _,
            );
        }
    }
}
