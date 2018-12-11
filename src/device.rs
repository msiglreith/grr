use __gl;
use __gl::types::{GLchar, GLenum, GLsizei, GLuint};

use std::os::raw::c_void;
use std::{ffi, mem};

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

/// Device debug control.
pub enum Debug<F> {
    Enable { callback: F, flags: DebugReport },
    Disable,
}

/// Logical device, representation one or multiple physical devices (hardware or software).
///
/// This wraps an existing GL context and acts as the main API interface.
/// It's the responsibility of the user to keep the context alive.
pub struct Device(pub(crate) __gl::Gl, Option<Box<DebugCallback>>);

///
pub type DebugCallback = fn(DebugReport, DebugSource, DebugType, u32, &str);

impl Device {
    /// Create a new device from an existing context.
    ///
    /// The context must be initialized with GL 4.5+ core profile.
    /// The passed `loader` is used to obtain the function pointers from the context.
    pub fn new<F>(loader: F, debug: Debug<DebugCallback>) -> Self
    where
        F: FnMut(&str) -> *const c_void,
    {
        let ctxt = __gl::Gl::load_with(loader);

        let cb = match debug {
            Debug::Enable { callback, flags } => unsafe {
                extern "system" fn callback_ffi(
                    source: GLenum,
                    gltype: GLenum,
                    id: GLuint,
                    severity: GLenum,
                    _length: GLsizei,
                    message: *const GLchar,
                    user_param: *mut c_void,
                ) {
                    unsafe {
                        let cb = Box::from_raw(user_param as *mut DebugCallback);
                        let msg = ffi::CStr::from_ptr(message).to_str().unwrap();
                        cb(
                            mem::transmute(severity),
                            mem::transmute(source),
                            mem::transmute(gltype),
                            id,
                            msg,
                        );
                        Box::into_raw(cb);
                    }
                }

                // TODO: flags

                let cb = Box::new(callback);
                let cb_raw = Box::into_raw(cb);
                ctxt.Enable(__gl::DEBUG_OUTPUT);
                ctxt.DebugMessageCallback(callback_ffi, cb_raw as *mut _);
                Some(Box::from_raw(cb_raw))
            },
            Debug::Disable => unsafe {
                ctxt.Disable(__gl::DEBUG_OUTPUT);
                None
            },
        };

        unsafe {
            // Enforce sRGB frmaebuffer handling
            ctxt.Enable(__gl::FRAMEBUFFER_SRGB);
            // Enforce lower-left window coordinate system with [0; 1] depth range
            ctxt.ClipControl(__gl::LOWER_LEFT, __gl::ZERO_TO_ONE);
            // Always enable scissor testing
            ctxt.Enable(__gl::SCISSOR_TEST);
            ctxt.Enable(__gl::TEXTURE_CUBE_MAP_SEAMLESS);
            ctxt.Enable(__gl::PROGRAM_POINT_SIZE);
        }

        Device(ctxt, cb)
    }
}
