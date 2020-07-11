use crate::__gl;

use crate::Device;

bitflags!(
    /// Memory barrier.
    pub struct Barrier: u32 {
        /// Read access to a vertex buffer.
        ///
        /// Bound via `bind_vertex_buffers`, used in drawing commands.
        const VERTEX_ATTRIBUTE_READ = __gl::VERTEX_ATTRIB_ARRAY_BARRIER_BIT;

        /// Read access to an index buffer.
        ///
        /// Bound via `bind_index_buffer`, used in indexed drawing commands.
        const INDEX_READ = __gl::ELEMENT_ARRAY_BARRIER_BIT;

        /// Read access to a uniform buffer.
        ///
        /// Bound via `bind_uniform_buffers`.
        const UNIFORM_READ = __gl::UNIFORM_BARRIER_BIT;

        /// Read access to a sampled image.
        ///
        /// Bound via `bind_image_views`.
        const SAMPLED_IMAGE_READ = __gl::TEXTURE_FETCH_BARRIER_BIT;

        /// Read/Write access to a storage image.
        ///
        /// Bound via `bind_storage_image_views`.
        const STORAGE_IMAGE_RW = __gl::SHADER_IMAGE_ACCESS_BARRIER_BIT;

        /// Read access to an indirect command buffer.
        ///
        /// Bound via `bind_draw_indirect_buffer` or `bind_dispatch_indirect_buffer`.
        const INDIRECT_COMMAND_READ = __gl::COMMAND_BARRIER_BIT;

        /// Read/write access to a buffer in transfer operations to/from images or attachments.
        ///
        /// Used in `copy_attachment_to_buffer` and `copy_buffer_to_image`.
        const BUFFER_IMAGE_TRANSFER_RW = __gl::PIXEL_BUFFER_BARRIER_BIT;

        /// Read/write access to an image in transfer operation.
        const IMAGE_TRANSFER_RW = __gl::TEXTURE_UPDATE_BARRIER_BIT;

        /// Read/write access to an buffer in a transfer operation and mapping operations.
        const BUFFER_TRANSFER_RW = __gl::BUFFER_UPDATE_BARRIER_BIT;

        /// Read/write access to framebuffer attachments.
        const FRAMEBUFFER_RW = __gl::FRAMEBUFFER_BARRIER_BIT;

        ///
        const TRANSFORM_FEEDBACK_WRITE = __gl::TRANSFORM_FEEDBACK_BARRIER_BIT;

        /// Read/write access to atomic counters.
        const ATOMIC_COUNTER_RW = __gl::ATOMIC_COUNTER_BARRIER_BIT;

        /// Read/write access to storage buffers.
        ///
        /// Bound via `bind_storage_buffers`.
        const STORAGE_BUFFER_RW = __gl::SHADER_STORAGE_BARRIER_BIT;

        /// Inserts a image (or texture) barrier to control read/write access
        /// of fragments in subsequent draw calls.
        ///
        /// Image barriers are required to prevent rendering feedback loops
        /// in case of reading texels of an image which is bound to the current
        /// framebuffer as attachment.
        ///
        /// The barrier will ensure that writes to the texel are finished and caches
        /// have been invalidated.
        const INPUT_ATTACHMENT_READ = 0x8000_0000;

        const ALL = __gl::ALL_BARRIER_BITS;
    }
);

bitflags!(
    /// Memory barrier for by-region dependencies.
    pub struct RegionBarrier: u32 {
        const UNIFORM_READ = __gl::UNIFORM_BARRIER_BIT;
        const SAMPLED_IMAGE_READ = __gl::TEXTURE_FETCH_BARRIER_BIT;
        const STORAGE_IMAGE_RW = __gl::SHADER_IMAGE_ACCESS_BARRIER_BIT;
        const STORAGE_BUFFER_RW = __gl::SHADER_STORAGE_BARRIER_BIT;
        const FRAMEBUFFER_RW = __gl::FRAMEBUFFER_BARRIER_BIT;
        const ATOMIC_COUNTER_RW = __gl::ATOMIC_COUNTER_BARRIER_BIT;
    }
);

impl Device {
    ///
    pub unsafe fn memory_barrier(&self, mut flags: Barrier) {
        if flags.contains(Barrier::INPUT_ATTACHMENT_READ) {
            self.0.TextureBarrier();
        }

        flags.remove(Barrier::INPUT_ATTACHMENT_READ);
        if flags.is_empty() {
            return;
        }

        self.0.MemoryBarrier(flags.bits());
    }

    ///
    pub unsafe fn memory_barrier_by_region(&self, flags: RegionBarrier) {
        self.0.MemoryBarrierByRegion(flags.bits());
    }
}
