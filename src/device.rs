use __gl;
use __gl::types::{GLchar, GLenum, GLsizei, GLuint};

use std::os::raw::c_void;
use std::{ffi, mem};

use debug::{DebugReport, DebugCallback};

/// Logical device, representation one or multiple physical devices (hardware or software).
///
/// This wraps an existing GL context and acts as the main API interface.
/// It's the responsibility of the user to keep the context alive.
pub struct Device(pub(crate) __gl::Gl, Option<Box<DebugCallback>>);

/// Device debug control.
pub enum Debug<F> {
    Enable { callback: F, flags: DebugReport },
    Disable,
}

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

    pub fn limits(&self) -> DeviceLimits {
        DeviceLimits {
            max_compute_work_group_invocations: self
                .get_u32(__gl::MAX_COMPUTE_WORK_GROUP_INVOCATIONS, None),
            max_compute_work_group_count: [
                self.get_u32(__gl::MAX_COMPUTE_WORK_GROUP_COUNT, Some(0)),
                self.get_u32(__gl::MAX_COMPUTE_WORK_GROUP_COUNT, Some(1)),
                self.get_u32(__gl::MAX_COMPUTE_WORK_GROUP_COUNT, Some(2)),
            ],
            max_compute_work_group_size: [
                self.get_u32(__gl::MAX_COMPUTE_WORK_GROUP_SIZE, Some(0)),
                self.get_u32(__gl::MAX_COMPUTE_WORK_GROUP_SIZE, Some(1)),
                self.get_u32(__gl::MAX_COMPUTE_WORK_GROUP_SIZE, Some(2)),
            ],
            max_compute_shared_memory_size: self
                .get_u32(__gl::MAX_COMPUTE_SHARED_MEMORY_SIZE, None),
            max_clip_distances: self.get_u32(__gl::MAX_CLIP_DISTANCES, None),
            max_cull_distances: self.get_u32(__gl::MAX_CULL_DISTANCES, None),
        }
    }

    fn get_u32(&self, target: GLenum, index: Option<usize>) -> u32 {
        self.get_i32(target, index) as _
    }

    fn get_i32(&self, target: GLenum, index: Option<usize>) -> i32 {
        let mut value = 0;
        unsafe {
            match index {
                Some(i) => self.0.GetIntegeri_v(target, i as _, &mut value),
                None => self.0.GetIntegerv(target, &mut value),
            }
        }
        value
    }
}

#[derive(Clone, Debug)]
pub struct DeviceLimits {
    /// Maximum number of total invocations in a single workgroup.
    pub max_compute_work_group_invocations: u32,
    /// Maximum number of local workgroups per dispatch call.
    pub max_compute_work_group_count: [u32; 3],
    /// Maximum size of a local workgroup in each dimensions.
    pub max_compute_work_group_size: [u32; 3],
    /// Maximum size in bytes of all shared memory variables in the compute pipeline.
    pub max_compute_shared_memory_size: u32,
    /// Maximum number of clip distances in a shader stage.
    ///
    /// Minimum value: 8
    pub max_clip_distances: u32,
    /// Maximum number of cull distances in a shader stage.
    ///
    /// Minimum value: 8
    pub max_cull_distances: u32,
}
