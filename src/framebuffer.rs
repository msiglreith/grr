//! Framebuffers

use crate::__gl;
use crate::__gl::types::{GLenum, GLuint};

use crate::debug::{Object, ObjectType};
use crate::device::Device;
use crate::error::Result;
use crate::{Format, ImageView, Region};

/// Attachment clearing description.
pub enum ClearAttachment {
    ColorInt(usize, [i32; 4]),
    ColorUint(usize, [u32; 4]),
    ColorFloat(usize, [f32; 4]),
    Depth(f32),
    Stencil(i32),
    DepthStencil(f32, i32),
}

/// Attachment reference.
#[derive(Copy, Clone, Debug)]
pub enum Attachment {
    Color(usize),
    Depth,
    Stencil,
    DepthStencil,
}

impl Attachment {
    fn target(&self) -> GLenum {
        match *self {
            Attachment::Color(slot) => __gl::COLOR_ATTACHMENT0 + slot as u32,
            Attachment::Depth => __gl::DEPTH_ATTACHMENT,
            Attachment::Stencil => __gl::STENCIL_ATTACHMENT,
            Attachment::DepthStencil => __gl::DEPTH_STENCIL_ATTACHMENT,
        }
    }
}

///
pub enum AttachmentView {
    Image(ImageView),
    Renderbuffer(Renderbuffer),
}

/// Framebuffer handle.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Framebuffer(pub(crate) GLuint);

impl Framebuffer {
    /// Default framebuffer handle.
    ///
    /// Thie is the base framebuffer associated with the context.
    /// It also represents the internal swapchain for presentation.
    pub const DEFAULT: Self = Framebuffer(0);
}

impl Object for Framebuffer {
    const TYPE: ObjectType = ObjectType::Framebuffer;
    fn handle(&self) -> GLuint {
        self.0
    }
}

/// Renderbuffer handle.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Renderbuffer(GLuint);

impl Object for Renderbuffer {
    const TYPE: ObjectType = ObjectType::Renderbuffer;
    fn handle(&self) -> GLuint {
        self.0
    }
}

impl Device {
    /// Create a new framebuffer.
    pub unsafe fn create_framebuffer(&self) -> Result<Framebuffer> {
        let mut framebuffer = 0;
        self.0.CreateFramebuffers(1, &mut framebuffer);
        self.get_error()?;

        Ok(Framebuffer(framebuffer))
    }

    /// Delete a framebuffer.
    pub unsafe fn delete_framebuffer(&self, framebuffer: Framebuffer) {
        self.delete_framebuffers(&[framebuffer])
    }

    /// Delete multiple framebuffers.
    pub unsafe fn delete_framebuffers(&self, framebuffers: &[Framebuffer]) {
        self.0.DeleteFramebuffers(
            framebuffers.len() as _,
            framebuffers.as_ptr() as *const _, // newtype
        );
    }

    /// Create a new framebuffer.
    pub unsafe fn create_renderbuffer(
        &self,
        format: Format,
        width: u32,
        height: u32,
        samples: u32,
    ) -> Result<Renderbuffer> {
        let mut renderbuffer = 0;
        self.0.CreateRenderbuffers(1, &mut renderbuffer);
        self.get_error()?;

        if samples > 1 {
            self.0.NamedRenderbufferStorageMultisample(
                renderbuffer,
                samples as _,
                format as _,
                width as _,
                height as _,
            );
        } else {
            self.0
                .NamedRenderbufferStorage(renderbuffer, format as _, width as _, height as _);
        }

        Ok(Renderbuffer(renderbuffer))
    }

    /// Delete a renderbuffer.
    pub unsafe fn delete_renderbuffer(&self, renderbuffer: Renderbuffer) {
        self.delete_renderbuffers(&[renderbuffer])
    }

    /// Delete multiple renderbuffers.
    pub unsafe fn delete_renderbuffers(&self, renderbuffers: &[Renderbuffer]) {
        self.0.DeleteRenderbuffers(
            renderbuffers.len() as _,
            renderbuffers.as_ptr() as *const _, // newtype
        );
    }

