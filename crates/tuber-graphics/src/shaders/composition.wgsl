struct VertexStageInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec3<f32>;
    [[location(2)]] texture_coordinates: vec2<f32>;
};

struct VertexStageOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] texture_coordinates: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(input: VertexStageInput) -> VertexStageOutput {
    var output: VertexStageOutput;
    output.texture_coordinates = input.texture_coordinates;
    output.clip_position = vec4<f32>(input.position.xy, 0.0, 1.0);
    return output;
}

[[block]]
struct GlobalUniform {
    rendered_g_buffer_component: i32;
};

[[group(0), binding(0)]]
var<uniform> u_global: GlobalUniform;

[[group(1), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]]
var s_diffuse: sampler;
[[group(1), binding(2)]]
var t_normal: texture_2d<f32>;
[[group(1), binding(3)]]
var s_normal: sampler;


[[group(2), binding(0)]]
var t_ui: texture_2d<f32>;
[[group(2), binding(1)]]
var s_ui: sampler;

[[stage(fragment)]]
fn fs_main(input: VertexStageOutput) -> [[location(0)]] vec4<f32> {
    let ui_fragment = textureSample(t_ui, s_ui, input.texture_coordinates);
    let diffuse_fragment = textureSample(t_diffuse, s_diffuse, input.texture_coordinates);

    if (ui_fragment.a == 0.0) {
        return diffuse_fragment;
    } else {
        return ui_fragment;
    }
}