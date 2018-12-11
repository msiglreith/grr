//! Drawing and Dispatching related commands.

use __gl;
use std::mem;
use std::ops::Range;

use device::Device;
use {Pipeline, Region};

/// Primitve topology.
///
/// Specifies how the input assembler (fixed-function) of the graphics pipeline will
/// assemble primitives based on the incoming vertex data.
#[repr(u32)]
pub enum Primitive {
    /// Independent vertex points.
    ///
    /// One vertex corresponds to one point. The size of a point can be changed
    /// dynamically in the vertex, geometry or tessellation evaluation stage.
    /// A point is rendered as square.
    ///
    /// GLSL: `gl_PointSize`.
    Points = __gl::POINTS,

    /// Lines segment list.
    ///
    /// Every two consecutive vertices will form a line segment.
    Lines = __gl::LINES,

    /// Lines segment strip.
    ///
    /// The vertices will build a connected list of line segments.
    LineStrip = __gl::LINE_STRIP,

    /// Triangle list.
    ///
    /// Three consecutive vertices will form triangle.
    Triangles = __gl::TRIANGLES,
    TriangleStrip = __gl::TRIANGLE_STRIP,
    LinesAdjacency = __gl::LINES_ADJACENCY,
    LinesStripAdjacency = __gl::LINE_STRIP_ADJACENCY,
    TrianglesAdjacency = __gl::TRIANGLES_ADJACENCY,
    TrianglesStripAdjacency = __gl::TRIANGLE_STRIP_ADJACENCY,
    Patches = __gl::PATCHES,
}

/// Index size.
///
/// Specifies the size of indices during indexed draw calls.
#[repr(u32)]
pub enum IndexTy {
    /// 8-bit unsigned integer.
    U8 = __gl::UNSIGNED_BYTE,
    // 16-bit unsigned integer.
    U16 = __gl::UNSIGNED_SHORT,
    // 32-bit unsigned integer.
    U32 = __gl::UNSIGNED_INT,
}

/// Viewport transformation.
///
/// Viewport transformation is part of the fixed-function Vertex Post-Processing pipeline.
/// The returning vertex positions (GLSL: `gl_Position`) in clip space are transformed to
/// normalized device coordinates (NDC) by perspective division. Viewport transformation
/// further converts the NDC into framebuffer coordinates.
///
/// During the geometry a primitive will be assigned a viewport index (GLSL: `gl_ViewportIndex`)
/// either automatically or manually. This index controls which viewport from the bounded
/// viewports will be selected for applying the transformation for the current **primitive**.
pub struct Viewport {
    /// Offset (x).
    pub x: f32,
    /// Offset (y).
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
    // Near.
    pub n: f64,
    // Far.
    pub f: f64,
}

/// Uniform constant.
///
/// Small values which can be written directly by the API.
/// No additional buffers and binding calls are required.
///
/// ## Example
///
/// GLSL: `layout (location = 0) uniform mat4 u_perspective;`
pub enum Constant {
    /// 32-bit unsigned integer.
    U32(u32),
    /// 32-bit single precision floating point.
    F32(f32),
    /// 3 elements single precision floating point vector.
    Vec3([f32; 3]),
    /// 3x3 elements single precision floating point matrix.
    Mat3x3([[f32; 3]; 3]),
    /// 4x4 elements single precision floating point matrix.
    Mat4x4([[f32; 4]; 4]),
}

///
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DrawIndirectCmd {
    ///
    pub vertex_count: u32,
    ///
    pub instance_count: u32,
    ///
    pub first_vertex: u32,
    ///
    pub first_instance: u32,
}

///
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DrawIndexedIndirectCmd {
    ///
    pub index_count: u32,
    ///
    pub instance_count: u32,
    ///
    pub first_index: u32,
    ///
    pub base_vertex: i32,
    ///
    pub first_instance: u32,
}

impl Device {
    /// Set uniform constants for a pipeline.
    pub fn bind_uniform_constants(&self, pipeline: &Pipeline, first: u32, constants: &[Constant]) {
        for (i, constant) in constants.iter().enumerate() {
            let location = first as i32 + i as i32;
            match constant {
                Constant::U32(val) => unsafe {
                    self.0.ProgramUniform1ui(pipeline.0, location, *val as _);
                },
                Constant::F32(val) => unsafe {
                    self.0.ProgramUniform1f(pipeline.0, location, *val as _);
                },
                Constant::Vec3(v) => unsafe {
                    self.0
                        .ProgramUniform3f(pipeline.0, location, v[0], v[1], v[2]);
                },
                Constant::Mat3x3(mat) => unsafe {
                    self.0.ProgramUniformMatrix3fv(
                        pipeline.0,
                        location,
                        1,
                        __gl::FALSE,
                        mat.as_ptr() as *const _,
                    );
                },
                Constant::Mat4x4(mat) => unsafe {
                    self.0.ProgramUniformMatrix4fv(
                        pipeline.0,
                        location,
                        1,
                        __gl::FALSE,
                        mat.as_ptr() as *const _,
                    );
                },
            }
        }
    }

    /// Set viewport transformation parameters.
    ///
    /// The viewport determines the mapping from NDC (normalized device coordinates)
    /// into framebuffer coordinates.
    ///
    /// See [Viewport](../command/struct.Viewport.html) for more information
    /// about the viewport transformation.
    pub fn set_viewport(&self, first: u32, viewports: &[Viewport]) {
        let rects = viewports
            .iter()
            .flat_map(|viewport| vec![viewport.x, viewport.y, viewport.w, viewport.h])
            .collect::<Vec<_>>();

        unsafe {
            self.0
                .ViewportArrayv(first, viewports.len() as _, rects.as_ptr());
        }

        let depth_ranges = viewports
            .iter()
            .flat_map(|viewport| vec![viewport.n, viewport.f])
            .collect::<Vec<_>>();

        unsafe {
            self.0
                .DepthRangeArrayv(first, viewports.len() as _, depth_ranges.as_ptr());
        }
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
    /// - `indices.end - indices.start` must allow to assemble complete primitives.
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
    }

    ///
    pub fn draw_indirect(&self, primitive: Primitive, offset: u64, count: u32, stride: u32) {
        unsafe {
            self.0
                .MultiDrawArraysIndirect(primitive as _, offset as _, count as _, stride as _);
        }
    }

    ///
    pub fn draw_indirect_from_host(&self, primitive: Primitive, data: &[DrawIndirectCmd]) {
        unsafe {
            self.0.MultiDrawArraysIndirect(
                primitive as _,
                data.as_ptr() as *const _,
                data.len() as _,
                mem::size_of::<DrawIndirectCmd>() as _,
            );
        }
    }

    ///
    pub fn draw_indexed_indirect(
        &self,
        primitive: Primitive,
        index_ty: IndexTy,
        offset: u64,
        count: u32,
        stride: u32,
    ) {
        unsafe {
            self.0.MultiDrawElementsIndirect(
                primitive as _,
                index_ty as _,
                offset as _,
                count as _,
                stride as _,
            );
        }
    }

    ///
    pub fn draw_indexed_indirect_from_host(
        &self,
        primitive: Primitive,
        index_ty: IndexTy,
        data: &[DrawIndexedIndirectCmd],
    ) {
        unsafe {
            self.0.MultiDrawElementsIndirect(
                primitive as _,
                index_ty as _,
                data.as_ptr() as *const _,
                data.len() as _,
                mem::size_of::<DrawIndirectCmd>() as _,
            );
        }
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
    }
}
