#[macro_use]
extern crate bitflags;

use __gl::types::{GLenum, GLuint};

mod __gl;

mod buffer;
mod command;
mod device;
mod error;
mod format;
mod framebuffer;
mod image;
mod sampler;
mod vertex;

pub use buffer::{Buffer, MappingFlags, MemoryFlags};
pub use device::Device;
pub use error::Error;
pub use format::Format;
pub use framebuffer::{Attachment, ClearAttachment, Framebuffer, Renderbuffer};
pub use image::Image;
pub use sampler::{Filter, Sampler, SamplerAddress, SamplerDesc};
pub use vertex::{InputRate, VertexArray, VertexAttributeDesc, VertexBufferView, VertexFormat};

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
}

///
#[repr(transparent)]
pub struct Shader(GLuint);

///
#[repr(transparent)]
pub struct Pipeline(GLuint);

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
