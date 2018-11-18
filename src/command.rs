use std::ops::Range;

use buffer::Buffer;
use device::Device;
use vertex::{InputRate, VertexArray, VertexBufferView};
use {IndexTy, Pipeline, Primitive, Region, Viewport};

impl Device {
    /// Bind vertex buffers to a vertex array.
    pub fn bind_vertex_buffers(&self, vao: &VertexArray, first: u32, views: &[VertexBufferView]) {
        let buffers = views.iter().map(|view| view.buffer.0).collect::<Vec<_>>();

        let offsets = views
            .iter()
            .map(|view| view.offset as _)
            .collect::<Vec<_>>();

        let strides = views
            .iter()
            .map(|view| view.stride as _)
            .collect::<Vec<_>>();

        unsafe {
            self.0.VertexArrayVertexBuffers(
                vao.0,
                first,
                views.len() as _,
                buffers.as_ptr(),
                offsets.as_ptr(),
                strides.as_ptr(),
            );
        }
        self.get_error("VertexArrayVertexBuffers");

        for (binding, view) in views.iter().enumerate() {
            let divisor = match view.input_rate {
                InputRate::Vertex => 0,
                InputRate::Instance { divisor } => divisor,
            };

            unsafe {
                self.0
                    .VertexArrayBindingDivisor(vao.0, first + binding as u32, divisor as _);
            }
            self.get_error("VertexArrayBindingDivisor");
        }
    }

    /// Bind a index buffer to a vertex array.
    pub fn bind_index_buffer(&self, vao: &VertexArray, buffer: &Buffer) {
        unsafe { self.0.VertexArrayElementBuffer(vao.0, buffer.0) }
        self.get_error("VertexArrayElementBuffer");
    }

    /// Bind a pipeline for usage.
    pub fn bind_pipeline(&self, pipeline: &Pipeline) {
        unsafe {
            self.0.UseProgram(pipeline.0);
        }
        self.get_error("UseProgram");
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
                primitive.into(),
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
                primitive.into(),
                (indices.end - indices.start) as _,
                index_ty.into(),
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
