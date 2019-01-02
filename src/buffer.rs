//! Buffer

use __gl;
use __gl::types::{GLbitfield, GLuint};

use std::ops::Range;
use std::{mem, ptr, slice};

use debug::{Object, ObjectType};
use device::Device;
use error::Result;
use format::{BaseFormat, Format, FormatLayout};

///
pub struct Buffer(pub(crate) GLuint, GLbitfield);

impl Object for Buffer {
    const TYPE: ObjectType = ObjectType::Buffer;
    fn handle(&self) -> GLuint {
        self.0
    }
}

/// Buffer Range.
///
/// Specifies a subrange of a buffer resource.
pub struct BufferRange<'a> {
    pub buffer: &'a Buffer,
    pub offset: usize,
    pub size: usize,
}

impl Device {
    fn create_buffer_impl(
        &self,
        size: isize,
        data_ptr: *const (),
        memory: MemoryFlags,
    ) -> Result<Buffer> {
        let flags = {
            let mut flags = 0;
            if !memory.contains(MemoryFlags::DEVICE_LOCAL) {
                flags |= __gl::CLIENT_STORAGE_BIT;
            }
            if memory.contains(MemoryFlags::COHERENT) {
                flags |= __gl::MAP_COHERENT_BIT | __gl::MAP_PERSISTENT_BIT;
            }
            if memory.contains(MemoryFlags::CPU_MAP_READ) {
                flags |= __gl::MAP_READ_BIT | __gl::MAP_PERSISTENT_BIT;
            }
            if memory.contains(MemoryFlags::CPU_MAP_WRITE) {
                flags |= __gl::MAP_WRITE_BIT | __gl::MAP_PERSISTENT_BIT;
            }
            if memory.contains(MemoryFlags::DYNAMIC) {
                flags |= __gl::DYNAMIC_STORAGE_BIT;
            }
            flags
        };

        let mut buffer = 0;
        unsafe {
            self.0.CreateBuffers(1, &mut buffer);
            self.get_error()?;
            self.0
                .NamedBufferStorage(buffer, size, data_ptr as *const _, flags);
            self.get_error()?;
        }

        Ok(Buffer(buffer, flags))
    }

    /// Create a new empty buffer.
    ///
    /// # Parameters
    ///
    /// - `size`: Length in bytes of the associated storage memory.
    /// - `memory`: Properties of the internal memory slice. Indicating the usage
    ///             and locality of the allocation.
    pub fn create_buffer(&self, size: u64, memory: MemoryFlags) -> Result<Buffer> {
        self.create_buffer_impl(size as _, ptr::null(), memory)
    }

    /// Create a new buffer from host memory data.
    ///
    /// # Parameters
    ///
    /// - `data`: Host data, which will copied into the buffer on creation.
    /// - `memory`: Properties of the internal memory slice. Indicating the usage
    ///             and locality of the allocation.
    pub fn create_buffer_from_host(&self, data: &[u8], memory: MemoryFlags) -> Result<Buffer> {
        self.create_buffer_impl(data.len() as _, data.as_ptr() as *const _, memory)
    }

    /// Persistently map memory to host accessible virtual memory.
    ///
    /// # Valid usage
    ///
    /// - `range.end` may not be larger than the size of the buffer.
    /// - `range.start` must be smaller than `range.end`
    /// - `buffer` must be created with `CPU_MAP_READ` or `CPU_MAP_WRITE` flags.
    /// - `range.end - range.start` must be a multiple of the size of `T`
    /// - If the buffer has not been created with `CPU_MAP_READ` the host should
    ///   not read from the returned slice.
    /// - If the buffer has not been created with `CPU_MAP_WRITE` the host should
    ///   not write to the returned slice.
    /// - A buffer can not be mapped multiple times.
    ///
    /// # Return
    ///
    /// Returns a typed slice of the mapped memory range.
    pub fn map_buffer<T>(
        &self,
        buffer: &Buffer,
        range: Range<u64>,
        mapping: MappingFlags,
    ) -> &mut [T] {
        let len = range.end - range.start;
        let stride = mem::size_of::<T>();
        assert_eq!(len % stride as u64, 0);

        let mut flags = 0;

        if mapping.contains(MappingFlags::UNSYNCHRONIZED) {
            flags |= __gl::MAP_UNSYNCHRONIZED_BIT;
        }
        flags |= buffer.1
            & (__gl::MAP_COHERENT_BIT
                | __gl::MAP_PERSISTENT_BIT
                | __gl::MAP_READ_BIT
                | __gl::MAP_WRITE_BIT);

        let stride = mem::size_of::<T>();

        let ptr = unsafe {
            self.0
                .MapNamedBufferRange(buffer.0, range.start as _, len as _, flags)
                as *mut _
        };

        unsafe { slice::from_raw_parts_mut(ptr, len as usize / stride) }
    }

