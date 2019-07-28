/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Sampler.

use crate::__gl;
use crate::__gl::types::{GLenum, GLuint};

use crate::debug::{Object, ObjectType};
use crate::device::Device;
use crate::error::Result;
use crate::Compare;

use std::ops::Range;

/// Sampler handle.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Sampler(GLuint);

impl Object for Sampler {
    const TYPE: ObjectType = ObjectType::Sampler;
    fn handle(&self) -> GLuint {
        self.0
    }
}

impl Device {
    /// Create a sampler object.
    pub unsafe fn create_sampler(&self, desc: SamplerDesc) -> Result<Sampler> {
        let mut sampler = 0;
        self.0.CreateSamplers(1, &mut sampler);
        self.get_error()?;

        // Texture min filter
        let min_filter = map_min_filter(desc.min_filter, desc.mip_map);
        self.0
            .SamplerParameteri(sampler, __gl::TEXTURE_MIN_FILTER, min_filter as _);

        // Texture mag filter
        let mag_filter = match desc.mag_filter {
            Filter::Nearest => __gl::NEAREST,
            Filter::Linear => __gl::LINEAR,
        };
        self.0
            .SamplerParameteri(sampler, __gl::TEXTURE_MAG_FILTER, mag_filter as _);

        // Texture address wrap modes
        self.0
            .SamplerParameteri(sampler, __gl::TEXTURE_WRAP_S, desc.address.0 as _);
        self.0
            .SamplerParameteri(sampler, __gl::TEXTURE_WRAP_T, desc.address.1 as _);
        self.0
            .SamplerParameteri(sampler, __gl::TEXTURE_WRAP_R, desc.address.2 as _);

        // LOD bias
        self.0
            .SamplerParameterf(sampler, __gl::TEXTURE_LOD_BIAS, desc.lod_bias);
        // LOD range
        self.0
            .SamplerParameterf(sampler, __gl::TEXTURE_MIN_LOD, desc.lod.start);
        self.0
            .SamplerParameterf(sampler, __gl::TEXTURE_MAX_LOD, desc.lod.end);

        // Texture comparison mode
        let (compare_mode, compare_op): (GLenum, Option<GLenum>) = match desc.compare {
            Some(op) => (__gl::COMPARE_REF_TO_TEXTURE, Some(op as _)),
            None => (__gl::NONE, None),
        };
        self.0
            .SamplerParameteri(sampler, __gl::TEXTURE_COMPARE_MODE, compare_mode as _);

        if let Some(op) = compare_op {
            self.0
                .SamplerParameteri(sampler, __gl::TEXTURE_COMPARE_FUNC, op as _);
        }

        // Border color
        self.0.SamplerParameterfv(
            sampler,
            __gl::TEXTURE_BORDER_COLOR,
            desc.border_color.as_ptr(),
        );

        Ok(Sampler(sampler))
    }

    /// Bind samplers to specific texture units.
    pub unsafe fn bind_samplers(&self, first: u32, samplers: &[Sampler]) {
        let samplers = samplers.iter().map(|s| s.0).collect::<Vec<_>>();
        self.0
            .BindSamplers(first, samplers.len() as _, samplers.as_ptr());
    }

    // Delete a sampler.
    pub unsafe fn delete_sampler(&self, sampler: Sampler) {
        self.delete_samplers(&[sampler])
    }

    /// Delete multiple samplers.
    pub unsafe fn delete_samplers(&self, samplers: &[Sampler]) {
        self.0.DeleteSamplers(
            samplers.len() as _,
            samplers.as_ptr() as *const _, // newtype
        );
    }
}

/// Sampler Descriptor.
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
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest = __gl::LINEAR,
    Linear = __gl::NEAREST,
}

fn map_min_filter(filter: Filter, mip_map: Option<Filter>) -> GLenum {
    match (filter, mip_map) {
        (Filter::Nearest, None) => __gl::NEAREST,
        (Filter::Nearest, Some(Filter::Nearest)) => __gl::NEAREST_MIPMAP_NEAREST,
        (Filter::Nearest, Some(Filter::Linear)) => __gl::NEAREST_MIPMAP_LINEAR,
        (Filter::Linear, None) => __gl::LINEAR,
        (Filter::Linear, Some(Filter::Nearest)) => __gl::LINEAR_MIPMAP_NEAREST,
        (Filter::Linear, Some(Filter::Linear)) => __gl::LINEAR_MIPMAP_LINEAR,
    }
}

/// Sampler addressing mode.
///
/// Specifies how coordinates outide of the texture coordinate system (`[0, 1]`) are treated during
/// sampling operations.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SamplerAddress {
    ///
    Repeat = __gl::REPEAT,
    ///
    MirrorRepeat = __gl::MIRRORED_REPEAT,
    ///
    ClampEdge = __gl::CLAMP_TO_EDGE,
    ///
    ClampBorder = __gl::CLAMP_TO_BORDER,
    ///
    MirrorClampEdge = __gl::MIRROR_CLAMP_TO_EDGE,
}
