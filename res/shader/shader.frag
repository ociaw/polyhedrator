#version 450

layout(location=0) in vec2 v_tex_coords;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec3 v_frag_pos;

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
    vec4 u_light_pos;
    vec4 u_light_color;
};

void main() {
    float ambient_intensity = 0.15;
    vec3 ambient_color = vec3(1.0, 1.0, 1.0);
    vec3 ambient = ambient_intensity * ambient_color;

    vec3 normal = normalize(v_normal);

    float sun_intensity = 0.85;
    vec3 sun_direction = normalize(u_light_pos.xyz - v_frag_pos);
    float sun_light = (dot(normal, sun_direction) + 1.0) / 2.0;
    vec3 sun_diffuse = sun_intensity * sun_light * vec3(1.0, 1.0, 1.0);

    vec3 lighting = (ambient + sun_diffuse) * u_light_color.xyz;

    f_color = vec4(lighting, 0.0) * texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
}
