/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::__gl;

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Format {
    R8_UNORM = __gl::R8,
    R8G8B8A8_SRGB = __gl::RGBA8,
    R8G8B8A8_UNORM = __gl::SRGB8_ALPHA8,
    R16G16_SFLOAT = __gl::RG16F,
    R16G16B16_SFLOAT = __gl::RGB16F,
    D32_SFLOAT = __gl::DEPTH_COMPONENT32F,
    // TODO
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BaseFormat {
    R = __gl::RED,
    RG = __gl::RG,
    RGB = __gl::RGB,
    RGBA = __gl::RGBA,
    Depth = __gl::DEPTH_COMPONENT,
    DepthStencil = __gl::DEPTH_STENCIL,
    Stencil = __gl::STENCIL_INDEX,
}

#[allow(non_camel_case_types)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FormatLayout {
    U8 = __gl::UNSIGNED_BYTE,
    U16 = __gl::UNSIGNED_SHORT,
    U32 = __gl::UNSIGNED_INT,
    I8 = __gl::BYTE,
    I16 = __gl::SHORT,
    I32 = __gl::INT,
    F16 = __gl::HALF_FLOAT,
    F32 = __gl::FLOAT,
    // TODO
}
