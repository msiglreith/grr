use crate::{
    BaseFormat, Buffer, BufferRange, Device, Extent, Format, FormatLayout, Image, Offset, Region,
    SubresourceLayers, __gl,
};

/// Specifies the layout of the host or buffer memory.
#[derive(Debug, Copy, Clone)]
pub struct MemoryLayout {
    ///
    pub base_format: BaseFormat,
    ///
    pub format_layout: FormatLayout,
    ///
    pub row_length: u32,
    ///
    pub image_height: u32,
    ///
    pub alignment: u32,
}

#[derive(Debug, Clone)]
pub struct ImageCopy {
    /// Layers of the source image.
    pub src_subresource: SubresourceLayers,
    /// Initial x,y,z texel offsets in the subregion of the source image.
    pub src_offset: Offset,
    /// Layers of the destination image.
    pub dst_subresource: SubresourceLayers,
    /// Initial x,y,z texel offsets in the subregion of the destination image.
    pub dst_offset: Offset,
    /// Size of texels to copy in the subregion of of the source/destination image.
    pub extent: Extent,
}

#[derive(Debug, Clone)]
pub struct BufferImageCopy {
    /// Offset in bytes from the start of the source/destination buffer.
    pub buffer_offset: u64,
    /// Layout of the source/destination buffer.
    pub buffer_layout: MemoryLayout,
    /// Layers of the source/destination image.
    pub image_subresource: SubresourceLayers,
    /// Initial x,y,z texel offsets in the subregion of the source/destination image.
    pub image_offset: Offset,
    /// Size of texels to copy in the subregion of of the source/destination image.
    pub image_extent: Extent,
}

#[derive(Debug, Clone)]
pub struct HostImageCopy {
    /// Layout of the source/destination host memory.
    pub host_layout: MemoryLayout,
    /// Layers of the source/destination image.
    pub image_subresource: SubresourceLayers,
    /// Initial x,y,z texel offsets in the subregion of the source/destination image.
    pub image_offset: Offset,
    /// Size of texels to copy in the subregion of of the source/destination image.
    pub image_extent: Extent,
}

impl Device {
    pub(crate) unsafe fn set_pixel_unpack_params(&self, layout: &MemoryLayout) {
        self.0
            .PixelStorei(__gl::UNPACK_ALIGNMENT, layout.alignment as _);
        self.0
            .PixelStorei(__gl::UNPACK_IMAGE_HEIGHT, layout.image_height as _);
        self.0
            .PixelStorei(__gl::UNPACK_ROW_LENGTH, layout.row_length as _);
    }

    pub(crate) unsafe fn set_pixel_pack_params(&self, layout: &MemoryLayout) {
        self.0
            .PixelStorei(__gl::PACK_ALIGNMENT, layout.alignment as _);
        self.0
            .PixelStorei(__gl::PACK_IMAGE_HEIGHT, layout.image_height as _);
        self.0
            .PixelStorei(__gl::PACK_ROW_LENGTH, layout.row_length as _);
    }

