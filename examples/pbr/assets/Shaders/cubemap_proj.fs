#version 450 core

const float PI = 3.141592653;

layout (location = 0) in vec3 a_view_dir;

layout (binding = 0) uniform sampler2D u_envmap;

out vec4 f_color;

vec2 sample_spherical_map(vec3 dir) {
    vec2 uv = vec2(atan(dir.z, dir.x) / (2 * PI), asin(dir.y) / PI);
    uv += 0.5;
    return uv;
}

void main() {
    vec2 uv = sample_spherical_map(normalize(a_view_dir));
    f_color = vec4(texture(u_envmap, uv).rgb, 1.0);
}
