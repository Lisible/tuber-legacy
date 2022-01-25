struct VertexStageInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec3<f32>;
    [[location(2)]] texture_coordinates: vec2<f32>;
};

struct VertexStageOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] texture_coordinates: vec2<f32>;
};

[[block]]
struct PointLightUniform {
    position: vec3<f32>;
    ambient_color: vec3<f32>;
    diffuse_color: vec3<f32>;
    specular_color: vec3<f32>;
    constant: f32;
    linear: f32;
    quadratic: f32;
};

[[group(1), binding(0)]]
var<uniform> light: PointLightUniform;

[[stage(vertex)]]
fn vs_main(input: VertexStageInput) -> VertexStageOutput {
    var output: VertexStageOutput;
    output.texture_coordinates = input.texture_coordinates;
    output.clip_position = vec4<f32>(input.position.xy, 0.0, 1.0);
    return output;
}

[[group(0), binding(0)]]
var t_albedo: texture_2d<f32>;
[[group(0), binding(1)]]
var s_albedo: sampler;
[[group(0), binding(2)]]
var t_normal: texture_2d<f32>;
[[group(0), binding(3)]]
var s_normal: sampler;
[[group(0), binding(4)]]
var t_position: texture_2d<f32>;
[[group(0), binding(5)]]
var s_position: sampler;

[[stage(fragment)]]
fn fs_main(input: VertexStageOutput) -> [[location(0)]] vec4<f32> {
    let light_position = vec3<f32>(300.0, 300.0, 200.0);
    let frag_position = textureSample(t_position, s_position, input.texture_coordinates).rgb;
    let normal = textureSample(t_normal, s_normal, input.texture_coordinates).rgb * 2.0 - vec3<f32>(1.0);
    let albedo = textureSample(t_albedo, s_albedo, input.texture_coordinates).rgb;

    var lighting = albedo * 0.05;
    let light_direction = normalize(light_position - frag_position);
    var diffuse = light.diffuse_color * max(dot(normal, light_direction), 0.0) * albedo;
    lighting = lighting + diffuse;

    return vec4<f32>(diffuse, 1.0);
}