// Text rendering shader
// Uses a glyph atlas texture for text rendering
// Supports bold, italic, underline, and blink attributes

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) attributes: vec4<f32>,
}

@group(0) @binding(0) var glyph_atlas: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) attributes: vec4<f32>
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = uv;
    output.color = color;
    output.attributes = attributes;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let glyph_alpha = textureSample(glyph_atlas, atlas_sampler, input.uv).a;
    
    // Skip if no glyph coverage (for underline/background quads with UV 0,0)
    // but allow drawing for non-glyph elements (underline) when UV is 0
    let is_glyph = input.uv.x != 0.0 || input.uv.y != 0.0;
    
    var final_alpha = glyph_alpha;
    
    // For non-glyph elements (like underline), use full alpha
    if (!is_glyph) {
        final_alpha = 1.0;
    }
    
    // Multiply glyph alpha by vertex color alpha for proper blending
    let final_color = vec4<f32>(
        input.color.rgb,
        input.color.a * final_alpha
    );
    
    return final_color;
}
