/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use __gl;
use __gl::types::{GLchar, GLenum, GLsizei, GLuint};

use std::os::raw::c_void;
use std::{ffi, mem};

use debug::{self, DebugCallback, DebugReport};

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
                ctxt.DebugMessageControl(
                    __gl::DONT_CARE,
                    __gl::DONT_CARE,
                    __gl::DONT_CARE,
                    0,
                    std::ptr::null(),
                    __gl::FALSE,
                );
                debug::set_debug_message_control(
                    &ctxt,
                    true,
                    debug::Filter::All,
                    debug::Filter::All,
                    flags,
                    None,
                );
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
            ctxt.Enable(__gl::SAMPLE_MASK);
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
            max_viewports: self.get_u32(__gl::MAX_VIEWPORTS, None),
            max_framebuffer_width: self.get_u32(__gl::MAX_FRAMEBUFFER_WIDTH, None),
            max_framebuffer_height: self.get_u32(__gl::MAX_FRAMEBUFFER_HEIGHT, None),
            max_framebuffer_layers: self.get_u32(__gl::MAX_FRAMEBUFFER_LAYERS, None),
            max_color_attachments: self.get_u32(__gl::MAX_COLOR_ATTACHMENTS, None),
            max_viewport_dimensions: [
                self.get_u32(__gl::MAX_VIEWPORT_DIMS, Some(0)),
                self.get_u32(__gl::MAX_VIEWPORT_DIMS, Some(1)),
            ],
            max_vertex_input_attributes: self.get_u32(__gl::MAX_VERTEX_ATTRIBS, None),
            max_vertex_input_bindings: self.get_u32(__gl::MAX_VERTEX_ATTRIB_BINDINGS, None),
            max_vertex_input_attribute_offset: self
                .get_u32(__gl::MAX_VERTEX_ATTRIB_RELATIVE_OFFSET, None),
            max_vertex_input_binding_stride: self.get_u32(__gl::MAX_VERTEX_ATTRIB_STRIDE, None),
            max_vertex_output_components: self.get_u32(__gl::MAX_VERTEX_OUTPUT_COMPONENTS, None),
        }
    }

    pub fn features(&self) -> DeviceFeatures {
        DeviceFeatures {}
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

    pub max_viewports: u32,

    pub max_viewport_dimensions: [u32; 2],

    pub max_framebuffer_width: u32,

    pub max_framebuffer_height: u32,

    pub max_framebuffer_layers: u32,

    pub max_color_attachments: u32,

    pub max_vertex_input_attributes: u32,

    pub max_vertex_input_bindings: u32,

    pub max_vertex_input_attribute_offset: u32,

    pub max_vertex_input_binding_stride: u32,

    pub max_vertex_output_components: u32,
}

#[derive(Clone, Debug)]
pub struct DeviceFeatures {}
