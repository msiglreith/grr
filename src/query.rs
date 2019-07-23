/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::__gl;
use crate::__gl::types::GLuint;

use crate::device::Device;

///
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QueryType {
    ///
    Timestamp = __gl::TIMESTAMP,
    ///
    TimeElapsed = __gl::TIME_ELAPSED,
    ///
    Occlusion = __gl::ANY_SAMPLES_PASSED,
    ///
    OcclusionConservative = __gl::ANY_SAMPLES_PASSED_CONSERVATIVE,
    ///
    OcclusionPrecision = __gl::SAMPLES_PASSED,
    ///
    InputAssemblyVertices = __gl::VERTICES_SUBMITTED,
    ///
    InputAssemblyPrimitives = __gl::PRIMITIVES_SUBMITTED,
    /// Number of vertex shader invocations.
    VertexShaderInvocations = __gl::VERTEX_SHADER_INVOCATIONS,
    /// Number of geometry shader invocations.
    GeometryShaderInvocations = __gl::GEOMETRY_SHADER_INVOCATIONS,
    ///
    GeometryShaderPrimitives = __gl::GEOMETRY_SHADER_PRIMITIVES_EMITTED,
    ///
    TransformFeedbackPrimitivesWritten = __gl::TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN,
    ///
    TransformFeedbackOverflow = __gl::TRANSFORM_FEEDBACK_OVERFLOW,
    ///
    TransformFeedbackStreamOverflow = __gl::TRANSFORM_FEEDBACK_STREAM_OVERFLOW,
    /// Number of input primitives for the primitive clipping stage.
    ClippingInvocations = __gl::CLIPPING_INPUT_PRIMITIVES,
    /// Number of output primitives for the primitive clipping stage.
    ClippingPrimitives = __gl::CLIPPING_OUTPUT_PRIMITIVES,
    /// Number of fragment shader invocations.
    FragmentShaderInvocations = __gl::FRAGMENT_SHADER_INVOCATIONS,
    /// Number of patches processed by tessellation control shader.
    TessellationControlShaderPatches = __gl::TESS_CONTROL_SHADER_PATCHES,
    /// Number of tessellation evaluation shader invocations.
    TessellationEvaluationShaderInvocations = __gl::TESS_EVALUATION_SHADER_INVOCATIONS,
    /// Number of compute shader invocations.
    ComputeShaderInvocations = __gl::COMPUTE_SHADER_INVOCATIONS,
}

///
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConditionalMode {
    ///
    NoWait = __gl::QUERY_NO_WAIT,
    ///
    NoWaitInverted = __gl::QUERY_NO_WAIT_INVERTED,
    /// Wait for query results available.
    Wait = __gl::QUERY_WAIT,
    /// Wait for query results available (inverted condition).
    WaitInverted = __gl::QUERY_WAIT_INVERTED,
    ///
    WaitByRegion = __gl::QUERY_BY_REGION_WAIT,
    ///
    WaitByRegionInverted = __gl::QUERY_BY_REGION_WAIT_INVERTED,
}

pub struct Query {
    raw: GLuint,
    ty: QueryType,
}

impl Device {
    pub unsafe fn create_query(&self, ty: QueryType) -> Query {
        let mut query = 0;
        self.0.CreateQueries(ty as _, 1, &mut query as *mut _);
        Query { raw: query, ty }
    }

    pub unsafe fn begin_query(&self, query: &Query) {
        let index = match query.ty {
            _ => 0,
        };

        self.0.BeginQueryIndexed(query.ty as _, index, query.raw);
    }

    pub unsafe fn end_query(&self, query: &Query) {
        let index = match query.ty {
            _ => 0,
        };

        self.0.EndQueryIndexed(query.ty as _, index);
    }

    pub unsafe fn write_timestamp(&self, query: &Query) {
        self.0.QueryCounter(query.raw, __gl::TIMESTAMP);
    }

    pub unsafe fn begin_conditional_rendering(&self, query: &Query, mode: ConditionalMode) {
        self.0.BeginConditionalRender(query.raw, mode as _);
    }

    pub unsafe fn end_conditional_rendering(&self) {
        self.0.EndConditionalRender();
    }
}
