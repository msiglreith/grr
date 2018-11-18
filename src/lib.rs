#[macro_use]
extern crate bitflags;

use __gl::types::{GLenum, GLuint};

use std::ops::Range;

mod __gl;

mod buffer;
mod device;
mod error;
mod framebuffer;
mod sampler;

pub use buffer::{Buffer, MappingFlags, MemoryFlags};
pub use device::Device;
pub use error::Error;
pub use framebuffer::{Attachment, ClearAttachment, Framebuffer, Renderbuffer};
pub use sampler::{Filter, Sampler, SamplerAddress, SamplerDesc};

impl Device {
    fn check_pipeline_log(&self, pipeline: GLuint) {
        let log = {
            let mut len = unsafe {
                let mut len = 0;
                self.0
                    .GetProgramiv(pipeline, __gl::INFO_LOG_LENGTH, &mut len);
                len
            };

            if len > 0 {
                let mut log = String::with_capacity(len as usize);
                log.extend(std::iter::repeat('\0').take(len as usize));
                unsafe {
                    self.0.GetProgramInfoLog(
                        pipeline,
                        len,
                        &mut len,
                        (&log[..]).as_ptr() as *mut _,
                    );
                }
                log.truncate(len as usize);
                log
            } else {
                String::new()
            }
        };

        if !log.is_empty() {
            println!("Pipeline Info Log: {}", log);
        }
    }

    /// Create a new shader from GLSL.
    ///
    /// # Valid usage
    ///
    /// - `source` must be a NULL-terminated C-String.
    /// - The GLSL shader version must be `450 core` or higher.
    /// - The `stage` parameter must be a valid stage of the passed shader source.
    pub fn create_shader(&self, stage: ShaderStage, source: &[u8]) -> Shader {
        let stage = match stage {
            ShaderStage::Vertex => __gl::VERTEX_SHADER,
            ShaderStage::TessellationControl => __gl::TESS_CONTROL_SHADER,
            ShaderStage::TessellationEvaluation => __gl::TESS_EVALUATION_SHADER,
            ShaderStage::Geometry => __gl::GEOMETRY_SHADER,
            ShaderStage::Fragment => __gl::FRAGMENT_SHADER,
            ShaderStage::Compute => __gl::COMPUTE_SHADER,
        };

        let shader = unsafe {
            let shader = self.0.CreateShader(stage);
            self.0.ShaderSource(
                shader,
                1,
                &(source.as_ptr() as *const _),
                &(source.len() as _),
            );
            self.0.CompileShader(shader);

            shader
        };

        let status = unsafe {
            let mut status = 0;
            self.0
                .GetShaderiv(shader, __gl::COMPILE_STATUS, &mut status);
            status
        };

        if status != __gl::TRUE as _ {
            println!("Shader could not be compiled successfully ({:?})", stage);
        }

        let log = {
            let mut len = unsafe {
                let mut len = 0;
                self.0.GetShaderiv(shader, __gl::INFO_LOG_LENGTH, &mut len);
                len
            };

            if len > 0 {
                let mut log = String::with_capacity(len as usize);
                log.extend(std::iter::repeat('\0').take(len as usize));
                unsafe {
                    self.0
                        .GetShaderInfoLog(shader, len, &mut len, (&log[..]).as_ptr() as *mut _);
                }
                log.truncate(len as usize);
                log
            } else {
                String::new()
            }
        };

        if !log.is_empty() {
            println!("Shader Info Log: {}", log);
        }

        Shader(shader)
    }

    /// Delete a shader.
    pub fn delete_shader(&self, shader: Shader) {
        unsafe {
            self.0.DeleteShader(shader.0);
        }
    }

