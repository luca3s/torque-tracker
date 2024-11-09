struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    const vertex_outputs: array<VertexOutput, 3> = array(
        VertexOutput(vec4(-1.0, -1.0, 0.0, 1.0), vec2(0.0, 1.0)),   // bot left corner
        VertexOutput(vec4( 3.0, -1.0, 0.0, 1.0), vec2(2.0, 1.0)),   // bot right overshoot
        VertexOutput(vec4(-1.0,  3.0, 0.0, 1.0), vec2(0.0, -1.0)),  // top left overshoot
    );

    return vertex_outputs[in_vertex_index];
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
