//! Graphics and Compute pipeline

use crate::__gl;
use crate::__gl::types::{GLint, GLuint};

use crate::debug::{Object, ObjectType};
use crate::device::Device;
use crate::error::{Error, Result};
use crate::Compare;

/// Shader.
///
/// Shaders are programmable parts of [`Pipelines`](struct.Pipeline.html). Each shader has a fixed
/// [`ShaderStage`](struct.ShaderStage.html) in the pipeline. Shaders may be reused in different pipelines
/// and specify the operations which will transform a predefined set of inputs into a set of output variables.
/// The shader stages defines the input and output layout.
///
/// Beside the input and output variables, shaders can also access GPU memory via bound buffers and images.
///
/// ## Shading Lanuage
///
/// OpenGL comes with an defined Shading Language (GLSL), which will be also used in the documentation
/// for writing shaders. The OpenGL drivers will translate the GLSL shaders into IHV specific machine language
/// via an built-in compiler. Beside the shader representation in text form (GLSL) with GL 4.6 comes also support
/// for the binary SPIR-V format.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shader(GLuint);

impl Object for Shader {
    const TYPE: ObjectType = ObjectType::Shader;
    fn handle(&self) -> GLuint {
        self.0
    }
}

/// Graphics or Compute pipeline.
///
/// Specifies how draw or dispatch commands are executed.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pipeline(pub(crate) GLuint);

impl Object for Pipeline {
    const TYPE: ObjectType = ObjectType::Pipeline;
    fn handle(&self) -> GLuint {
        self.0
    }
}

/// Shader Stages.
///
/// Each [`Shader`](struct.Shader.html) has an associated stage in the pipeline.
/// See [`GraphicsPipelineDesc`](struct.GraphicsPipelineDesc.html) for more details about graphics pipeline stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    /// Vertex stage.
    Vertex,
    /// Tessellation (Control) stage.
    TessellationControl,
    /// Tessellation (Evaluation) stage.
    TessellationEvaluation,
    /// Geometry stage.
    Geometry,
    /// Fragment stage.
    Fragment,
    /// Compute stage.
    Compute,
    /// Mesh stage (NVIDIA).
    MeshNv,
    /// Task stage (NVIDIA).
    TaskNv,
}

bitflags!(
    /// Shader compilation flags.
    pub struct ShaderFlags: u8 {
        /// Write compilation errors to stdout.
        const VERBOSE = 0x1;
    }
);

bitflags!(
    /// Pipeline link flags.
    pub struct PipelineFlags: u8 {
        /// Write link errors to stdout.
        const VERBOSE = 0x1;
    }
);

/// Graphics Pipeline Descriptor.
///
/// ## Overview
///
/// The graphics pipeline is invoked by executing a draw command. The pipeline consists of multiple stages,
/// where some are fully programmable ([`Shader`](struct.Shader.html)) and other fixed-function stages can be only configured.
///
/// ## Stages
///
/// We will go through the the different stages starting from top to bottom.
/// At the highest abstraction level we split the graphics pipeline into three components (`grr` terminology):
///
///  * *Primitive Stage*: Reading data from buffers and generates primitives.
///  * *Rasterizer*: Transforms primitives into fragments.
///  * *Fragment Stage*: Shades fragments and blends them into the [`framebuffer`](struct.Framebuffer.html).
///
/// Fig. 1 shows a very simplistic view of a graphics pipeline consisting of a vertex (VS) and fragment (FS) shader. We will discuss the different
/// stages in more detail later on. The *Primitive Stage* in this examples consists of the Input Assembler (IA) and the Vertex Shader (VS).
/// The *Rasterizer* is shown as the fixed function RS stage. The fragment shader together with the framebuffer output (FB) build the *Fragment Stage*.
///
/// <figure>
///     <img src="https://raw.githubusercontent.com/msiglreith/grr/master/info/doc/graphics_pipeline_base_vs_ps.png" width="500px">
///     <figcaption>Fig.1 Basic Vertex-Fragment Shader Pipeline</figcaption>
/// </figure>
///
/// In the following the different top-level stages will be split up and discussed in more detail
///
/// ### Primitive Stage
///
/// ### Rasterizer
///
/// ### Fragment Stage
///
/// ## Examples
///
#[derive(Copy, Clone)]
pub struct GraphicsPipelineDesc {
    pub vertex_shader: Option<Shader>,
    pub tessellation_control_shader: Option<Shader>,
    pub tessellation_evaluation_shader: Option<Shader>,
    pub geometry_shader: Option<Shader>,
    pub fragment_shader: Option<Shader>,
    pub mesh_shader: Option<Shader>,
    pub task_shader: Option<Shader>,
}

