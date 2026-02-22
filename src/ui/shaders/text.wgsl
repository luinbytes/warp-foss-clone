// Text rendering shader
// Uses a glyph atlas texture for text rendering

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0) var glyph_atlas: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = uv;
    output.color = color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let glyph_alpha = textureSample(glyph_atlas, atlas_sampler, input.uv).a;
    
    // Multiply glyph alpha by vertex color alpha for proper blending
    let final_color = vec4<f32>(
        input.color.rgb,
        input.color.a * glyph_alpha
    );
    
    return final_color;
}
