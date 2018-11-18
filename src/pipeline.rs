use __gl;

use device::Device;

///
pub struct InputAssembly {
    pub primitive_restart: Option<u32>,
}

///
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
}
