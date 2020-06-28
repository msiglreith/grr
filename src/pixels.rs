//! Read/write pixel information directly
use crate::buffer::BufferRange;
use crate::device::Device;
use crate::image::SubresourceLayout;
use crate::{Region, __gl};

impl Device {
    /// Read a region of pixel data from the current read framebuffer
    /// into the provided storage slice.
    ///
    /// # Remarks:
    ///
    /// The transfer in `copy_attachement_to_host` is done
    /// synchronously; the method won't return until the transfer is
    /// complete.
    ///
    /// # See also
    ///
    /// * [copy_attachement_to_buffer](struct.Device.html#method.copy_attachment_to_buffer)
    /// for an asynchronous alternative.
    pub unsafe fn copy_attachment_to_host<T: Sized>(
        &self,
        region: Region,
        layout: SubresourceLayout,
        data: &mut [T],
    ) {
        self.0
            .PixelStorei(__gl::PACK_ALIGNMENT, layout.alignment as _);
        self.unbind_pixel_pack_buffer();
        self.0.ReadnPixels(
            region.x,
            region.y,
            region.w as _,
            region.h as _,
            layout.base_format as _,
            layout.format_layout as _,
            (data.len() * std::mem::size_of::<T>()) as _,
            data.as_mut_ptr() as _,
        );
    }

    /// Read a region of pixel data from the current read framebuffer
    /// into a buffer object.
    ///
    /// # Remarks:
    ///
    /// The transfer for `copy_attachment_to_buffer` is asynchronous.
    pub unsafe fn copy_attachment_to_buffer(
        &self,
        region: Region,
        layout: SubresourceLayout,
        buffer_range: BufferRange,
    ) {
        self.0
            .PixelStorei(__gl::PACK_ALIGNMENT, layout.alignment as _);
        self.bind_pixel_pack_buffer(buffer_range.buffer);
        self.0.ReadnPixels(
            region.x,
            region.y,
            region.w as _,
            region.h as _,
            layout.base_format as _,
            layout.format_layout as _,
            buffer_range.size as _,
            buffer_range.offset as _,
        );
    }
}