    /// Unmap a buffer from virtual host memory.
    ///
    /// # Valid usage
    ///
    /// - The buffer must be currently mapped.
    ///
    /// # Return
    ///
    /// Returns if the unmapping operation was successfull.
    pub fn unmap_buffer(&self, buffer: &Buffer) -> bool {
        unsafe { self.0.UnmapNamedBuffer(buffer.0) != 0 }
    }

    /// Delete a buffer.
    pub fn delete_buffer(&self, buffer: Buffer) {
        self.delete_buffers(&[buffer]);
    }

    /// Delete multiple buffers.
    pub fn delete_buffers(&self, buffers: &[Buffer]) {
        let buffers = buffers.iter().map(|buffer| buffer.0).collect::<Vec<_>>();
        unsafe {
            self.0.DeleteBuffers(buffers.len() as _, buffers.as_ptr());
        }
    }

    /// Copy memory from the host into the buffer memory.
    pub fn copy_host_to_buffer(&self, buffer: &Buffer, offset: isize, data: &[u8]) {
        unsafe {
            self.0
                .NamedBufferSubData(buffer.0, offset, data.len() as _, data.as_ptr() as *const _);
        }
    }

    /// Copy data from one buffer into another buffer.
    pub fn copy_buffer(
        &self,
        src_buffer: &Buffer,
        src_offset: isize,
        dst_buffer: &Buffer,
        dst_offset: isize,
        size: u64,
    ) {
        unsafe {
            self.0.CopyNamedBufferSubData(
                src_buffer.0,
                dst_buffer.0,
                src_offset,
                dst_offset,
                size as _,
            );
        }
    }

    /// Fill buffer with data.
    pub fn fill_buffer(
        &self,
        buffer: BufferRange,
        buffer_format: Format,
        base_format: BaseFormat,
        format_layout: FormatLayout,
        value: &[u8],
    ) {
        unsafe {
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

    /// Bind buffer ranges as uniform buffers.
    ///
    /// Shader can access the buffer memory as readonly.
    pub fn bind_uniform_buffers(&self, first: u32, ranges: &[BufferRange]) {
        let buffers = ranges.iter().map(|view| view.buffer.0).collect::<Vec<_>>();
        let offsets = ranges
            .iter()
            .map(|view| view.offset as _)
            .collect::<Vec<_>>();
        let sizes = ranges.iter().map(|view| view.size as _).collect::<Vec<_>>();

        unsafe {
            self.0.BindBuffersRange(
                __gl::UNIFORM_BUFFER,
                first,
                ranges.len() as _,
                buffers.as_ptr(),
                offsets.as_ptr(),
                sizes.as_ptr(),
            );
        }
    }

    /// Bind buffer ranges as shader storage buffers.
    ///
    /// Shaders can access the buffer memory as readwrite.
    pub fn bind_shader_storage_buffers(&self, first: u32, ranges: &[BufferRange]) {
        let buffers = ranges.iter().map(|view| view.buffer.0).collect::<Vec<_>>();
        let offsets = ranges
            .iter()
            .map(|view| view.offset as _)
            .collect::<Vec<_>>();
        let sizes = ranges.iter().map(|view| view.size as _).collect::<Vec<_>>();

        unsafe {
            self.0.BindBuffersRange(
                __gl::SHADER_STORAGE_BUFFER,
                first,
                ranges.len() as _,
                buffers.as_ptr(),
                offsets.as_ptr(),
                sizes.as_ptr(),
            );
        }
    }

    /// Bind indirect buffer for draw commands.
    pub fn bind_draw_indirect_buffer(&self, buffer: &Buffer) {
        unsafe {
            self.0.BindBuffer(__gl::DRAW_INDIRECT_BUFFER, buffer.0);
        }
    }

    /// Bind indirect buffer for dispatch commands.
    pub fn bind_dispatch_indirect_buffer(&self, buffer: &Buffer) {
        unsafe {
            self.0.BindBuffer(__gl::DRAW_INDIRECT_BUFFER, buffer.0);
        }
    }
}

bitflags!(
    /// Memory property flags.
    pub struct MemoryFlags: u8 {
        /// Device local memory on the GPU.
        const DEVICE_LOCAL = 0x1;

        /// CPU-GPU coherent.
        const COHERENT = 0x2;

        /// CPU can read from mapped memory.
        const CPU_MAP_READ = 0x4;

        /// CPU can write to mapped memory.
        const CPU_MAP_WRITE = 0x8;

        /// Required for copies to buffer from host memory.
        const DYNAMIC = 0x10;
    }
);

bitflags!(
    /// Memory mapping flags.
    pub struct MappingFlags: u8 {
        /// Driver won't synchronize memory access.
        ///
        /// The user needs to manually synchonize access via fences.
        const UNSYNCHRONIZED = 0x1;
    }
);
