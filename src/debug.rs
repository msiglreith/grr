/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use __gl;
use __gl::types::{GLenum, GLuint};
use device::Device;

/// Message filter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Filter<T> {
    /// Referencing all values of the type `T`.
    All,
    ///
    Some(T),
}

impl Filter<DebugSource> {
    fn as_gl(self) -> GLenum {
        match self {
            Filter::All => __gl::DONT_CARE,
            Filter::Some(v) => v as _,
        }
    }
}

impl Filter<DebugType> {
    fn as_gl(self) -> GLenum {
        match self {
            Filter::All => __gl::DONT_CARE,
            Filter::Some(v) => v as _,
        }
    }
}

bitflags!(
    /// Debug report flags.
    ///
    /// Denotes which events will trigger a debug report.
    pub struct DebugReport: GLenum {
        const NOTIFICATION = __gl::DEBUG_SEVERITY_NOTIFICATION;
        const WARNING = __gl::DEBUG_SEVERITY_MEDIUM;
        const ERROR = __gl::DEBUG_SEVERITY_HIGH;
        const PERFORMANCE_WARNING = __gl::DEBUG_SEVERITY_LOW;
    }
);

/// Debug message source.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

pub(crate) fn set_debug_message_control(
    ctxt: &__gl::Gl,
    enable: bool,
    src: Filter<DebugSource>,
    ty: Filter<DebugType>,
    flags: DebugReport,
    ids: Option<&[u32]>,
) {
    let src = src.as_gl();
    let ty = ty.as_gl();
    let num_ids = match ids {
        Some(ids) => ids.len() as i32,
        None => 0,
    };
    let id_ptr = match ids {
        Some(ids) => ids.as_ptr(),
        None => std::ptr::null(),
    };
    let enable = if enable { __gl::TRUE } else { __gl::FALSE };

    unsafe {
        if flags.contains(DebugReport::NOTIFICATION) {
            ctxt.DebugMessageControl(
                src,
                ty,
                DebugReport::NOTIFICATION.bits(),
                num_ids,
                id_ptr,
                enable,
            );
        }
        if flags.contains(DebugReport::WARNING) {
            ctxt.DebugMessageControl(
                src,
                ty,
                DebugReport::WARNING.bits(),
                num_ids,
                id_ptr,
                enable,
            );
        }
        if flags.contains(DebugReport::ERROR) {
            ctxt.DebugMessageControl(src, ty, DebugReport::ERROR.bits(), num_ids, id_ptr, enable);
        }
        if flags.contains(DebugReport::PERFORMANCE_WARNING) {
            ctxt.DebugMessageControl(
                src,
                ty,
                DebugReport::PERFORMANCE_WARNING.bits(),
                num_ids,
                id_ptr,
                enable,
            );
        }
    }
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

    pub fn enable_debug_message(
        &self,
        src: Filter<DebugSource>,
        ty: Filter<DebugType>,
        flags: DebugReport,
        ids: Option<&[u32]>,
    ) {
        set_debug_message_control(&self.0, true, src, ty, flags, ids);
    }

    pub fn disable_debug_message(
        &self,
        src: Filter<DebugSource>,
        ty: Filter<DebugType>,
        flags: DebugReport,
        ids: Option<&[u32]>,
    ) {
        set_debug_message_control(&self.0, false, src, ty, flags, ids);
    }

    pub fn begin_debug_marker(&self, src: DebugSource, id: u32, msg: &str) {
        unsafe {
            self.0
                .PushDebugGroup(src as _, id, msg.len() as _, msg.as_ptr() as *const _);
        }
    }

    pub fn end_debug_marker(&self) {
        unsafe {
            self.0.PopDebugGroup();
        }
    }
}
