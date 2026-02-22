// Basic vertex shader
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Placeholder - we'll add actual vertex data later
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>(0.5, -0.5)
    );
    
    return vec4<f32>(pos[vertex_index], 0.0, 1.0);
}

// Basic fragment shader
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // Placeholder color - we'll render text/glyphs here later
    return vec4<f32>(0.2, 0.6, 0.8, 1.0);
}