    /// Clear framebuffer attachment.
    pub unsafe fn clear_attachment(&self, fb: Framebuffer, cv: ClearAttachment) {
        match cv {
            ClearAttachment::ColorInt(id, color) => {
                self.0
                    .ClearNamedFramebufferiv(fb.0, __gl::COLOR, id as _, color.as_ptr());
            }
            ClearAttachment::ColorUint(id, color) => {
                self.0
                    .ClearNamedFramebufferuiv(fb.0, __gl::COLOR, id as _, color.as_ptr());
            }
            ClearAttachment::ColorFloat(id, color) => {
                self.0
                    .ClearNamedFramebufferfv(fb.0, __gl::COLOR, id as _, color.as_ptr());
            }
            ClearAttachment::Depth(depth) => {
                self.0
                    .ClearNamedFramebufferfv(fb.0, __gl::DEPTH, 0, &depth as *const _);
            }
            ClearAttachment::Stencil(stencil) => {
                self.0
                    .ClearNamedFramebufferiv(fb.0, __gl::STENCIL, 0, &stencil as *const _);
            }
            ClearAttachment::DepthStencil(depth, stencil) => {
                self.0
                    .ClearNamedFramebufferfi(fb.0, __gl::DEPTH_STENCIL, 0, depth, stencil);
            }
        }
    }

    ///
    pub unsafe fn invalidate_attachments(
        &self,
        framebuffer: Framebuffer,
        attachments: &[Attachment],
        region: Region,
    ) {
        let attachments = attachments
            .iter()
            .map(|att| att.target())
            .collect::<Vec<_>>();

        self.0.InvalidateNamedFramebufferSubData(
            framebuffer.0,
            attachments.len() as _,
            attachments.as_ptr(),
            region.x,
            region.y,
            region.w,
            region.h,
        )
    }

    /// Bind a framebuffer for draw commands.
    pub unsafe fn bind_framebuffer(&self, framebuffer: Framebuffer) {
        self.0
            .BindFramebuffer(__gl::DRAW_FRAMEBUFFER, framebuffer.0);
    }

    /// Bind a framebuffer for read commands.
    pub unsafe fn bind_read_framebuffer(&self, framebuffer: Framebuffer) {
        self.0
            .BindFramebuffer(__gl::READ_FRAMEBUFFER, framebuffer.0);
    }

    /// Bind attachments to the framebuffer.
    ///
    /// All previously bound attachments become invalid.
    pub unsafe fn bind_attachments(
        &self,
        framebuffer: Framebuffer,
        attachments: &[(Attachment, AttachmentView)],
    ) {
        assert_ne!(
            framebuffer.0, 0,
            "The default framebuffer can't be changed."
        );

        for (attachment, view) in attachments {
            let target = attachment.target();
            match *view {
                AttachmentView::Image(image) => {
                    self.0
                        .NamedFramebufferTexture(framebuffer.0, target, image.0, 0);
                }
                AttachmentView::Renderbuffer(renderbuffer) => {
                    self.0.NamedFramebufferRenderbuffer(
                        framebuffer.0,
                        target,
                        __gl::RENDERBUFFER,
                        renderbuffer.0,
                    );
                }
            }
        }
    }

    /// Specify color attachments.
    ///
    /// Defines the color render targets for the next draw calls.
    /// This builds the link between fragment outputs in the fragment shader
    /// and attachments bound on the framebuffer.
    pub unsafe fn set_color_attachments(&self, framebuffer: Framebuffer, attachments: &[u32]) {
        assert_ne!(
            framebuffer.0, 0,
            "The default framebuffer can't be changed."
        );

        let attachments = attachments
            .iter()
            .map(|i| i + __gl::COLOR_ATTACHMENT0)
            .collect::<Vec<_>>();
        self.0.NamedFramebufferDrawBuffers(
            framebuffer.0,
            attachments.len() as _,
            attachments.as_ptr(),
        );
    }
}
