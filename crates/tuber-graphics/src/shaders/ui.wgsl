[[block]]
struct GlobalUniform {
    view_projection: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> u_global: GlobalUniform;

[[block]]
struct QuadUniform {
    model: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> u_quad: QuadUniform;

struct VertexStageInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec3<f32>;
    [[location(2)]] texture_coordinates: vec2<f32>;
};

struct VertexStageOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
    [[location(1)]] texture_coordinates: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(input: VertexStageInput) -> VertexStageOutput {
    var output: VertexStageOutput;
    output.color = input.color;
    output.texture_coordinates = input.texture_coordinates;
    output.clip_position = u_global.view_projection * u_quad.model * vec4<f32>(input.position.x, input.position.y, 0.0, 1.0);
    return output;
}


[[group(2), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(2), binding(1)]]
var s_diffuse: sampler;


struct FragmentStageOutput {
    [[location(0)]] albedo: vec4<f32>;
    [[location(1)]] normal: vec4<f32>;
    [[location(2)]] emission: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(input: VertexStageOutput) -> FragmentStageOutput {
    var output: FragmentStageOutput;
    output.albedo = textureSample(t_diffuse, s_diffuse, input.texture_coordinates);
    output.normal = vec4<f32>(0.0);
    output.emission = vec4<f32>(0.0);
    return output;
}