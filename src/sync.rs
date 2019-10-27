use crate::__gl;

use crate::Device;

bitflags!(
    /// Memory barrier.
    pub struct Barrier: u32 {
        const VERTEX_ATRRIBUTE_READ = __gl::VERTEX_ATTRIB_ARRAY_BARRIER_BIT;
        const INDEX_READ = __gl::ELEMENT_ARRAY_BARRIER_BIT;
        const UNIFORM_READ = __gl::UNIFORM_BARRIER_BIT;
        const SAMPLED_IMAGE_READ = __gl::TEXTURE_FETCH_BARRIER_BIT;
        const STORAGE_IMAGE_RW = __gl::SHADER_IMAGE_ACCESS_BARRIER_BIT;
        const INDIRECT_COMMAND_READ = __gl::COMMAND_BARRIER_BIT;
        const PIXEL_BUFFER_RW = __gl::PIXEL_BUFFER_BARRIER_BIT; // TODO
        const TEXTURE_TRANSFER = __gl::TEXTURE_UPDATE_BARRIER_BIT;
        const BUFFER_TRANSFER = __gl::BUFFER_UPDATE_BARRIER_BIT;
        const FRAMEBUFFER_RW = __gl::FRAMEBUFFER_BARRIER_BIT;
        const TRANSFORM_FEEDBACK_WRITE = __gl::TRANSFORM_FEEDBACK_BARRIER_BIT;
        const ATOMIC_COUNTER_RW = __gl::ATOMIC_COUNTER_BARRIER_BIT;
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
