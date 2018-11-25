use __gl;
use std::ops::Range;

use device::Device;
use {IndexTy, Pipeline, Primitive, Region, Viewport};

pub enum Constant {
    U32(u32),
    Mat3x3([[f32; 3]; 3]),
    Mat4x4([[f32; 4]; 4]),
}

impl Device {
    pub fn bind_uniform_constants(&self, pipeline: &Pipeline, first: u32, constants: &[Constant]) {
        for (i, constant) in constants.iter().enumerate() {
            let location = first as i32 + i as i32;
            match constant {
                Constant::U32(val) => unsafe {
                    self.0.Uniform1ui(location, *val as _);
                    self.get_error("ProgramUniform1ui");
                },
                Constant::Mat3x3(mat) => unsafe {
                    self.0.ProgramUniformMatrix3fv(
                        pipeline.0,
                        location,
                        1,
                        __gl::FALSE,
                        mat.as_ptr() as *const _,
                    );
                    self.get_error("ProgramUniformMatrix3fv");
                },
                Constant::Mat4x4(mat) => unsafe {
                    self.0.ProgramUniformMatrix4fv(
                        pipeline.0,
                        location,
                        1,
                        __gl::FALSE,
                        mat.as_ptr() as *const _,
                    );
                    self.get_error("ProgramUniformMatrix4fv");
                },
            }
        }
    }

    /// Set viewport transformation parameters.
    ///
    /// The viewport determines the mapping from NDC (normalized device coordinates)
    /// into window coordinates.
    pub fn set_viewport(&self, first: u32, viewports: &[Viewport]) {
        let rects = viewports
            .iter()
            .flat_map(|viewport| vec![viewport.x, viewport.y, viewport.w, viewport.h])
            .collect::<Vec<_>>();

        unsafe {
            self.0
                .ViewportArrayv(first, viewports.len() as _, rects.as_ptr());
        }
        self.get_error("ViewportArrayv");

        let depth_ranges = viewports
            .iter()
            .flat_map(|viewport| vec![viewport.n, viewport.f])
            .collect::<Vec<_>>();

        unsafe {
            self.0
                .DepthRangeArrayv(first, viewports.len() as _, depth_ranges.as_ptr());
        }
        self.get_error("DepthRangeArrayv");
    }

    /// Set scissor rectangles for viewports.
    ///
    /// # Valid usage
    ///
    /// - Every active viewport needs an associated scissor.
    pub fn set_scissor(&self, first: u32, scissors: &[Region]) {
        let scissors_raw = scissors
            .iter()
            .flat_map(|scissor| vec![scissor.x, scissor.y, scissor.w, scissor.h])
            .collect::<Vec<_>>();

        unsafe {
            self.0
                .ScissorArrayv(first, scissors.len() as _, scissors_raw.as_ptr());
        }
        self.get_error("ScissorArrayv");
    }

    /// Submit a (non-indexed) draw call.
    ///
    /// # Valid usage
    ///
    /// - There must be a valid graphics pipeline currently bound.
    /// - There must be a calid vertex array currently bound.
    /// - For each attribute in the bound vertex array there must be a vertex buffer bound
    ///   at the specified binding slot.
    /// - For each attribute in the bound vertex array there must be a vertex attribute
    ///   specified in the shader with matching format and location.
    /// - The access vertices must be in bound of the vertex buffers bound.
    /// - `vertices.end` must be larger than `vertices.start`.
    /// - `vertices.end - vertices.start` must be allow assembling complete primitives.
    /// - `instances.end` must be larger than `instances.start`.
    pub fn draw(&self, primitive: Primitive, vertices: Range<u32>, instance: Range<u32>) {
        unsafe {
            self.0.DrawArraysInstancedBaseInstance(
                primitive as _,
                vertices.start as _,
                (vertices.end - vertices.start) as _,
                (instance.end - instance.start) as _,
                instance.start as _,
            )
        }
        self.get_error("DrawArraysInstancedBaseInstance");
    }

    /// Submit an indexed draw call.
    ///
    /// # Valid usage
    ///
    /// - There must be a valid graphics pipeline currently bound.
    /// - There must be a calid vertex array currently bound.
    /// - For each attribute in the bound vertex array there must be a vertex buffer bound
    ///   at the specified binding slot.
    /// - For each attribute in the bound vertex array there must be a vertex attribute
    ///   specified in the shader with matching format and location.
    /// - The access vertices must be in bound of the vertex buffers bound.
    /// - `indices.end` must be larger than `indices.start`.
    /// - `indices.end - indices.start` must be allow assembling complete primitives.
    /// - `instances.end` must be larger than `instances.start`.
    pub fn draw_indexed(
        &self,
        primitive: Primitive,
        index_ty: IndexTy,
        indices: Range<u32>,
        instance: Range<u32>,
        base_vertex: i32,
    ) {
        unsafe {
            self.0.DrawElementsInstancedBaseVertexBaseInstance(
                primitive as _,
                (indices.end - indices.start) as _,
                index_ty as _,
                indices.start as _,
                (instance.end - instance.start) as _,
                base_vertex,
                instance.start as _,
            )
        }
        self.get_error("DrawElementsInstancedBaseVertexBaseInstance");
    }

    /// Dispatch a workgroup for computation.
    ///
    /// # Valid usage
    ///
    /// - `group_x`, `group_y` and `group_z` must be larger than 0.
    /// - There must be a valid compute shader currently bound.
    pub fn dispatch(&self, groups_x: u32, groups_y: u32, groups_z: u32) {
        unsafe {
            self.0.DispatchCompute(groups_x, groups_y, groups_z);
        }
        self.get_error("DispatchCompute");
    }
}
