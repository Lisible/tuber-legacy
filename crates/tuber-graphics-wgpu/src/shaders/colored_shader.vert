#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_color;
layout(location=2) in vec2 a_tex_coords;

layout(location=3) in vec4 model_matrix_0;
layout(location=4) in vec4 model_matrix_1;
layout(location=5) in vec4 model_matrix_2;
layout(location=6) in vec4 model_matrix_3;
layout(location=7) in vec3 color;
layout(location=8) in vec2 size;
layout(location=10) in int apply_view_transform;

layout(location=0) out vec3 v_color;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_proj;
    mat4 u_view;
};

void main() {
    mat4 model_matrix = mat4 (
        model_matrix_0,
        model_matrix_1,
        model_matrix_2,
        model_matrix_3
    );

    v_color = color;
    mat4 view_proj;
    if (apply_view_transform != 0) {
        view_proj = u_proj * u_view;
    } else {
        view_proj = u_proj;
    }
    gl_Position = view_proj * model_matrix * vec4(a_position.x * size.x, a_position.y * size.y, 0.0, 1.0);
}