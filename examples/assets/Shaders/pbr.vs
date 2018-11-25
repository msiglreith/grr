#version 450 core
layout (location = 0) in vec3 v_pos;
layout (location = 1) in vec2 v_uv;

layout (location = 0) out vec3 a_pos;
layout (location = 1) out vec2 a_uv;

layout (location = 0) uniform mat4 u_perspective;
layout (location = 1) uniform mat4 u_view;

void main() {
    a_pos = v_pos;
    a_uv = v_uv;
    gl_Position = u_perspective * u_view * vec4(v_pos, 1.0);
}