    /// Copy image data from a location to device memory.
    ///
    /// Location can be either a buffer or a host memory, depending on
    /// whether or not pixel_unpack_buffer is bound.
    unsafe fn copy_to_image(
        &self,
        image: Image,
        subresource: SubresourceLayers,
        offset: Offset,
        extent: Extent,
        data_ptr: *const __gl::types::GLvoid,
        layout: MemoryLayout,
    ) {
        self.set_pixel_unpack_params(&layout);
        match image.target {
            __gl::TEXTURE_1D if subresource.layers == (0..1) => self.0.TextureSubImage1D(
                image.raw,
                subresource.level as _,
                offset.x,
                extent.width as _,
                layout.base_format as _,
                layout.format_layout as _,
                data_ptr,
            ),
            __gl::TEXTURE_1D_ARRAY => self.0.TextureSubImage2D(
                image.raw,
                subresource.level as _,
                offset.x,
                subresource.layers.start as _,
                extent.width as _,
                (subresource.layers.end - subresource.layers.start) as _,
                layout.base_format as _,
                layout.format_layout as _,
                data_ptr,
            ),
            __gl::TEXTURE_2D if subresource.layers == (0..1) => self.0.TextureSubImage2D(
                image.raw,
                subresource.level as _,
                offset.x,
                offset.y,
                extent.width as _,
                extent.height as _,
                layout.base_format as _,
                layout.format_layout as _,
                data_ptr,
            ),
            __gl::TEXTURE_2D_ARRAY => self.0.TextureSubImage3D(
                image.raw,
                subresource.level as _,
                offset.x,
                offset.y,
                subresource.layers.start as _,
                extent.width as _,
                extent.height as _,
                (subresource.layers.end - subresource.layers.start) as _,
                layout.base_format as _,
                layout.format_layout as _,
                data_ptr,
            ),
            __gl::TEXTURE_3D if subresource.layers == (0..1) => self.0.TextureSubImage3D(
                image.raw,
                subresource.level as _,
                offset.x,
                offset.y,
                offset.z,
                extent.width as _,
                extent.height as _,
                extent.depth as _,
                layout.base_format as _,
                layout.format_layout as _,
                data_ptr,
            ),
            _ => unimplemented!(), // panic!("Invalid target image: {}", image.target),
        }
    }

    /// Copy image data from host memory to device memory.
    pub unsafe fn copy_host_to_image<T>(
        &self,
        src_host: &[T],
        dst_image: Image,
        region: HostImageCopy,
    ) {
        self.unbind_pixel_unpack_buffer();
        self.copy_to_image(
            dst_image,
            region.image_subresource,
            region.image_offset,
            region.image_extent,
            src_host.as_ptr() as *const _,
            region.host_layout,
        );
    }

    /// Copy image data from buffer to device memory.
    pub unsafe fn copy_buffer_to_image(
        &self,
        src_buffer: Buffer,
        dst_image: Image,
        region: BufferImageCopy,
    ) {
        self.bind_pixel_unpack_buffer(src_buffer);
        self.copy_to_image(
            dst_image,
            region.image_subresource,
            region.image_offset,
            region.image_extent,
            region.buffer_offset as *const _,
            region.buffer_layout,
        );
    }

