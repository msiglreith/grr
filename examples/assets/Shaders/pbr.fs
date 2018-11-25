#version 450 core

layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec2 a_uv;

out vec4 f_color;

layout (binding = 0) uniform sampler2D u_albedo;
layout (binding = 1) uniform sampler2D u_normals;

float srgb_to_linear(float x) {
    if (x <= 0.04045) {
        return x / 12.92;
    } else {
        return pow((x + 0.055) / 1.055, 2.4);
    }
}

vec3 srgb_to_linear(vec3 v) {
    return vec3(srgb_to_linear(v.x), srgb_to_linear(v.y), srgb_to_linear(v.z));
}

void main() {
    vec3 albedo = texture(u_albedo, a_uv).rgb;
    vec3 normals = texture(u_normals, a_uv).rgb;
    f_color = vec4(srgb_to_linear(normals * 0.5 + 0.5), 1.0); // visualize normals
}
