# 1. Use wgpu for GPU Rendering

Date: 2026-02-28

## Status

Accepted

## Context

We need to choose a GPU rendering technology for the terminal emulator. The main requirements are:

1. **Cross-platform**: Must work on Linux, macOS, and Windows
2. **Performance**: Terminal emulation requires fast text rendering
3. **Modern**: Should use modern GPU APIs for better performance
4. **Maintainable**: Should have good Rust support and active maintenance

Options considered:
- **wgpu**: Rust-native, WebGPU-based, cross-platform
- **OpenGL/Vulkan directly**: More control but more complexity
- **Skia**: Google's 2D graphics library, proven in Chrome/Firefox
- **Cairo**: Mature 2D graphics, but CPU-focused
- **glium**: Safe OpenGL wrapper, but less actively maintained

## Decision

We will use **wgpu** for GPU rendering.

wgpu is a Rust-native, cross-platform graphics API based on the WebGPU standard. It provides:
- Native performance on Vulkan, Metal, DirectX, and WebGPU
- Safe Rust API
- Active development and community
- Good documentation and examples
- Future-proof (WebGPU is the emerging standard)

## Consequences

**Positive:**
- Excellent cross-platform support
- Modern, safe Rust API
- Good performance characteristics
- Future-proof (WebGPU standard)
- Active community and development

**Negative:**
- Learning curve for WebGPU concepts
- Less mature than OpenGL/Vulkan
- May have more complexity than needed for simple text rendering
- Binary size overhead

**Neutral:**
- Requires understanding of GPU concepts (pipelines, shaders, buffers)
- WebGPU spec is still evolving

## Implementation Notes

- Text rendering uses a glyph atlas cached in a GPU texture
- Character rendering is done via instanced rendering for performance
- wgpu's `Surface` is used for window integration via winit
- Shader code is written in WGSL (WebGPU Shading Language)

## References

- [wgpu repository](https://github.com/gfx-rs/wgpu)
- [WebGPU specification](https://www.w3.org/TR/webgpu/)
- [Learn wgpu tutorial](https://sotrh.github.io/learn-wgpu/)
