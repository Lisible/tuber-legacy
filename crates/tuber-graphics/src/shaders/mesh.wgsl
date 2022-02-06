[[block]]
struct MeshUniform {
    model: mat4x4<f32>;
    view: mat4x4<f32>;
    projection: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> u_mesh: MeshUniform;

struct VertexStageInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] texture_coordinates: vec2<f32>;
};

struct VertexStageOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] texture_coordinates: vec2<f32>;
    [[location(1)]] world_position: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(input: VertexStageInput) -> VertexStageOutput {
    var output: VertexStageOutput;
    output.texture_coordinates = input.texture_coordinates;

    let world_position = u_mesh.model * vec4<f32>(input.position.x, input.position.y, 0.0, 1.0);
    output.world_position = world_position.xyz;
    output.clip_position = u_mesh.projection * u_mesh.view  * world_position;
    return output;
}


[[group(1), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]]
var s_diffuse: sampler;
[[group(1), binding(2)]]
var t_normal: texture_2d<f32>;
[[group(1), binding(3)]]
var s_normal: sampler;
[[group(1), binding(4)]]
var t_emission: texture_2d<f32>;
[[group(1), binding(5)]]
var s_emission: sampler;


struct FragmentStageOutput {
    [[location(0)]] albedo: vec4<f32>;
    [[location(1)]] normal: vec4<f32>;
    [[location(2)]] emission: vec4<f32>;
    [[location(3)]] position: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(input: VertexStageOutput) -> FragmentStageOutput {
    var output: FragmentStageOutput;
    output.albedo = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    output.normal = textureSample(t_normal, s_normal, input.texture_coordinates);
    output.emission = textureSample(t_emission, s_emission, input.texture_coordinates);
    output.position = vec4<f32>(input.world_position.xyz, 1.0);
    return output;
}