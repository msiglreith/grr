
use __gl;

use device::Device;

///
pub struct Buffer(GLuint, GLbitfield);

impl Device {
    /// Create a new empty buffer.
    ///
    /// # Parameters
    ///
    /// - `size`: Length in bytes of the associated storage memory.
    /// - `memory`: Properties of the internal memory slice. Indicating the usage
    ///             and locality of the allocation.
    pub fn create_buffer(&self, size: u64, memory: MemoryFlags) -> Buffer {
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
            self.0.NamedBufferStorage(buffer, size as _, ptr::null(), flags);
        }

        Buffer(buffer, flags)
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
    pub fn map_buffer<T>(&self, buffer: &Buffer, range: Range<u64>, mapping: MappingFlags) -> &mut [T] {
        let len = range.end - range.start;
        let stride = std::mem::size_of::<T>();
        assert_eq!(len % stride as u64, 0);

        let mut flags = 0;

        if mapping.contains(MappingFlags::UNSYNCHRONIZED) {
            flags |= __gl::MAP_UNSYNCHRONIZED_BIT;
        }
        flags |= buffer.1 & (
            __gl::MAP_COHERENT_BIT |
            __gl::MAP_PERSISTENT_BIT |
            __gl::MAP_READ_BIT |
            __gl::MAP_WRITE_BIT
        );

        let stride = std::mem::size_of::<T>();

        let ptr = unsafe {
            self.0.MapNamedBufferRange(
                buffer.0,
                range.start as _,
                len as _,
                flags,
            ) as *mut _
        };

        unsafe { std::slice::from_raw_parts_mut(ptr, len as usize / stride) }
    }

    /// Unmap a buffer from virtual host memory.
    ///
    /// # Valid usage
    ///
    /// - The buffer must be currently mapped.
    pub fn unmap_buffer(&self, buffer: &Buffer) -> bool {
        unsafe { self.0.UnmapNamedBuffer(buffer.0) != 0 }
    }

    /// Delete a buffer.
    pub fn delete_buffer(&self, buffer: Buffer) {
        unsafe { self.0.DeleteBuffers(1, &buffer.0); }
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

        ///
        const DYNAMIC = 0x10;
    }
);

bitflags!(
    /// Memory mapping flags.
    pub struct MappingFlags: u8 {
        /// Driver won't synchronize memory access.
        const UNSYNCHRONIZED = 0x1;
    }
);

