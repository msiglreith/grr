/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::__gl;
use crate::__gl::types::GLuint;

use crate::buffer::Buffer;
use crate::debug::{Object, ObjectType};
use crate::device::Device;
use crate::error::Result;

/// Vertex array handle.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct VertexArray(pub(crate) GLuint);

impl Object for VertexArray {
    const TYPE: ObjectType = ObjectType::VertexArray;
    fn handle(&self) -> GLuint {
        self.0
    }
}

/// Buffer representation for vertex attributes
pub struct VertexBufferView {
    pub buffer: Buffer,
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

/// Vertex attribute addresssing.
///
/// Specifies if the vertex attribute address depends on vertex index or instance index.
pub enum InputRate {
    /// Vertex index addressing.
    ///
    /// Attribute data is fetched from the bound vertex buffers depending on the current vertex index.
    Vertex,
    /// Instance index addressing.
    ///
    /// Attribute data is fetched from the bound vertex buffers depending on the current instance index.
    /// The `divisor` further defines how many consecutive instances will use the same vertex attribute data.
    /// The instance index will be divided by the divisor to donate the addressing index.
    Instance { divisor: usize },
}

/// Vertex attribute formats.
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

impl Device {
    /// Create a new vertex array, storing information for the input assembler.
    ///
    /// The vertex array specified the vertex attributes and their binding to
    /// vertex buffer objects.
    pub unsafe fn create_vertex_array(
        &self,
        attributes: &[VertexAttributeDesc],
    ) -> Result<VertexArray> {
        let mut vao = 0;
        self.0.CreateVertexArrays(1, &mut vao);
        self.get_error()?;

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

            self.0.EnableVertexArrayAttrib(vao, desc.location);
            match base {
                VertexBase::Int => {
                    self.0
                        .VertexArrayAttribIFormat(vao, desc.location, num, ty, desc.offset);
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
                }
                VertexBase::Double => {
                    self.0
                        .VertexArrayAttribLFormat(vao, desc.location, num, ty, desc.offset);
                }
            }

            self.0
                .VertexArrayAttribBinding(vao, desc.location, desc.binding);
        }

        Ok(VertexArray(vao))
    }

    /// Delete a vertex array.
    pub unsafe fn delete_vertex_array(&self, vao: VertexArray) {
        self.delete_vertex_arrays(&[vao]);
    }

    /// Delete multiple vertex arrays.
    pub unsafe fn delete_vertex_arrays(&self, vao: &[VertexArray]) {
        self.0.DeleteVertexArrays(
            vao.len() as _,
            vao.as_ptr() as *const _, // newtype
        );
    }

    /// Bind a vertex array for usage.
    pub unsafe fn bind_vertex_array(&self, vao: VertexArray) {
        self.0.BindVertexArray(vao.0);
    }

    /// Bind vertex buffers to a vertex array.
    pub unsafe fn bind_vertex_buffers(
        &self,
        vao: VertexArray,
        first: u32,
        views: &[VertexBufferView],
    ) {
        let buffers = views.iter().map(|view| view.buffer.0).collect::<Vec<_>>();

        let offsets = views
            .iter()
            .map(|view| view.offset as _)
            .collect::<Vec<_>>();

        let strides = views
            .iter()
            .map(|view| view.stride as _)
            .collect::<Vec<_>>();

        self.0.VertexArrayVertexBuffers(
            vao.0,
            first,
            views.len() as _,
            buffers.as_ptr(),
            offsets.as_ptr(),
            strides.as_ptr(),
        );

        for (binding, view) in views.iter().enumerate() {
            let divisor = match view.input_rate {
                InputRate::Vertex => 0,
                InputRate::Instance { divisor } => divisor,
            };

            self.0
                .VertexArrayBindingDivisor(vao.0, first + binding as u32, divisor as _);
        }
    }

    /// Bind a index buffer to a vertex array.
    pub unsafe fn bind_index_buffer(&self, vao: VertexArray, buffer: Buffer) {
        self.0.VertexArrayElementBuffer(vao.0, buffer.0);
    }
}
