use __gl;
use __gl::types::{GLenum, GLuint};

pub enum QueryType {
    Occlusion = __gl::ANY_SAMPLES_PASSED,
    OcclusionConservative = __gl::ANY_SAMPLES_PASSED_CONSERVATIVE,
    OcclusionPrecis = __gl::SAMPLES_PASSED,
    InputAssemblyVertices = __gl::VERTICES_SUBMITTED,
    InputAssemblyPrimitives = __gl::PRIMITIVES_SUBMITTED,
    VertexShaderInvocations = __gl::VERTEX_SHADER_INVOCATIONS,
    GeometryShaderInvocations = __gl::GEOMETRY_SHADER_INVOCATIONS,
    GeometryShaderPrimitives = __gl::GEOMETRY_SHADER_PRIMTIIVES,
}

pub struct Query {
    raw: GLuint,
    ty: QueryType,
}

impl Device {
    pub fn create_query(&self, ty: QueryType) -> Query {
        let mut query = 0;
        unsafe {
            self.0.CreateQueries(1, &mut query as *mut _);
        }

        Query(query)
    }

    pub fn begin_query(&self, query: &Query) {
        let index = match query.ty {
            _ => 0,
        };

        unsafe {
            self.0.BeginQueryIndexed(query.ty as _, index, query.raw);
        }
    }

    pub fn end_query(&self) {}

    pub fn write_timestamp(&self, query: &Query) {}

    pub fn begin_conditional_rendering(&self) {}
}
