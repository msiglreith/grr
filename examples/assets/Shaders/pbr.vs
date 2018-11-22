#version 450 core
layout (location = 0) in vec3 v_pos;

void main() {
    gl_Position = vec4(v_pos, 1.0);
}