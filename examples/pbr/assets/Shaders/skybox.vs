#version 450 core

layout (location = 0) out vec3 a_view_dir;

layout(location = 0) uniform mat4 u_inv_proj;
layout(location = 1) uniform mat3 u_inv_view;

vec2 screen_space_triangle() {
    return vec2(
        float((gl_VertexID & 1) << 2) - 1.0,
        float((gl_VertexID & 2) << 1) - 1.0
    );
}

void main() {
    vec4 pos = vec4(screen_space_triangle(), 0.0, 1.0);
    a_view_dir = u_inv_view * (u_inv_proj * pos).xyz;
    gl_Position = pos;
}