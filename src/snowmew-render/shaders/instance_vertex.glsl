#version 440
#extension GL_ARB_shader_draw_parameters: require

uniform int instance_offset;
uniform mat4 mat_view;
uniform mat4 mat_proj;

uniform samplerBuffer mat_model0;
uniform samplerBuffer mat_model1;
uniform samplerBuffer mat_model2;
uniform samplerBuffer mat_model3;

uniform usamplerBuffer info;

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;
in uint in_draw_id;

out vec2 fs_texture;
out vec3 fs_normal;
flat out uint fs_object_id;
flat out uint fs_material_id;

void main() {
    uvec4 info = texelFetch(info, gl_DrawIDARB + gl_InstanceID);
    int matrix_id = int(info.y);
    mat4 mat_model = mat4(texelFetch(mat_model0, matrix_id),
                          texelFetch(mat_model1, matrix_id),
                          texelFetch(mat_model2, matrix_id),
                          texelFetch(mat_model3, matrix_id));

    vec4 normal = mat_model * vec4(in_normal, 0.);
    gl_Position = mat_proj * mat_view * mat_model * vec4(in_position, 1.);

    fs_texture = in_texture;
    fs_normal = normalize(normal).xyz;
    fs_material_id = info.z;
    fs_object_id = info.x;
}