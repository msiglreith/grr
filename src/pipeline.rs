use __gl;

use device::Device;
use std::ops::Range;
use Compare;

///
#[derive(Debug)]
pub struct InputAssembly {
    pub primitive_restart: Option<u32>,
}

///
#[derive(Debug)]
pub struct Rasterization {
    pub depth_clamp: bool,
    pub rasterizer_discard: bool,
    pub polygon_mode: PolygonMode,
    pub cull_mode: Option<CullMode>,
    pub front_face: FrontFace,
    pub depth_bias: bool,
}

///
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PolygonMode {
    Point = __gl::POINT,
    Line = __gl::LINE,
    Fill = __gl::FILL,
}

///
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CullMode {
    Front = __gl::FRONT,
    Back = __gl::BACK,
    FrontBack = __gl::FRONT_AND_BACK,
}

///
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FrontFace {
    CounterClockwise = __gl::CCW,
    Clockwise = __gl::CW,
}

///
#[derive(Debug)]
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
pub struct DepthStencil {
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_compare_op: Compare,
    pub stencil_test: bool,
    pub stencil_front: StencilFace,
    pub stencil_back: StencilFace,
}

impl Device {
    pub fn bind_input_assembly_state(&self, state: &InputAssembly) {
        match state.primitive_restart {
            Some(index) => unsafe {
                self.0.Enable(__gl::PRIMITIVE_RESTART);
                self.get_error("Enable (Primitive Restart)");
                self.0.PrimitiveRestartIndex(index);
                self.get_error("PrimitiveRestartIndex");
            },
            None => unsafe {
                self.0.Disable(__gl::PRIMITIVE_RESTART);
                self.get_error("Disable (Primitive Restart)");
            },
        }
    }

    pub fn bind_color_blend_state(&self, state: &ColorBlend) {
        for (i, attachment) in state.attachments.iter().enumerate() {
            let slot = i as u32;
            if attachment.blend_enable {
                unsafe {
                    self.0.Enablei(__gl::BLEND, slot);
                    self.get_error("Enable (Blend)");
                    self.0.BlendEquationSeparatei(
                        slot,
                        attachment.color.blend_op as _,
                        attachment.alpha.blend_op as _,
                    );
                    self.get_error("BlendEquationSeparatei");
                    self.0.BlendFuncSeparatei(
                        slot,
                        attachment.color.src_factor as _,
                        attachment.color.dst_factor as _,
                        attachment.alpha.src_factor as _,
                        attachment.alpha.dst_factor as _,
                    );
                    self.get_error("BlendFuncSeparatei");
                }
            } else {
                unsafe {
                    self.0.Disablei(__gl::BLEND, slot);
                    self.get_error("Disable (Blend)");
                }
            }
        }
    }

    pub fn bind_depth_stencil_state(&self, state: &DepthStencil) {
        if state.depth_test {
            unsafe {
                self.0.Enable(__gl::DEPTH_TEST);
                self.get_error("Enable (Depth Test)");
                self.0.DepthMask(if state.depth_write {
                    __gl::TRUE
                } else {
                    __gl::FALSE
                });
                self.get_error("DepthMask");
                self.0.DepthFunc(state.depth_compare_op as _);
                self.get_error("DepthFunc");
            }
        } else {
            unsafe {
                self.0.Disable(__gl::DEPTH_TEST);
            }
            self.get_error("Disable (Depth Test)");
        }

        if state.stencil_test {
            unsafe {
                self.0.Enable(__gl::STENCIL_TEST);
                self.get_error("Enable (Stencil Test)");
                self.0.StencilFuncSeparate(
                    __gl::FRONT,
                    state.stencil_front.compare_op as _,
                    state.stencil_front.reference as _,
                    state.stencil_front.compare_mask,
                );
                self.get_error("StencilFuncSeparate (Front)");
                self.0.StencilOpSeparate(
                    __gl::FRONT,
                    state.stencil_front.fail as _,
                    state.stencil_front.depth_fail as _,
                    state.stencil_front.pass as _,
                );
                self.get_error("StencilOpSeparate (Front)");
                self.0.StencilFuncSeparate(
                    __gl::BACK,
                    state.stencil_back.compare_op as _,
                    state.stencil_back.reference as _,
                    state.stencil_back.compare_mask,
                );
                self.get_error("StencilFuncSeparate (Back)");
                self.0.StencilOpSeparate(
                    __gl::BACK,
                    state.stencil_back.fail as _,
                    state.stencil_back.depth_fail as _,
                    state.stencil_back.pass as _,
                );
                self.get_error("StencilOpSeparate (Back)");
            }
        } else {
            unsafe {
                self.0.Disable(__gl::STENCIL_TEST);
            }
            self.get_error("Disable (Stencil Test)");
        }
    }

    pub fn bind_rasterization_state(&self, state: &Rasterization) {
        if state.depth_clamp {
            unsafe {
                self.0.Enable(__gl::DEPTH_CLAMP);
            }
            self.get_error("Enable (Depth Clamp)");
        } else {
            unsafe {
                self.0.Disable(__gl::DEPTH_CLAMP);
            }
            self.get_error("Disable (Depth Clamp)");
        }

        if state.rasterizer_discard {
            unsafe {
                self.0.Enable(__gl::RASTERIZER_DISCARD);
            }
            self.get_error("Enable (Rasterizer Discard)");
        } else {
            unsafe {
                self.0.Disable(__gl::RASTERIZER_DISCARD);
            }
            self.get_error("Disable (Rasterizer Discard)");
        }

        let bias_primitive = match state.polygon_mode {
            PolygonMode::Point => __gl::POLYGON_OFFSET_POINT,
            PolygonMode::Line => __gl::POLYGON_OFFSET_LINE,
            PolygonMode::Fill => __gl::POLYGON_OFFSET_FILL,
        };

        if state.depth_bias {
            unsafe {
                self.0.Enable(bias_primitive);
            }
            self.get_error("Enable (Depth Bias)");
        } else {
            unsafe {
                self.0.Disable(bias_primitive);
            }
            self.get_error("Disable (Depth Bias)");
        }

        unsafe {
            self.0
                .PolygonMode(__gl::FRONT_AND_BACK, state.polygon_mode as _);
            self.get_error("PolygonMode");
            self.0.FrontFace(state.front_face as _);
            self.get_error("PolygonMode");
        }

        match state.cull_mode {
            Some(cull) => unsafe {
                self.0.Enable(__gl::CULL_FACE);
                self.get_error("Enable (Cull Face)");
                self.0.CullFace(cull as _);
                self.get_error("CullFace");
            },
            None => unsafe {
                self.0.Disable(__gl::CULL_FACE);
                self.get_error("Disable (Cull Face)");
            },
        }
    }
}
