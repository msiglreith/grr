//! Read/write pixel information directly
use crate::buffer::BufferRange;
use crate::device::Device;
use crate::image::SubresourceLayout;
use crate::{Region, __gl};

impl Device {
    /// Ready a region of pixel data from the current read framebuffer
    /// into the provided storage slice.
    pub unsafe fn read_pixels<T: Sized>(
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
    pub unsafe fn read_pixels_to_buffer(
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
