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

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var colors: texture_1d<f32>;
@group(0) @binding(3) var color_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // const colors: array<vec4<f32>, 4> = array(
    //     vec4(0., 0.2, 0.7, 0.),
    //     vec4(0.3, 0., 0.5, 0.),
    //     vec4(0.4, 0.4, 0., 0.),
    //     vec4(0.2, 0.2, 0.2, 0.),
    // );
    let idx = textureSample(texture, texture_sampler, in.tex_coords)[0] * 16;
    
    return textureSample(colors, color_sampler, idx);
    //return textureSample(texture_diffuse, sampler_diffuse, in.tex_coords);
    // let idx = textureSample(texture_diffuse, sampler_diffuse, in.tex_coords);

    // return vec4(idx[0], idx[0], idx[0], 0);
    // return colors[u32(idx[0])];
    // return textureLoad(texture_diffuse, in.tex_coords, 0);
}