    /// Create a graphics pipeline.
    ///
    /// This equals a `Program` in GL terminology.
    ///
    /// # Valid usage
    ///
    /// - The vertex shader in `desc` must be valid and created with `ShaderStage::Vertex`.
    /// - The tessellation control shader in `desc` must be valid and created with
    ///   `ShaderStage::TessellationControl` if specified.
    /// - The tessellation evaluation shader in `desc` must be valid and created with
    ///   `ShaderStage::TessellationEvalution` if specified.
    /// - The geometry shader in `desc` must be valid and created with
    ///   `ShaderStage::Geometry` if specified.
    /// - The fragment shader in `desc` must be valid and created with
    ///   `ShaderStage::Fragment` if specified.
    pub fn create_graphics_pipeline(&self, desc: GraphicsPipelineDesc) -> Pipeline {
        let pipeline = unsafe { self.0.CreateProgram() };

        unsafe {
            // Attach
            self.0.AttachShader(pipeline, desc.vertex_shader.0);
            if let Some(tsc) = desc.tessellation_control_shader {
                self.0.AttachShader(pipeline, tsc.0);
            }
            if let Some(tse) = desc.tessellation_evaluation_shader {
                self.0.AttachShader(pipeline, tse.0);
            }
            if let Some(geometry) = desc.geometry_shader {
                self.0.AttachShader(pipeline, geometry.0);
            }
            if let Some(fragment) = desc.fragment_shader {
                self.0.AttachShader(pipeline, fragment.0);
            }

            self.0.LinkProgram(pipeline);

            // Detach
            self.0.DetachShader(pipeline, desc.vertex_shader.0);
            if let Some(tsc) = desc.tessellation_control_shader {
                self.0.DetachShader(pipeline, tsc.0);
            }
            if let Some(tse) = desc.tessellation_evaluation_shader {
                self.0.DetachShader(pipeline, tse.0);
            }
            if let Some(geometry) = desc.geometry_shader {
                self.0.DetachShader(pipeline, geometry.0);
            }
            if let Some(fragment) = desc.fragment_shader {
                self.0.DetachShader(pipeline, fragment.0);
            }
        }

        let status = unsafe {
            let mut status = 0;
            self.0
                .GetProgramiv(pipeline, __gl::LINK_STATUS, &mut status);
            status
        };

        if status != __gl::TRUE as _ {
            println!("Graphics pipeline could not be linked successfully");
        }

        self.check_pipeline_log(pipeline);
        Pipeline(pipeline)
    }

    /// Create a compute pipeline.
    ///
    /// This equals a `Program` in GL terminology.
    ///
    /// # Valid usage
    ///
    /// - The compute shader in must be valid and created with `ShaderStage::Compute`.
    pub fn create_compute_pipeline(&self, compute_shader: &Shader) -> Pipeline {
        let pipeline = unsafe { self.0.CreateProgram() };

        unsafe {
            self.0.AttachShader(pipeline, compute_shader.0);
            self.get_error("AttachShader");
            self.0.LinkProgram(pipeline);
            self.get_error("LinkProgram");
            self.0.DetachShader(pipeline, compute_shader.0);
            self.get_error("DetachShader");
        }

        let status = unsafe {
            let mut status = 0;
            self.0
                .GetProgramiv(pipeline, __gl::LINK_STATUS, &mut status);
            status
        };

        if status != __gl::TRUE as _ {
            println!("Compute pipeline could not be linked successfully");
        }

        self.check_pipeline_log(pipeline);
        Pipeline(pipeline)
    }

