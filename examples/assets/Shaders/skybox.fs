#version 450 core

layout (location = 0) in vec3 a_view_dir;

layout (binding = 0) uniform samplerCube u_skybox;

out vec4 f_color;

vec3 tonemap(vec3 color) {
    return color / (color + 1.0);
}

void main() {
    vec3 env_color = textureLod(u_skybox, a_view_dir, 0.0).rgb;
    f_color = vec4(tonemap(env_color), 1.0);
}
