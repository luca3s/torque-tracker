struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    // coordinate space: x,y from -1 to 1, (0,0) middle
    var positions = array<vec4<f32>, 3>(
        vec4(-1.0, -1.0, 0.0, 1.0),   // bot left corner
        vec4( 3.0, -1.0, 0.0, 1.0),   // bot right overshoot
        vec4(-1.0,  3.0, 0.0, 1.0),   // top left overshoot
    );

    // coordinate space: x,y from 0 to 1, (0,0) top left
    var tex_coords = array<vec2<f32>, 3>(
        vec2(0.0, 1.0),     // bot left
        vec2(2.0, 1.0),     // bot right overshoot
        vec2(0.0, -1.0),    // top left overshoot
    );

    var out: VertexOutput;
    out.position = positions[in_vertex_index];
    out.tex_coords = tex_coords[in_vertex_index];
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
