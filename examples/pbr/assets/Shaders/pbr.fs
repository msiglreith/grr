#version 450 core

layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec2 a_uv;
layout (location = 2) in vec3 a_normal;

out vec4 f_color;

layout (binding = 0) uniform sampler2D u_albedo;
layout (binding = 1) uniform sampler2D u_normals;
layout (binding = 2) uniform sampler2D u_metalness;
layout (binding = 3) uniform sampler2D u_roughness;
layout (binding = 4) uniform sampler2D u_occlusion;

// Image Based Lighting
layout (binding = 5) uniform sampler2D u_brdf_lut;
layout (binding = 6) uniform samplerCube u_env_prefiltered;
layout (binding = 7) uniform samplerCube u_env_irradiance;

layout (location = 3) uniform vec3 u_camera_pos;

const float PI = 3.14159265359;
// ----------------------------------------------------------------------------
// Easy trick to get tangent-normals to world-space to keep PBR code simplified.
// Don't worry if you don't get what's going on; you generally want to do normal
// mapping the usual way for performance anways; I do plan make a note of this
// technique somewhere later in the normal mapping tutorial.
vec3 getNormalFromMap()
{
    vec3 tangentNormal = texture(u_normals, a_uv).xyz * 2.0 - 1.0;

    vec3 Q1  = dFdx(a_pos);
    vec3 Q2  = dFdy(a_pos);
    vec2 st1 = dFdx(a_uv);
    vec2 st2 = dFdy(a_uv);

    vec3 N = normalize(a_normal);
    vec3 T = normalize(Q1*st2.t - Q2*st1.t);
    vec3 B = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * tangentNormal);
}
// ----------------------------------------------------------------------------
float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}
// ----------------------------------------------------------------------------
vec3 fresnelSchlickRoughness(float cosTheta, vec3 F0, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(1.0 - cosTheta, 5.0);
}

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

vec3 tonemap(vec3 color) {
    return color / (color + 1.0);
}

void main() {
    vec3 albedo = texture(u_albedo, a_uv).rgb;
    vec3 normals = texture(u_normals, a_uv).rgb;
    float metalness = texture(u_metalness, a_uv).r;
    float roughness = texture(u_roughness, a_uv).r;
    float ao = texture(u_occlusion, a_uv).r;

    vec3 N = getNormalFromMap();
    vec3 V = normalize(u_camera_pos - a_pos);
    vec3 R = reflect(-V, N);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metalness);

    vec3 F = fresnelSchlickRoughness(max(dot(N, V), 0.0), F0, roughness);

    vec3 kS = F;
    vec3 kD = 1.0 - kS;
    kD *= 1.0 - metalness;

    vec3 irradiance = texture(u_env_irradiance, N).rgb;
    vec3 diffuse = irradiance * albedo;

    // sample both the pre-filter map and the BRDF lut and combine them together as per the Split-Sum approximation to get the IBL specular part.
    const float MAX_REFLECTION_LOD = 4.0;
    vec3 prefilteredColor = textureLod(u_env_prefiltered, R, roughness * MAX_REFLECTION_LOD).rgb;
    vec2 brdf = texture(u_brdf_lut, vec2(max(dot(N, V), 0.0), roughness)).rg;
    vec3 specular = prefilteredColor * (F * brdf.x + brdf.y);

    vec3 ambient = (kD * diffuse + specular) * ao;

    f_color = vec4(tonemap(ambient), 1.0);
}