///
#[derive(Copy, Clone)]
pub struct VertexPipelineDesc {
    pub vertex_shader: Shader,
    pub tessellation_control_shader: Option<Shader>,
    pub tessellation_evaluation_shader: Option<Shader>,
    pub geometry_shader: Option<Shader>,
    pub fragment_shader: Option<Shader>,
}

impl From<VertexPipelineDesc> for GraphicsPipelineDesc {
    fn from(desc: VertexPipelineDesc) -> Self {
        GraphicsPipelineDesc {
            vertex_shader: Some(desc.vertex_shader),
            tessellation_control_shader: desc.tessellation_control_shader,
            tessellation_evaluation_shader: desc.tessellation_evaluation_shader,
            geometry_shader: desc.geometry_shader,
            fragment_shader: desc.fragment_shader,
            mesh_shader: None,
            task_shader: None,
        }
    }
}

///
#[derive(Copy, Clone)]
pub struct MeshPipelineDesc {
    pub mesh_shader: Shader,
    pub task_shader: Option<Shader>,
    pub fragment_shader: Option<Shader>,
}

impl From<MeshPipelineDesc> for GraphicsPipelineDesc {
    fn from(desc: MeshPipelineDesc) -> Self {
        GraphicsPipelineDesc {
            vertex_shader: None,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            fragment_shader: desc.fragment_shader,
            mesh_shader: Some(desc.mesh_shader),
            task_shader: desc.task_shader,
        }
    }
}

/// Input Assembly Descriptor.
///
/// Configures the input assembler for primitive shading.
#[derive(Debug, Copy, Clone)]
pub struct InputAssembly {
    /// Specifies if a special vertex index indicates a restart of the primitive assembly.
    pub primitive_restart: Option<u32>,
}

/// Rasteriyer Descriptor.
///
/// Controls the rasterization process for converting primitives into fragments.
#[derive(Debug, Copy, Clone)]
pub struct Rasterization {
    /// Clamp depth values of fragments to the z-planes instead of clipping.
    pub depth_clamp: bool,
    /// Discard primitives before rasterization.
    pub rasterizer_discard: bool,
    /// Specifies how polygons will be rendered.
    pub polygon_mode: PolygonMode,
    ///
    pub cull_mode: Option<CullMode>,
    /// Specifes the winding order for triangles.
    ///
    /// The winding order determines which the visible face of a triangle.
    pub front_face: FrontFace,
    ///
    pub depth_bias: bool,
}

/// Polygon rendering mode.
///
/// Used during [`Rasterization`](struct.Rasterization.html).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PolygonMode {
    Point = __gl::POINT,
    Line = __gl::LINE,
    Fill = __gl::FILL,
}

/// Polygon culling mode.
///
/// Used during [`Rasterization`](struct.Rasterization.html).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CullMode {
    Front = __gl::FRONT,
    Back = __gl::BACK,
    FrontBack = __gl::FRONT_AND_BACK,
}

/// Polygon front face.
///
/// Used during [`Rasterization`](struct.Rasterization.html).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FrontFace {
    CounterClockwise = __gl::CCW,
    Clockwise = __gl::CW,
}