    /// Create a new vertex array, storing information for the input assembler.
    ///
    /// The vertex array specified the vertex attributes and their binding to
    /// vertex buffer objects.
    pub fn create_vertex_array(&self, attributes: &[VertexAttributeDesc]) -> VertexArray {
        let mut vao = 0;
        unsafe {
            self.0.CreateVertexArrays(1, &mut vao);
        }
        self.get_error("CreateVertexArrays");

        enum VertexBase {
            Int,
            Float,
            Double,
        }

        for desc in attributes {
            let (base, num, ty, norm) = match desc.format {
                VertexFormat::X8Int => (VertexBase::Int, 1, __gl::BYTE, false),
                VertexFormat::X8Uint => (VertexBase::Int, 1, __gl::UNSIGNED_BYTE, false),
                VertexFormat::X8Unorm => (VertexBase::Float, 1, __gl::UNSIGNED_BYTE, true),
                VertexFormat::X8Inorm => (VertexBase::Float, 1, __gl::BYTE, true),
                VertexFormat::X8Uscaled => (VertexBase::Float, 1, __gl::UNSIGNED_BYTE, false),
                VertexFormat::X8Iscaled => (VertexBase::Float, 1, __gl::BYTE, false),

                VertexFormat::Xy8Int => (VertexBase::Int, 2, __gl::BYTE, false),
                VertexFormat::Xy8Uint => (VertexBase::Int, 2, __gl::UNSIGNED_BYTE, false),
                VertexFormat::Xy8Unorm => (VertexBase::Float, 2, __gl::UNSIGNED_BYTE, true),
                VertexFormat::Xy8Inorm => (VertexBase::Float, 2, __gl::BYTE, true),
                VertexFormat::Xy8Uscaled => (VertexBase::Float, 2, __gl::UNSIGNED_BYTE, false),
                VertexFormat::Xy8Iscaled => (VertexBase::Float, 2, __gl::BYTE, false),

                VertexFormat::Xyz8Int => (VertexBase::Int, 3, __gl::BYTE, false),
                VertexFormat::Xyz8Uint => (VertexBase::Int, 3, __gl::UNSIGNED_BYTE, false),
                VertexFormat::Xyz8Unorm => (VertexBase::Float, 3, __gl::UNSIGNED_BYTE, true),
                VertexFormat::Xyz8Inorm => (VertexBase::Float, 3, __gl::BYTE, true),
                VertexFormat::Xyz8Uscaled => (VertexBase::Float, 3, __gl::UNSIGNED_BYTE, false),
                VertexFormat::Xyz8Iscaled => (VertexBase::Float, 3, __gl::BYTE, false),

                VertexFormat::Xyzw8Int => (VertexBase::Int, 4, __gl::BYTE, false),
                VertexFormat::Xyzw8Uint => (VertexBase::Int, 4, __gl::UNSIGNED_BYTE, false),
                VertexFormat::Xyzw8Unorm => (VertexBase::Float, 4, __gl::UNSIGNED_BYTE, true),
                VertexFormat::Xyzw8Inorm => (VertexBase::Float, 4, __gl::BYTE, true),
                VertexFormat::Xyzw8Uscaled => (VertexBase::Float, 4, __gl::UNSIGNED_BYTE, false),
                VertexFormat::Xyzw8Iscaled => (VertexBase::Float, 4, __gl::BYTE, false),

                VertexFormat::X16Int => (VertexBase::Int, 1, __gl::SHORT, false),
                VertexFormat::X16Uint => (VertexBase::Int, 1, __gl::UNSIGNED_SHORT, false),
                VertexFormat::X16Float => (VertexBase::Float, 1, __gl::HALF_FLOAT, false),
                VertexFormat::X16Unorm => (VertexBase::Float, 1, __gl::UNSIGNED_SHORT, true),
                VertexFormat::X16Inorm => (VertexBase::Float, 1, __gl::SHORT, true),
                VertexFormat::X16Uscaled => (VertexBase::Float, 1, __gl::UNSIGNED_SHORT, false),
                VertexFormat::X16Iscaled => (VertexBase::Float, 1, __gl::SHORT, false),

                VertexFormat::Xy16Int => (VertexBase::Int, 2, __gl::SHORT, false),
                VertexFormat::Xy16Uint => (VertexBase::Int, 2, __gl::UNSIGNED_SHORT, false),
                VertexFormat::Xy16Float => (VertexBase::Float, 2, __gl::HALF_FLOAT, false),
                VertexFormat::Xy16Unorm => (VertexBase::Float, 2, __gl::UNSIGNED_SHORT, true),
                VertexFormat::Xy16Inorm => (VertexBase::Float, 2, __gl::SHORT, true),
                VertexFormat::Xy16Uscaled => (VertexBase::Float, 2, __gl::UNSIGNED_SHORT, false),
                VertexFormat::Xy16Iscaled => (VertexBase::Float, 2, __gl::SHORT, false),

                VertexFormat::Xyz16Int => (VertexBase::Int, 3, __gl::SHORT, false),
                VertexFormat::Xyz16Uint => (VertexBase::Int, 3, __gl::UNSIGNED_SHORT, false),
                VertexFormat::Xyz16Float => (VertexBase::Float, 3, __gl::HALF_FLOAT, false),
                VertexFormat::Xyz16Unorm => (VertexBase::Float, 3, __gl::UNSIGNED_SHORT, true),
                VertexFormat::Xyz16Inorm => (VertexBase::Float, 3, __gl::SHORT, true),
                VertexFormat::Xyz16Uscaled => (VertexBase::Float, 3, __gl::UNSIGNED_SHORT, false),
                VertexFormat::Xyz16Iscaled => (VertexBase::Float, 3, __gl::SHORT, false),

                VertexFormat::Xyzw16Int => (VertexBase::Int, 4, __gl::SHORT, false),
                VertexFormat::Xyzw16Uint => (VertexBase::Int, 4, __gl::UNSIGNED_SHORT, false),
                VertexFormat::Xyzw16Float => (VertexBase::Float, 4, __gl::HALF_FLOAT, false),
                VertexFormat::Xyzw16Unorm => (VertexBase::Float, 4, __gl::UNSIGNED_SHORT, true),
                VertexFormat::Xyzw16Inorm => (VertexBase::Float, 4, __gl::SHORT, true),
                VertexFormat::Xyzw16Uscaled => (VertexBase::Float, 4, __gl::UNSIGNED_SHORT, false),
                VertexFormat::Xyzw16Iscaled => (VertexBase::Float, 4, __gl::SHORT, false),

                VertexFormat::X32Int => (VertexBase::Int, 1, __gl::INT, false),
                VertexFormat::X32Uint => (VertexBase::Int, 1, __gl::UNSIGNED_INT, false),
                VertexFormat::X32Float => (VertexBase::Float, 1, __gl::FLOAT, false),
                VertexFormat::X32Unorm => (VertexBase::Float, 1, __gl::UNSIGNED_INT, true),
                VertexFormat::X32Inorm => (VertexBase::Float, 1, __gl::INT, true),
                VertexFormat::X32Uscaled => (VertexBase::Float, 1, __gl::UNSIGNED_INT, false),
                VertexFormat::X32Iscaled => (VertexBase::Float, 1, __gl::INT, false),

                VertexFormat::Xy32Int => (VertexBase::Int, 2, __gl::INT, false),
                VertexFormat::Xy32Uint => (VertexBase::Int, 2, __gl::UNSIGNED_INT, false),
                VertexFormat::Xy32Float => (VertexBase::Float, 2, __gl::FLOAT, false),
                VertexFormat::Xy32Unorm => (VertexBase::Float, 2, __gl::UNSIGNED_INT, true),
                VertexFormat::Xy32Inorm => (VertexBase::Float, 2, __gl::INT, true),
                VertexFormat::Xy32Uscaled => (VertexBase::Float, 2, __gl::UNSIGNED_INT, false),
                VertexFormat::Xy32Iscaled => (VertexBase::Float, 2, __gl::INT, false),

                VertexFormat::Xyz32Int => (VertexBase::Int, 3, __gl::INT, false),
                VertexFormat::Xyz32Uint => (VertexBase::Int, 3, __gl::UNSIGNED_INT, false),
                VertexFormat::Xyz32Float => (VertexBase::Float, 3, __gl::FLOAT, false),
                VertexFormat::Xyz32Unorm => (VertexBase::Float, 3, __gl::UNSIGNED_INT, true),
                VertexFormat::Xyz32Inorm => (VertexBase::Float, 3, __gl::INT, true),
                VertexFormat::Xyz32Uscaled => (VertexBase::Float, 3, __gl::UNSIGNED_INT, false),
                VertexFormat::Xyz32Iscaled => (VertexBase::Float, 3, __gl::INT, false),

                VertexFormat::Xyzw32Int => (VertexBase::Int, 4, __gl::INT, false),
                VertexFormat::Xyzw32Uint => (VertexBase::Int, 4, __gl::UNSIGNED_INT, false),
                VertexFormat::Xyzw32Float => (VertexBase::Float, 4, __gl::FLOAT, false),
                VertexFormat::Xyzw32Unorm => (VertexBase::Float, 4, __gl::UNSIGNED_INT, true),
                VertexFormat::Xyzw32Inorm => (VertexBase::Float, 4, __gl::INT, true),
                VertexFormat::Xyzw32Uscaled => (VertexBase::Float, 4, __gl::UNSIGNED_INT, false),
                VertexFormat::Xyzw32Iscaled => (VertexBase::Float, 4, __gl::INT, false),

                VertexFormat::X64Float => (VertexBase::Double, 1, __gl::DOUBLE, false),
                VertexFormat::Xy64Float => (VertexBase::Double, 2, __gl::DOUBLE, false),
                VertexFormat::Xyz64Float => (VertexBase::Double, 3, __gl::DOUBLE, false),
                VertexFormat::Xyzw64Float => (VertexBase::Double, 4, __gl::DOUBLE, false),
            };

            unsafe {
                self.0.EnableVertexArrayAttrib(vao, desc.location);
                self.get_error("EnableVertexArrayAttrib");

                match base {
                    VertexBase::Int => {
                        self.0
                            .VertexArrayAttribIFormat(vao, desc.location, num, ty, desc.offset);
                        self.get_error("VertexArrayAttribIFormat");
                    }
                    VertexBase::Float => {
                        self.0.VertexArrayAttribFormat(
                            vao,
                            desc.location,
                            num,
                            ty,
                            norm as _,
                            desc.offset,
                        );
                        self.get_error("VertexArrayAttribFormat");
                    }
                    VertexBase::Double => {
                        self.0
                            .VertexArrayAttribLFormat(vao, desc.location, num, ty, desc.offset);
                        self.get_error("VertexArrayAttribLFormat");
                    }
                }

                self.0
                    .VertexArrayAttribBinding(vao, desc.location, desc.binding);
                self.get_error("VertexArrayAttribBinding");
            }
        }

        VertexArray(vao)
    }

