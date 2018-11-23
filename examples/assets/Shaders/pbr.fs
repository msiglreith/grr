#version 450 core

layout (location = 0) in vec3 a_pos;
out vec4 f_color;

void main() {
    f_color = vec4(a_pos / 40.0 + 0.5, 1.0);
}