///
#[derive(Debug, Clone)]
pub struct ColorBlend {
    pub attachments: Vec<ColorBlendAttachment>,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlendFactor {
    Zero = __gl::ZERO,
    One = __gl::ONE,
    SrcColor = __gl::SRC_COLOR,
    OneMinusSrcColor = __gl::ONE_MINUS_SRC_COLOR,
    DstColor = __gl::DST_COLOR,
    OneMinusDstColor = __gl::ONE_MINUS_DST_COLOR,
    SrcAlpha = __gl::SRC_ALPHA,
    OneMinusSrcAlpha = __gl::ONE_MINUS_SRC_ALPHA,
    DstAlpha = __gl::DST_ALPHA,
    OneMinusDstAlpha = __gl::ONE_MINUS_DST_ALPHA,
    ConstantColor = __gl::CONSTANT_COLOR,
    OneMinusConstantColor = __gl::ONE_MINUS_CONSTANT_COLOR,
    ConstantAlpha = __gl::CONSTANT_ALPHA,
    OneMinusConstantAlpha = __gl::ONE_MINUS_CONSTANT_ALPHA,
    SrcAlphaSaturate = __gl::SRC_ALPHA_SATURATE,
    Src1Color = __gl::SRC1_COLOR,
    OneMinusSrc1Color = __gl::ONE_MINUS_SRC1_COLOR,
    Src1Alpha = __gl::SRC1_ALPHA,
    OneMinusSrc1Alpha = __gl::ONE_MINUS_SRC1_ALPHA,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlendOp {
    Add = __gl::FUNC_ADD,
    Substract = __gl::FUNC_SUBTRACT,
    ReverseSubstract = __gl::FUNC_REVERSE_SUBTRACT,
    Min = __gl::MIN,
    Max = __gl::MAX,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlendChannel {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub blend_op: BlendOp,
}

///
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ColorBlendAttachment {
    pub blend_enable: bool,
    pub color: BlendChannel,
    pub alpha: BlendChannel,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StencilOp {
    Keep = __gl::KEEP,
    Zero = __gl::ZERO,
    Replace = __gl::REPLACE,
    IncrementClamp = __gl::INCR,
    DecrementClamp = __gl::DECR,
    Invert = __gl::INVERT,
    IncrementWrap = __gl::INCR_WRAP,
    DecrementWrap = __gl::DECR_WRAP,
}

///
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StencilFace {
    pub fail: StencilOp,
    pub pass: StencilOp,
    pub depth_fail: StencilOp,
    pub compare_op: Compare,
    pub compare_mask: u32,
    pub reference: u32,
}

impl StencilFace {
    pub const KEEP: StencilFace = StencilFace {
        fail: StencilOp::Keep,
        pass: StencilOp::Keep,
        depth_fail: StencilOp::Keep,
        compare_op: Compare::Always,
        compare_mask: !0,
        reference: 0,
    };
}

///
#[derive(Debug, Copy, Clone)]
pub struct DepthStencil {
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_compare_op: Compare,
    pub stencil_test: bool,
    pub stencil_front: StencilFace,
    pub stencil_back: StencilFace,
}

///
#[derive(Debug, Copy, Clone)]
pub struct Multisample {
    pub sample_shading: bool,
    pub min_sample_shading: f32,
    pub sample_mask: u64,
    pub alpha_to_coverage: bool,
    pub alpha_to_one: bool,
}

impl Device {
    /// Compile a new shader from GLSL, returning the shader object iff compilation was successful.
    unsafe fn compile_shader(&self, stage: ShaderStage, source: &[u8]) -> Result<Shader> {
        let stage = match stage {
            ShaderStage::Vertex => __gl::VERTEX_SHADER,
            ShaderStage::TessellationControl => __gl::TESS_CONTROL_SHADER,
            ShaderStage::TessellationEvaluation => __gl::TESS_EVALUATION_SHADER,
            ShaderStage::Geometry => __gl::GEOMETRY_SHADER,
            ShaderStage::Fragment => __gl::FRAGMENT_SHADER,
            ShaderStage::Compute => __gl::COMPUTE_SHADER,
            ShaderStage::MeshNv => __gl::MESH_SHADER_NV,
            ShaderStage::TaskNv => __gl::TASK_SHADER_NV,
        };

        let shader = {
            let shader = self.0.CreateShader(stage);
            self.get_error()?;
            self.0.ShaderSource(
                shader,
                1,
                &(source.as_ptr() as *const _),
                &(source.len() as _),
            );
            self.0.CompileShader(shader);

            Shader(shader)
        };
        let status = {
            let mut status = 0;
            self.0
                .GetShaderiv(shader.0, __gl::COMPILE_STATUS, &mut status);
            status
        };

        if status != GLint::from(__gl::TRUE) {
            //self.0.DeleteShader(shader);
            return Err(Error::CompileError(shader));
        }

        Ok(shader)
    }

    /// Create a new shader from GLSL.
    ///
    /// # Valid usage
    ///
    /// - `source` must be a NULL-terminated C-String.
    /// - The GLSL shader version must be `450 core` or higher.
    /// - The `stage` parameter must be a valid stage of the passed shader source.
    pub unsafe fn create_shader(
        &self,
        stage: ShaderStage,
        source: &[u8],
        flags: ShaderFlags,
    ) -> Result<Shader> {
        let shader = self.compile_shader(stage, source);

        // If we're not in a verbose mode, just return the result of
        // the shader compilation.
        if !(flags.contains(ShaderFlags::VERBOSE)) {
            return shader;
        }

        match shader {
            Ok(s) | Err(Error::CompileError(s)) => {
                if shader.is_err() {
                    println!("Shader could not be compiled successfully ({:?})", stage);
                }

                let log = self.get_shader_log(s);
                if let Some(msg) = log {
                    println!("Shader Info Log: {}", msg);
                }
            }
            _ => {}
        }

        shader
    }

    /// Return the log, if any, from compiling the shader.
    pub unsafe fn get_shader_log(&self, shader: Shader) -> Option<String> {
        let mut len = {
            let mut len = 0;
            self.0
                .GetShaderiv(shader.0, __gl::INFO_LOG_LENGTH, &mut len);
            len
        };

        if len > 0 {
            let mut log = String::with_capacity(len as usize);
            log.extend(std::iter::repeat('\0').take(len as usize));
            self.0
                .GetShaderInfoLog(shader.0, len, &mut len, (&log[..]).as_ptr() as *mut _);
            log.truncate(len as usize);
            Some(log)
        } else {
            None
        }
    }

    /// Delete a shader.
    pub unsafe fn delete_shader(&self, shader: Shader) {
        self.0.DeleteShader(shader.0);
    }

    /// Delete multiple shaders.
    pub unsafe fn delete_shaders(&self, shaders: &[Shader]) {
        for shader in shaders.iter() {
            self.0.DeleteShader(shader.0);
        }
    }
    /// Retrieve the log from the most recent program link.
    ///
    /// # Returns
    ///
    /// - Ok(log) if the link was successful.
    /// - Err(log) if the link failed.
    pub unsafe fn get_pipeline_log(&self, pipeline: Pipeline) -> Option<String> {
        let mut len = {
            let mut len = 0;
            self.0
                .GetProgramiv(pipeline.0, __gl::INFO_LOG_LENGTH, &mut len);
            len
        };

        if len > 0 {
            let mut log = String::with_capacity(len as usize);
            log.extend(std::iter::repeat('\0').take(len as usize));
            self.0
                .GetProgramInfoLog(pipeline.0, len, &mut len, (&log[..]).as_ptr() as *mut _);
            log.truncate(len as usize);
            Some(log)
        } else {
            None
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
    pub unsafe fn create_graphics_pipeline<D>(
        &self,
        desc: D,
        flags: PipelineFlags,
    ) -> Result<Pipeline>
    where
        D: Into<GraphicsPipelineDesc>,
    {
        let desc = desc.into();

        let shaders: Vec<_> = [
            desc.vertex_shader,
            desc.tessellation_control_shader,
            desc.tessellation_evaluation_shader,
            desc.geometry_shader,
            desc.fragment_shader,
            desc.mesh_shader,
            desc.task_shader,
        ]
        .iter()
        .filter_map(|&x| x)
        .collect();

        self.create_pipeline(&shaders, flags)
    }

    /// Create a compute pipeline.
    ///
    /// This equals a `Program` in GL terminology.
    ///
    /// # Valid usage
    ///
    /// - The compute shader in must be valid and created with `ShaderStage::Compute`.
    pub unsafe fn create_compute_pipeline(
        &self,
        compute_shader: Shader,
        flags: PipelineFlags,
    ) -> Result<Pipeline> {
        self.create_pipeline(&[compute_shader], flags)
    }

    /// Create a generic pipeline with an arbitrary list of shaders.
    ///
    /// This equals a `Program` in GL terminology.
    ///
    /// # Valid usage
    ///
    /// - The shaders must all be valid.
    /// - The shader stages must be mutually compatible.
    pub unsafe fn create_pipeline(
        &self,
        shaders: &[Shader],
        flags: PipelineFlags,
    ) -> Result<Pipeline> {
        let pipeline = self.0.CreateProgram();
        self.get_error()?;

        for shader in shaders {
            self.0.AttachShader(pipeline, shader.0);
        }
        self.0.LinkProgram(pipeline);
        for shader in shaders {
            self.0.DetachShader(pipeline, shader.0);
        }

        let pipeline_result = {
            let status = {
                let mut status = 0;
                self.0
                    .GetProgramiv(pipeline, __gl::LINK_STATUS, &mut status);
                status
            };

            if status == GLint::from(__gl::TRUE) {
                Ok(Pipeline(pipeline))
            } else {
                Err(Error::LinkError(Pipeline(pipeline)))
            }
        };

        if !flags.contains(PipelineFlags::VERBOSE) {
            return pipeline_result;
        }

        match pipeline_result {
            Ok(p) | Err(Error::LinkError(p)) => {
                if pipeline_result.is_err() {
                    println!("Pipeline could not be linked.");
                }
                if let Some(msg) = self.get_pipeline_log(p) {
                    println!("Pipeline Info Log: {}", msg);
                }
            }
            _ => {}
        }

        pipeline_result
    }

    /// Delete a pipeline.
    pub unsafe fn delete_pipeline(&self, pipeline: Pipeline) {
        self.0.DeleteProgram(pipeline.0);
    }

    /// Delete multiple pipelines.
    pub unsafe fn delete_pipelines(&self, pipelines: &[Pipeline]) {
        for pipeline in pipelines {
            self.0.DeleteProgram(pipeline.0);
        }
    }

    /// Bind input assembly pipeline state.
    pub unsafe fn bind_input_assembly_state(&self, state: InputAssembly) {
        match state.primitive_restart {
            Some(index) => {
                self.0.Enable(__gl::PRIMITIVE_RESTART);
                self.0.PrimitiveRestartIndex(index);
            }
            None => {
                self.0.Disable(__gl::PRIMITIVE_RESTART);
            }
        }
    }

    /// Bind color blending pipeline state.
    pub unsafe fn bind_color_blend_state(&self, state: &ColorBlend) {
        for (i, attachment) in state.attachments.iter().enumerate() {
            let slot = i as u32;
            if attachment.blend_enable {
                self.0.Enablei(__gl::BLEND, slot);
                self.0.BlendEquationSeparatei(
                    slot,
                    attachment.color.blend_op as _,
                    attachment.alpha.blend_op as _,
                );
                self.0.BlendFuncSeparatei(
                    slot,
                    attachment.color.src_factor as _,
                    attachment.color.dst_factor as _,
                    attachment.alpha.src_factor as _,
                    attachment.alpha.dst_factor as _,
                );
            } else {
                self.0.Disablei(__gl::BLEND, slot);
            }
        }
    }

    /// Bind depth-stencil pipeline state.
    ///
    /// # Examples
    ///
    /// Basic `Less-Equal` depth test with write:
    ///
    /// ```no_run
    /// # unsafe {
    /// # let grr = grr::Device::new(|_| panic!(), grr::Debug::Disable);
    /// grr.bind_depth_stencil_state(&grr::DepthStencil {
    ///     depth_test: true,
    ///     depth_write: true,
    ///     depth_compare_op: grr::Compare::LessEqual,
    ///     stencil_test: false,
    ///     stencil_front: grr::StencilFace::KEEP,
    ///     stencil_back: grr::StencilFace::KEEP,
    /// });
    /// # }
    /// ```
    pub unsafe fn bind_depth_stencil_state(&self, state: &DepthStencil) {
        if state.depth_test {
            self.0.Enable(__gl::DEPTH_TEST);
            self.0.DepthMask(if state.depth_write {
                __gl::TRUE
            } else {
                __gl::FALSE
            });
            self.0.DepthFunc(state.depth_compare_op as _);
        } else {
            self.0.Disable(__gl::DEPTH_TEST);
        }

        if state.stencil_test {
            self.0.Enable(__gl::STENCIL_TEST);
            self.0.StencilFuncSeparate(
                __gl::FRONT,
                state.stencil_front.compare_op as _,
                state.stencil_front.reference as _,
                state.stencil_front.compare_mask,
            );
            self.0.StencilOpSeparate(
                __gl::FRONT,
                state.stencil_front.fail as _,
                state.stencil_front.depth_fail as _,
                state.stencil_front.pass as _,
            );
            self.0.StencilFuncSeparate(
                __gl::BACK,
                state.stencil_back.compare_op as _,
                state.stencil_back.reference as _,
                state.stencil_back.compare_mask,
            );
            self.0.StencilOpSeparate(
                __gl::BACK,
                state.stencil_back.fail as _,
                state.stencil_back.depth_fail as _,
                state.stencil_back.pass as _,
            );
        } else {
            self.0.Disable(__gl::STENCIL_TEST);
        }
    }

    /// Bind rasterization pipeline state.
    pub unsafe fn bind_rasterization_state(&self, state: &Rasterization) {
        if state.depth_clamp {
            self.0.Enable(__gl::DEPTH_CLAMP);
        } else {
            self.0.Disable(__gl::DEPTH_CLAMP);
        }

        if state.rasterizer_discard {
            self.0.Enable(__gl::RASTERIZER_DISCARD);
        } else {
            self.0.Disable(__gl::RASTERIZER_DISCARD);
        }

        let bias_primitive = match state.polygon_mode {
            PolygonMode::Point => __gl::POLYGON_OFFSET_POINT,
            PolygonMode::Line => __gl::POLYGON_OFFSET_LINE,
            PolygonMode::Fill => __gl::POLYGON_OFFSET_FILL,
        };

        if state.depth_bias {
            self.0.Enable(bias_primitive);
        } else {
            self.0.Disable(bias_primitive);
        }

        self.0
            .PolygonMode(__gl::FRONT_AND_BACK, state.polygon_mode as _);
        self.0.FrontFace(state.front_face as _);

        match state.cull_mode {
            Some(cull) => {
                self.0.Enable(__gl::CULL_FACE);
                self.0.CullFace(cull as _);
            }
            None => {
                self.0.Disable(__gl::CULL_FACE);
            }
        }
    }

    pub unsafe fn bind_multisample_state(&self, state: Option<&Multisample>) {
        match state {
            Some(state) => {
                self.0.Enable(__gl::MULTISAMPLE);

                if state.sample_shading {
                    self.0.Enable(__gl::SAMPLE_SHADING);
                    self.0.MinSampleShading(state.min_sample_shading);
                } else {
                    self.0.Disable(__gl::SAMPLE_SHADING);
                }

                self.0
                    .SampleMaski(0, (state.sample_mask & 0xFFFF_FFFF) as _);
                self.0
                    .SampleMaski(1, ((state.sample_mask >> 32) & 0xFFFF_FFFF) as _);

                if state.alpha_to_coverage {
                    self.0.Enable(__gl::SAMPLE_ALPHA_TO_COVERAGE);
                } else {
                    self.0.Disable(__gl::SAMPLE_ALPHA_TO_COVERAGE);
                }

                if state.alpha_to_one {
                    self.0.Enable(__gl::SAMPLE_ALPHA_TO_ONE);
                } else {
                    self.0.Disable(__gl::SAMPLE_ALPHA_TO_ONE);
                }
            }
            None => {
                self.0.Disable(__gl::MULTISAMPLE);
            }
        }
    }

    /// Bind a pipeline for usage.
    pub unsafe fn bind_pipeline(&self, pipeline: Pipeline) {
        self.0.UseProgram(pipeline.0);
    }
}