    /// Delete a vertex array.
    pub fn delete_vertex_array(&self, vao: VertexArray) {
        unsafe { self.0.DeleteVertexArrays(1, &vao.0) }
        self.get_error("DeleteVertexArrays");
    }

    /// Bind a vertex array for usage.
    pub fn bind_vertex_array(&self, vao: &VertexArray) {
        unsafe {
            self.0.BindVertexArray(vao.0);
        }
        self.get_error("BindVertexArray");
    }

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

///
pub struct Shader(GLuint);

///
pub struct Pipeline(GLuint);

///
pub struct VertexArray(GLuint);

///
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    /// Width
    pub w: f32,
    /// Height
    pub h: f32,
    // Near
    pub n: f64,
    // Far
    pub f: f64,
}

///
pub struct Region {
    pub x: i32,
    pub y: i32,
    /// Width
    pub w: i32,
    /// Height
    pub h: i32,
}

///
#[derive(Debug, Clone, Copy)]
pub enum ShaderStage {
    Vertex,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
    Fragment,
    Compute,
}

///
pub enum Primitive {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    LinesAdjacency,
    LinesStripAdjacency,
    TrianglesAdjacency,
    TrianglesStripAdjacency,
    Patches,
}

impl From<Primitive> for GLenum {
    fn from(primitive: Primitive) -> Self {
        match primitive {
            Primitive::Points => __gl::POINTS,
            Primitive::Lines => __gl::LINES,
            Primitive::LineStrip => __gl::LINE_STRIP,
            Primitive::Triangles => __gl::TRIANGLES,
            Primitive::TriangleStrip => __gl::TRIANGLE_STRIP,
            Primitive::LinesAdjacency => __gl::LINES_ADJACENCY,
            Primitive::LinesStripAdjacency => __gl::LINE_STRIP_ADJACENCY,
            Primitive::TrianglesAdjacency => __gl::TRIANGLES_ADJACENCY,
            Primitive::TrianglesStripAdjacency => __gl::TRIANGLE_STRIP_ADJACENCY,
            Primitive::Patches => __gl::PATCHES,
        }
    }
}

