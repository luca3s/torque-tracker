struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

var<private> positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(-1.0, -1.0),   // bottom left corner i think?
    vec2( 1.0, -1.0),   // bottom right
    vec2(-1.0,  1.0),   // top left

    vec2( 1.0, -1.0),   // bottom right
    vec2( 1.0,  1.0),   // top right corner?
    vec2(-1.0,  1.0),   // top left
);

var<private> tex_coords: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),

    vec2(1.0, 1.0),
    vec2(1.0, 0.0),
    vec2(0.0, 0.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(positions[in_vertex_index].x, positions[in_vertex_index].y, 0.0, 1.0);
    out.tex_coords = tex_coords[in_vertex_index];
    return out;
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
