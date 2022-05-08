struct CameraUniform {
    view_projection_matrix: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct MeshUniform {
    world_transform: mat4x4<f32>;
};

[[group(2), binding(0)]]
var<uniform> mesh_uniform: MeshUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec3<f32>;
    [[location(2)]] texture_coordinates: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
    [[location(1)]] texture_coordinates: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    vertex_input: VertexInput,
) -> VertexOutput {
    var vertex_output: VertexOutput;
    vertex_output.texture_coordinates = vertex_input.texture_coordinates;
    vertex_output.color = vertex_input.color;
    vertex_output.clip_position = camera.view_projection_matrix * mesh_uniform.world_transform * vec4<f32>(vertex_input.position, 1.0);
    return vertex_output;
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;

[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(vertex_output: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(vertex_output.color, 1.0) * textureSample(t_diffuse, s_diffuse, vertex_output.texture_coordinates);
}
