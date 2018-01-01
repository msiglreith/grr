
use __gl;
use __gl::types::{GLenum, GLuint};

use Compare;
use device::Device;

use std::ops::Range;

///
pub struct Sampler(GLuint);

impl Device {
    /// Create a sampler object.
    pub fn create_sampler(&self, desc: SamplerDesc) -> Sampler {
        let mut sampler = 0;
        unsafe { self.0.CreateSamplers(1, &mut sampler); }
        self.get_error("CreateSamplers");

        // Texture min filter
        let min_filter = map_min_filter(desc.min_filter, desc.mip_map);
        unsafe { self.0.SamplerParameteri(sampler, __gl::TEXTURE_MIN_FILTER, min_filter as _); }
        self.get_error("SamplerParameteri (MIN_FILTER)");

        // Texture mag filter
        let mag_filter = match desc.mag_filter {
            Filter::Nearest => __gl::NEAREST,
            Filter::Linear => __gl::LINEAR,
        };
        unsafe { self.0.SamplerParameteri(sampler, __gl::TEXTURE_MAG_FILTER, mag_filter as _); }
        self.get_error("SamplerParameteri (MAG_FILTER)");

        // Texture address wrap modes
        let (wrap_s, wrap_t, wrap_r) = (
            map_sampler_address(desc.address.0),
            map_sampler_address(desc.address.1),
            map_sampler_address(desc.address.2),
        );
        unsafe { self.0.SamplerParameteri(sampler, __gl::TEXTURE_WRAP_S, wrap_s as _); }
        self.get_error("SamplerParameteri (WRAP_S)");
        unsafe { self.0.SamplerParameteri(sampler, __gl::TEXTURE_WRAP_T, wrap_t as _); }
        self.get_error("SamplerParameteri (WRAP_T)");
        unsafe { self.0.SamplerParameteri(sampler, __gl::TEXTURE_WRAP_R, wrap_r as _); }
        self.get_error("SamplerParameteri (WRAP_R)");

        // LOD bias
        unsafe { self.0.SamplerParameterf(sampler, __gl::TEXTURE_LOD_BIAS, desc.lod_bias); }
        self.get_error("SamplerParameterf (LOD_BIAS)");

        // LOD range
        unsafe { self.0.SamplerParameterf(sampler, __gl::TEXTURE_MIN_LOD, desc.lod.start); }
        self.get_error("SamplerParameterf (MIN_LOD)");
        unsafe { self.0.SamplerParameterf(sampler, __gl::TEXTURE_MAX_LOD, desc.lod.end); }
        self.get_error("SamplerParameterf (MAX_LOD)");

        // Texture comparison mode
        let (compare_mode, compare_op): (GLenum, Option<GLenum>) = match desc.compare {
            Some(op) => (__gl::COMPARE_REF_TO_TEXTURE, Some(op.into())),
            None => (__gl::NONE, None),
        };
        unsafe { self.0.SamplerParameteri(sampler, __gl::TEXTURE_COMPARE_MODE, compare_mode as _); }
        self.get_error("SamplerParameteri (COMPARE_MODE)");

        if let Some(op) = compare_op {
            unsafe { self.0.SamplerParameteri(sampler, __gl::TEXTURE_COMPARE_FUNC, op as _); }
            self.get_error("SamplerParameteri (COMPARE_FUNC)");
        }

        // Border color
        unsafe { self.0.SamplerParameterfv(sampler, __gl::TEXTURE_BORDER_COLOR, desc.border_color.as_ptr()); }
        self.get_error("SamplerParameterfv (BORDER_COLOR)");

        Sampler(sampler)
    }

    /// Bind a sampler to a specific texture unit.
    pub fn bind_sampler(&self, unit: u32, sampler: &Sampler) {
        unsafe { self.0.BindSampler(unit, sampler.0); }
        self.get_error("BindSampler");
    }

    /// Delete a sampler.
    pub fn delete_sampler(&self, sampler: Sampler) {
        unsafe { self.0.DeleteSamplers(1, &sampler.0); }
    }
}

///
#[derive(Debug, Clone)]
pub struct SamplerDesc {
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub mip_map: Option<Filter>,
    pub address: (SamplerAddress, SamplerAddress, SamplerAddress),
    pub lod_bias: f32,
    pub lod: Range<f32>,
    pub compare: Option<Compare>,
    pub border_color: [f32; 4],
}

///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Linear,
}

fn map_min_filter(filter: Filter, mip_map: Option<Filter>) -> GLenum {
    match (filter, mip_map) {
        (Filter::Nearest, None)                  => __gl::NEAREST,
        (Filter::Nearest, Some(Filter::Nearest)) => __gl::NEAREST_MIPMAP_NEAREST,
        (Filter::Nearest, Some(Filter::Linear))  => __gl::NEAREST_MIPMAP_LINEAR,
        (Filter::Linear,  None)                  => __gl::LINEAR,
        (Filter::Linear,  Some(Filter::Nearest)) => __gl::LINEAR_MIPMAP_NEAREST,
        (Filter::Linear,  Some(Filter::Linear))  => __gl::LINEAR_MIPMAP_LINEAR,
    }
}

///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SamplerAddress {
    Repeat,
    MirrorRepeat,
    ClampEdge,
    ClampBorder,
    MirrorClampEdge,
}

fn map_sampler_address(address: SamplerAddress) -> GLenum {
    match address {
        SamplerAddress::Repeat => __gl::REPEAT,
        SamplerAddress::MirrorRepeat => __gl::MIRRORED_REPEAT,
        SamplerAddress::ClampEdge => __gl::CLAMP_TO_EDGE,
        SamplerAddress::ClampBorder => __gl::CLAMP_TO_BORDER,
        SamplerAddress::MirrorClampEdge => __gl::MIRROR_CLAMP_TO_EDGE,
    }
}
