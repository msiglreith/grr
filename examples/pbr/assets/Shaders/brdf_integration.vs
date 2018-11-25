#version 450 core

layout (location = 0) out vec2 a_uv;

vec2 screen_space_triangle_pos() {
    return vec2(
        float((gl_VertexID & 1) << 2) - 1.0,
        float((gl_VertexID & 2) << 1) - 1.0
    );
}

vec2 screen_space_triangle_uv() {
    return vec2(
        float((gl_VertexID & 1) << 2) * 0.5,
        float((gl_VertexID & 2) << 1) * 0.5
    );
}

void main() {
    a_uv = screen_space_triangle_uv();
    gl_Position = vec4(screen_space_triangle_pos(), 0.0, 1.0);
}