
use __gl;

use std::os::raw::c_void;

/// Logical device, representation one or multiple physical devices (hardware or software).
///
/// This wraps an existing GL context and acts as the main API interface.
/// It's the responsibility of the user to keep the context alive.
pub struct Device(pub(crate) __gl::Gl);

impl Device {
    /// Create a new device from an existing context.
    ///
    /// The context must be initialized with GL 4.5+ core.
    /// The passed `loader` is used to obtain the function pointers from the context.
    pub fn new<F>(loader: F) -> Self
    where
        F: FnMut(&str) -> *const c_void,
    {
        let ctxt = __gl::Gl::load_with(loader);

        // Enforce sRGB frmaebuffer handling
        unsafe { ctxt.Enable(__gl::FRAMEBUFFER_SRGB); }
        // Enforce lower-left window coordinate system with [0; 1] depth range
        unsafe { ctxt.ClipControl(__gl::LOWER_LEFT, __gl::ZERO_TO_ONE); }
        // Always enable scissor testing
        unsafe { ctxt.Enable(__gl::SCISSOR_TEST); }

        Device(ctxt)
    }
}