    unsafe fn map_subresource_region(
        image: Image,
        subresource: &SubresourceLayers,
        offset: Offset,
        extent: Extent,
    ) -> (Offset, Extent) {
        match image.target {
            __gl::TEXTURE_1D => (
                Offset {
                    x: offset.x,
                    y: 0,
                    z: 0,
                },
                Extent {
                    width: extent.width,
                    height: 1,
                    depth: 1,
                },
            ),
            __gl::TEXTURE_1D_ARRAY => (
                Offset {
                    x: offset.x,
                    y: subresource.layers.start as _,
                    z: 0,
                },
                Extent {
                    width: extent.width,
                    height: (subresource.layers.end - subresource.layers.start) as _,
                    depth: 1,
                },
            ),
            __gl::TEXTURE_2D => (
                Offset {
                    x: offset.x,
                    y: offset.y,
                    z: 0,
                },
                Extent {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
            ),
            __gl::TEXTURE_2D_ARRAY => (
                Offset {
                    x: offset.x,
                    y: offset.y,
                    z: subresource.layers.start as _,
                },
                Extent {
                    width: extent.width,
                    height: extent.height,
                    depth: (subresource.layers.end - subresource.layers.start) as _,
                },
            ),
            __gl::TEXTURE_3D => (offset, extent),
            _ => {
                // todo
                unimplemented!(
                    "Cannot copy from image for multisample, cube array, or buffer textures"
                );
            }
        }
    }

    unsafe fn copy_image_to(
        &self,
        image: Image,
        subresource: SubresourceLayers,
        offset: Offset,
        extent: Extent,
        layout: MemoryLayout,
        (buf_size, buf_ptr): (u32, *mut __gl::types::GLvoid),
    ) {
        self.set_pixel_pack_params(&layout);
        let (offset, extent) = Self::map_subresource_region(image, &subresource, offset, extent);
        self.0.GetTextureSubImage(
            image.raw,
            subresource.level as _,
            offset.x,
            offset.y,
            offset.z,
            extent.width as _,
            extent.height as _,
            extent.depth as _,
            layout.base_format as _,
            layout.format_layout as _,
            buf_size as _,
            buf_ptr,
        );
    }

    /// Copy image data from device memory to a host array.
    pub unsafe fn copy_image_to_host<T>(
        &self,
        src_image: Image,
        dst_host: &mut [T],
        region: HostImageCopy,
    ) {
        self.unbind_pixel_pack_buffer();
        self.copy_image_to(
            src_image,
            region.image_subresource,
            region.image_offset,
            region.image_extent,
            region.host_layout,
            (
                (dst_host.len() * std::mem::size_of::<T>()) as _,
                dst_host.as_mut_ptr() as _,
            ),
        );
    }

    /// Copy image data from device memory to a buffer object.
    pub unsafe fn copy_image_to_buffer(
        &self,
        src_image: Image,
        dst_buffer: Buffer,
        region: BufferImageCopy,
    ) {
        self.bind_pixel_pack_buffer(dst_buffer);
        let buffer_size = self.get_buffer_size(dst_buffer) - region.buffer_offset;
        self.copy_image_to(
            src_image,
            region.image_subresource,
            region.image_offset,
            region.image_extent,
            region.buffer_layout,
            (buffer_size as _, region.buffer_offset as _),
        );
    }

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
        layout: MemoryLayout,
        data: &mut [T],
    ) {
        self.set_pixel_pack_params(&layout);
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
        layout: MemoryLayout,
        buffer_range: BufferRange,
    ) {
        self.set_pixel_pack_params(&layout);
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

    pub unsafe fn copy_image(&self, src_image: Image, dst_image: Image, region: ImageCopy) {
        let (src_offset, _) = Self::map_subresource_region(
            src_image,
            &region.src_subresource,
            region.src_offset,
            region.extent,
        );
        let (dst_offset, extent) = Self::map_subresource_region(
            dst_image,
            &region.dst_subresource,
            region.dst_offset,
            region.extent,
        );
        self.0.CopyImageSubData(
            src_image.raw,
            src_image.target,
            region.src_subresource.level as _,
            src_offset.x,
            src_offset.y,
            src_offset.z,
            dst_image.raw,
            dst_image.target,
            region.dst_subresource.level as _,
            dst_offset.x,
            dst_offset.y,
            dst_offset.z,
            extent.width as _,
            extent.height as _,
            extent.depth as _,
        );
    }

    /// Copy data from one buffer into another buffer.
    ///
    /// # Valid usage
    ///
    /// - `src_buffer` and `dst_buffer` **must** be valid handles.
    /// - `src_offset` **must** be less than the size of `src_buffer`.
    /// - `dst_offset` **must** be less than the size of `dst_buffer`.
    /// - `size` **must** be less than or equal the size of `src_buffer` minus `src_offset`.
    /// - `size` **must** be less than or equal the size of `dst_buffer` minus `dst_offset`.
    /// - The source and destination region **must** not overlap in memory.
    pub unsafe fn copy_buffer(
        &self,
        src_buffer: Buffer,
        src_offset: u64,
        dst_buffer: Buffer,
        dst_offset: u64,
        size: u64,
    ) {
        self.0.CopyNamedBufferSubData(
            src_buffer.0,
            dst_buffer.0,
            src_offset as _,
            dst_offset as _,
            size as _,
        );
    }

    /// Fill buffer with data.
    pub unsafe fn fill_buffer(
        &self,
        buffer: BufferRange,
        buffer_format: Format,
        base_format: BaseFormat,
        format_layout: FormatLayout,
        value: &[u8],
    ) {
        self.0.ClearNamedBufferSubData(
            buffer.buffer.0,
            buffer_format as _,
            buffer.offset as _,
            buffer.size as _,
            format_layout as _,
            base_format as _,
            value.as_ptr() as *const _,
        );
    }
}
