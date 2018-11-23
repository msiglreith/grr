use __gl;
use __gl::types::GLuint;

use Region;
use device::Device;

///
pub enum ClearAttachment {
    ColorInt(usize, [i32; 4]),
    ColorUint(usize, [u32; 4]),
    ColorFloat(usize, [f32; 4]),
    Depth(f32),
    Stencil(i32),
    DepthStencil(f32, i32),
}

///
pub enum Attachment {
    Color(usize),
    Depth,
    Stencil,
    DepthStencil,
}

///
pub struct Framebuffer(GLuint);

impl Framebuffer {
    pub const DEFAULT: &'static Self = &Framebuffer(0);
}

///
pub struct Renderbuffer(GLuint);

impl Device {
    ///
    pub fn create_framebuffer(&self) -> Framebuffer {
        let mut framebuffer = 0;
        unsafe {
            self.0.CreateFramebuffers(1, &mut framebuffer);
        }
        self.get_error("CreateFramebuffers");

        Framebuffer(framebuffer)
    }

    ///
    pub fn delete_framebuffer(&self, framebuffer: Framebuffer) {
        unsafe {
            self.0.DeleteFramebuffers(1, &framebuffer.0);
        }
        self.get_error("DeleteFramebuffers");
    }

    ///
    pub fn create_renderbuffer(&self) -> Renderbuffer {
        let mut renderbuffer = 0;
        unsafe {
            self.0.CreateRenderbuffers(1, &mut renderbuffer);
        }
        self.get_error("CreateRenderbuffers");

        Renderbuffer(renderbuffer)
    }

    ///
    pub fn delete_renderbuffer(&self, renderbuffer: Renderbuffer) {
        unsafe {
            self.0.DeleteRenderbuffers(1, &renderbuffer.0);
        }
        self.get_error("DeleteRenderbuffers");
    }

    /// Clear framebuffer attachment.
    pub fn clear_attachment(&self, fb: &Framebuffer, cv: ClearAttachment) {
        unsafe {
            match cv {
                ClearAttachment::ColorInt(id, color) => {
                    self.0
                        .ClearNamedFramebufferiv(fb.0, __gl::COLOR, id as _, color.as_ptr());
                    self.get_error("ClearNamedFramebufferiv (Color)");
                }
                ClearAttachment::ColorUint(id, color) => {
                    self.0
                        .ClearNamedFramebufferuiv(fb.0, __gl::COLOR, id as _, color.as_ptr());
                    self.get_error("ClearNamedFramebufferuiv (Color)");
                }
                ClearAttachment::ColorFloat(id, color) => {
                    self.0
                        .ClearNamedFramebufferfv(fb.0, __gl::COLOR, id as _, color.as_ptr());
                    self.get_error("ClearNamedFramebufferfv (Color)");
                }
                ClearAttachment::Depth(depth) => {
                    self.0
                        .ClearNamedFramebufferfv(fb.0, __gl::DEPTH, 0, &depth as *const _);
                    self.get_error("ClearNamedFramebufferfv (Depth)");
                }
                ClearAttachment::Stencil(stencil) => {
                    self.0
                        .ClearNamedFramebufferiv(fb.0, __gl::STENCIL, 0, &stencil as *const _);
                    self.get_error("ClearNamedFramebufferiv (Stencil");
                }
                ClearAttachment::DepthStencil(depth, stencil) => {
                    self.0
                        .ClearNamedFramebufferfi(fb.0, __gl::DEPTH_STENCIL, 0, depth, stencil);
                    self.get_error("ClearNamedFramebufferfi (Depth-Stencil)");
                }
            }
        }
    }

    ///
    pub fn invalidate_framebuffer(
        &self,
        _framebuffer: &Framebuffer,
        _attachments: &[Attachment],
        _region: Region,
    ) {
        unimplemented!()
    }
}