///
pub enum IndexTy {
    U8,
    U16,
    U32,
}

impl From<IndexTy> for GLenum {
    fn from(ty: IndexTy) -> Self {
        match ty {
            IndexTy::U8 => __gl::UNSIGNED_BYTE,
            IndexTy::U16 => __gl::UNSIGNED_SHORT,
            IndexTy::U32 => __gl::UNSIGNED_INT,
        }
    }
}

///
pub struct GraphicsPipelineDesc<'a> {
    pub vertex_shader: &'a Shader,
    pub tessellation_control_shader: Option<&'a Shader>,
    pub tessellation_evaluation_shader: Option<&'a Shader>,
    pub geometry_shader: Option<&'a Shader>,
    pub fragment_shader: Option<&'a Shader>,
}

///
pub struct VertexBufferView<'a> {
    pub buffer: &'a Buffer,
    pub offset: u64,
    pub stride: u32,
    pub input_rate: InputRate,
}

///
pub struct VertexAttributeDesc {
    pub location: u32,
    pub binding: u32,
    pub format: VertexFormat,
    pub offset: u32,
}

///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Compare {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    Always,
    Never,
}

impl From<Compare> for GLenum {
    fn from(compare: Compare) -> Self {
        match compare {
            Compare::Less => __gl::LESS,
            Compare::LessEqual => __gl::LEQUAL,
            Compare::Greater => __gl::GREATER,
            Compare::GreaterEqual => __gl::GEQUAL,
            Compare::Equal => __gl::EQUAL,
            Compare::NotEqual => __gl::NOTEQUAL,
            Compare::Always => __gl::ALWAYS,
            Compare::Never => __gl::NEVER,
        }
    }
}

