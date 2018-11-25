#version 450 core
layout (location = 0) in vec3 v_pos;
layout (location = 1) in vec2 v_uv;
layout (location = 2) in vec3 v_normal;

layout (location = 0) out vec3 a_pos;
layout (location = 1) out vec2 a_uv;
layout (location = 2) out vec3 a_normal;

layout (location = 0) uniform mat4 u_perspective;
layout (location = 1) uniform mat4 u_view;
layout (location = 2) uniform mat4 u_model;

void main() {
    a_pos = v_pos;
    a_uv = v_uv;
    a_normal = v_normal;
    gl_Position = u_perspective * u_view * u_model * vec4(v_pos, 1.0);
}