///
pub enum InputRate {
    Vertex,
    Instance { divisor: usize },
}

///
pub enum VertexFormat {
    X8Int,
    X8Uint,
    X8Unorm,
    X8Inorm,
    X8Uscaled,
    X8Iscaled,

    Xy8Int,
    Xy8Uint,
    Xy8Unorm,
    Xy8Inorm,
    Xy8Uscaled,
    Xy8Iscaled,

    Xyz8Int,
    Xyz8Uint,
    Xyz8Unorm,
    Xyz8Inorm,
    Xyz8Uscaled,
    Xyz8Iscaled,

    Xyzw8Int,
    Xyzw8Uint,
    Xyzw8Unorm,
    Xyzw8Inorm,
    Xyzw8Uscaled,
    Xyzw8Iscaled,

    X16Int,
    X16Uint,
    X16Float,
    X16Unorm,
    X16Inorm,
    X16Uscaled,
    X16Iscaled,

    Xy16Int,
    Xy16Uint,
    Xy16Float,
    Xy16Unorm,
    Xy16Inorm,
    Xy16Uscaled,
    Xy16Iscaled,

    Xyz16Int,
    Xyz16Uint,
    Xyz16Float,
    Xyz16Unorm,
    Xyz16Inorm,
    Xyz16Uscaled,
    Xyz16Iscaled,

    Xyzw16Int,
    Xyzw16Uint,
    Xyzw16Float,
    Xyzw16Unorm,
    Xyzw16Inorm,
    Xyzw16Uscaled,
    Xyzw16Iscaled,

    X32Int,
    X32Uint,
    X32Float,
    X32Unorm,
    X32Inorm,
    X32Uscaled,
    X32Iscaled,

    Xy32Int,
    Xy32Uint,
    Xy32Float,
    Xy32Unorm,
    Xy32Inorm,
    Xy32Uscaled,
    Xy32Iscaled,

    Xyz32Int,
    Xyz32Uint,
    Xyz32Float,
    Xyz32Unorm,
    Xyz32Inorm,
    Xyz32Uscaled,
    Xyz32Iscaled,

    Xyzw32Int,
    Xyzw32Uint,
    Xyzw32Float,
    Xyzw32Unorm,
    Xyzw32Inorm,
    Xyzw32Uscaled,
    Xyzw32Iscaled,

    X64Float,
    Xy64Float,
    Xyz64Float,
    Xyzw64Float,
}
