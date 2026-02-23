//! Text rendering using fontdue and wgpu.
//!
//! This module provides GPU-accelerated text rendering for the terminal emulator.
//! It uses fontdue for font rasterization and wgpu for GPU rendering.

use fontdue::{Font, FontSettings};
use std::collections::HashMap;
use std::sync::LazyLock;
use thiserror::Error;
use wgpu::{Device, Queue, TextureFormat};

use crate::terminal::parser::Color;

/// Maximum glyph size in the atlas (width or height)
const MAX_GLYPH_SIZE: u32 = 64;

/// Number of glyphs per row in the atlas
const ATLAS_COLUMNS: u32 = 16;

/// Number of rows in the atlas
const ATLAS_ROWS: u32 = 16;

/// Total atlas size
const ATLAS_SIZE: u32 = MAX_GLYPH_SIZE * ATLAS_COLUMNS;

/// Static ANSI color palette (256 colors, each with RGBA f32)
/// Using LazyLock to avoid stack allocation during runtime
static ANSI_PALETTE: LazyLock<[[f32; 4]; 256]> = LazyLock::new(|| {
    let mut palette = [[0.0f32; 4]; 256];

    // Basic 16 colors
    let basic: [[f32; 4]; 16] = [
        [0.0, 0.0, 0.0, 1.0],       // 0: Black
        [0.8, 0.0, 0.0, 1.0],       // 1: Red
        [0.0, 0.8, 0.0, 1.0],       // 2: Green
        [0.8, 0.8, 0.0, 1.0],       // 3: Yellow
        [0.0, 0.0, 0.8, 1.0],       // 4: Blue
        [0.8, 0.0, 0.8, 1.0],       // 5: Magenta
        [0.0, 0.8, 0.8, 1.0],       // 6: Cyan
        [0.8, 0.8, 0.8, 1.0],       // 7: White
        [0.5, 0.5, 0.5, 1.0],       // 8: Bright Black
        [1.0, 0.0, 0.0, 1.0],       // 9: Bright Red
        [0.0, 1.0, 0.0, 1.0],       // 10: Bright Green
        [1.0, 1.0, 0.0, 1.0],       // 11: Bright Yellow
        [0.0, 0.0, 1.0, 1.0],       // 12: Bright Blue
        [1.0, 0.0, 1.0, 1.0],       // 13: Bright Magenta
        [0.0, 1.0, 1.0, 1.0],       // 14: Bright Cyan
        [1.0, 1.0, 1.0, 1.0],       // 15: Bright White
    ];

    for (i, &color) in basic.iter().enumerate() {
        palette[i] = color;
    }

    // 216 color cube (16-231)
    for i in 0..216 {
        let r = (i / 36) % 6;
        let g = (i / 6) % 6;
        let b = i % 6;
        palette[16 + i] = [
            if r > 0 { (r * 40 + 55) as f32 / 255.0 } else { 0.0 },
            if g > 0 { (g * 40 + 55) as f32 / 255.0 } else { 0.0 },
            if b > 0 { (b * 40 + 55) as f32 / 255.0 } else { 0.0 },
            1.0,
        ];
    }

    // Grayscale (232-255)
    for i in 0..24 {
        let gray = (i * 10 + 8) as f32 / 255.0;
        palette[232 + i] = [gray, gray, gray, 1.0];
    }

    palette
});

#[derive(Error, Debug)]
pub enum TextError {
    #[error("Failed to load font: {0}")]
    FontLoad(String),
    
    #[error("Failed to create texture: {0}")]
    TextureCreation(String),
    
    #[error("Glyph not in atlas: {0}")]
    GlyphNotInAtlas(char),
    
    #[error("Atlas is full")]
    AtlasFull,
}

/// A glyph's position in the atlas texture.
#[derive(Debug, Clone, Copy)]
pub struct AtlasGlyph {
    /// UV coordinates in the atlas texture (top-left corner)
    pub uv_min: (f32, f32),
    /// UV coordinates in the atlas texture (bottom-right corner)
    pub uv_max: (f32, f32),
    /// Glyph width in pixels
    pub width: u32,
    /// Glyph height in pixels
    pub height: u32,
    /// Horizontal advance
    pub advance_width: f32,
    /// Left side bearing
    pub left_side_bearing: f32,
    /// Ascent (bearing_y)
    pub ascent: f32,
}

/// Glyph atlas for caching rendered glyphs in a GPU texture.
pub struct GlyphAtlas {
    /// The fontdue font
    font: Font,
    /// Font size in pixels
    font_size: f32,
    /// The atlas texture
    texture: Option<wgpu::Texture>,
    /// Texture view for sampling
    texture_view: Option<wgpu::TextureView>,
    /// Sampler for the texture
    sampler: Option<wgpu::Sampler>,
    /// Map from character to atlas position
    glyph_cache: HashMap<char, AtlasGlyph>,
    /// Next position in the atlas (column, row)
    next_pos: (u32, u32),
    /// Temporary pixel buffer for atlas updates
    atlas_buffer: Vec<u8>,
    /// Whether the atlas needs to be uploaded to GPU
    needs_upload: bool,
}

impl GlyphAtlas {
    /// Create a new glyph atlas with embedded monospace font.
    pub fn new(_device: &Device, font_size: f32) -> Result<Self, TextError> {
        // Load embedded monospace font (DejaVu Sans Mono is a good default)
        let font_data = include_bytes!("../fonts/DejaVuSansMono.ttf");
        let font = Font::from_bytes(
            font_data.as_slice(),
            FontSettings {
                collection_index: 0,
                scale: font_size,
                load_substitutions: true,
            },
        )
        .map_err(|e| TextError::FontLoad(e.to_string()))?;

        // Initialize atlas buffer (RGBA)
        let atlas_buffer = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE) as usize * 4];

        Ok(Self {
            font,
            font_size,
            texture: None,
            texture_view: None,
            sampler: None,
            glyph_cache: HashMap::new(),
            next_pos: (0, 0),
            atlas_buffer,
            needs_upload: false,
        })
    }

    /// Initialize GPU resources
    pub fn init_gpu(&mut self, device: &Device) {
        // Create the atlas texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas Texture"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.texture = Some(texture);
        self.texture_view = Some(texture_view);
        self.sampler = Some(sampler);
    }

    /// Cache a glyph in the atlas.
    pub fn cache_glyph(&mut self, c: char) -> Result<AtlasGlyph, TextError> {
        // Check if already cached
        if let Some(&glyph) = self.glyph_cache.get(&c) {
            return Ok(glyph);
        }

        // Rasterize the glyph
        let (metrics, bitmap) = self.font.rasterize(c, self.font_size);

        // Check if atlas is full
        if self.next_pos.1 >= ATLAS_ROWS {
            return Err(TextError::AtlasFull);
        }

        let (col, row) = self.next_pos;
        let x_offset = col * MAX_GLYPH_SIZE;
        let y_offset = row * MAX_GLYPH_SIZE;

        // Copy glyph bitmap into atlas buffer (convert to RGBA)
        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let src_idx = y * metrics.width + x;
                let alpha = bitmap[src_idx];
                
                let dst_x = x_offset as usize + x;
                let dst_y = y_offset as usize + y;
                let dst_idx = (dst_y * ATLAS_SIZE as usize + dst_x) * 4;

                // Write RGBA (white with alpha from glyph)
                self.atlas_buffer[dst_idx] = 255;     // R
                self.atlas_buffer[dst_idx + 1] = 255; // G
                self.atlas_buffer[dst_idx + 2] = 255; // B
                self.atlas_buffer[dst_idx + 3] = alpha; // A
            }
        }

        // Calculate UV coordinates (normalized 0-1)
        let uv_min = (
            x_offset as f32 / ATLAS_SIZE as f32,
            y_offset as f32 / ATLAS_SIZE as f32,
        );
        let uv_max = (
            (x_offset + metrics.width as u32) as f32 / ATLAS_SIZE as f32,
            (y_offset + metrics.height as u32) as f32 / ATLAS_SIZE as f32,
        );

        let atlas_glyph = AtlasGlyph {
            uv_min,
            uv_max,
            width: metrics.width as u32,
            height: metrics.height as u32,
            advance_width: metrics.advance_width,
            left_side_bearing: metrics.bounds.xmin,
            // Ascent is from baseline to top of glyph
            ascent: metrics.bounds.height + metrics.bounds.ymin,
        };

        self.glyph_cache.insert(c, atlas_glyph);
        self.needs_upload = true;

        // Advance position in atlas
        self.next_pos.0 += 1;
        if self.next_pos.0 >= ATLAS_COLUMNS {
            self.next_pos.0 = 0;
            self.next_pos.1 += 1;
        }

        Ok(atlas_glyph)
    }

    /// Upload the atlas texture to GPU.
    pub fn upload(&mut self, queue: &Queue) {
        if !self.needs_upload {
            return;
        }

        if let Some(ref texture) = self.texture {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.atlas_buffer,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(ATLAS_SIZE * 4),
                    rows_per_image: Some(ATLAS_SIZE),
                },
                wgpu::Extent3d {
                    width: ATLAS_SIZE,
                    height: ATLAS_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.needs_upload = false;
    }

    /// Get the cached glyph info.
    pub fn get_glyph(&self, c: char) -> Option<&AtlasGlyph> {
        self.glyph_cache.get(&c)
    }

    /// Get the texture view for binding.
    pub fn texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }

    /// Get the sampler for binding.
    pub fn sampler(&self) -> Option<&wgpu::Sampler> {
        self.sampler.as_ref()
    }

    /// Get the font size.
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Cache common ASCII characters.
    pub fn cache_common_glyphs(&mut self) -> Result<(), TextError> {
        // Cache ASCII printable characters
        for c in ' '..='~' {
            self.cache_glyph(c)?;
        }
        Ok(())
    }
}

/// Vertex data for text rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    /// Position in pixels (x, y)
    pub position: [f32; 2],
    /// UV coordinates
    pub uv: [f32; 2],
    /// Color (RGBA)
    pub color: [f32; 4],
    /// Text attributes packed as flags:
    /// - x: bold (1.0 or 0.0)
    /// - y: italic (1.0 or 0.0)
    /// - z: underline (1.0 or 0.0)
    /// - w: blink (1.0 or 0.0)
    pub attributes: [f32; 4],
}

impl TextVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = [
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 2]>() as u64,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 4]>() as u64,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x4,
        },
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<[f32; 8]>() as u64,
            shader_location: 3,
            format: wgpu::VertexFormat::Float32x4,
        },
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Text renderer that uses the glyph atlas and wgpu.
pub struct TextRenderer {
    /// Glyph atlas
    atlas: GlyphAtlas,
    /// Render pipeline
    pipeline: Option<wgpu::RenderPipeline>,
    /// Bind group layout
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    /// Vertex buffer
    vertex_buffer: Option<wgpu::Buffer>,
    /// Staging vertices
    vertices: Vec<TextVertex>,
    /// Screen dimensions
    screen_size: (u32, u32),
}

impl TextRenderer {
    /// Create a new text renderer.
    pub fn new(device: &Device, font_size: f32, screen_size: (u32, u32)) -> Result<Self, TextError> {
        let mut atlas = GlyphAtlas::new(device, font_size)?;
        atlas.init_gpu(device);
        
        // Cache common glyphs
        atlas.cache_common_glyphs()?;

        Ok(Self {
            atlas,
            pipeline: None,
            bind_group_layout: None,
            vertex_buffer: None,
            vertices: Vec::new(),
            screen_size,
        })
    }

    /// Initialize the render pipeline.
    pub fn init_pipeline(&mut self, device: &Device, format: TextureFormat) {
        // Create bind group layout for texture sampler
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Text Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/text.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for 2D
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create vertex buffer with some initial capacity
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Vertex Buffer"),
            size: 1024 * 1024, // 1MB initial capacity
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.pipeline = Some(pipeline);
        self.bind_group_layout = Some(bind_group_layout);
        self.vertex_buffer = Some(vertex_buffer);
    }

    /// Create bind group for the current frame.
    pub fn create_bind_group(&self, device: &Device) -> Option<wgpu::BindGroup> {
        let layout = self.bind_group_layout.as_ref()?;
        let texture_view = self.atlas.texture_view()?;
        let sampler = self.atlas.sampler()?;

        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        }))
    }

    /// Resize the screen.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_size = (width, height);
    }

    /// Convert Color to RGBA f32 array.
    fn color_to_rgba(color: Color, default_fg: [f32; 4], _default_bg: [f32; 4]) -> [f32; 4] {
        match color {
            Color::Default => default_fg,
            Color::Indexed(idx) => {
                // Use static ANSI color palette to avoid stack allocation
                let palette = &*ANSI_PALETTE;
                let idx = idx as usize;
                if idx < palette.len() {
                    palette[idx]
                } else {
                    default_fg
                }
            }
            Color::Rgb(r, g, b) => [
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                1.0,
            ],
        }
    }

    /// Queue a character for rendering.
    pub fn queue_char(
        &mut self,
        c: char,
        x: f32,
        y: f32,
        fg_color: Color,
        bg_color: Color,
        bold: bool,
        italic: bool,
        underline: bool,
        blink: bool,
    ) -> Result<(), TextError> {
        // Cache glyph if not already cached
        if self.atlas.get_glyph(c).is_none() {
            self.atlas.cache_glyph(c)?;
        }

        let glyph = self.atlas.get_glyph(c).ok_or(TextError::GlyphNotInAtlas(c))?;
        
        let (screen_w, screen_h) = self.screen_size;
        let screen_w = screen_w as f32;
        let screen_h = screen_h as f32;

        // Default terminal colors
        let default_fg = [0.9, 0.9, 0.9, 1.0];
        let default_bg = [0.05, 0.05, 0.05, 1.0];

        let fg = Self::color_to_rgba(fg_color, default_fg, default_bg);
        let _bg = Self::color_to_rgba(bg_color, default_bg, default_bg);

        // Calculate glyph position
        let glyph_x = x + glyph.left_side_bearing;
        let glyph_y = y + (self.atlas.font_size() - glyph.ascent);

        // Convert to normalized device coordinates (-1 to 1)
        let ndc_x = glyph_x / screen_w * 2.0 - 1.0;
        let ndc_y = 1.0 - glyph_y / screen_h * 2.0;
        let ndc_w = glyph.width as f32 / screen_w * 2.0;
        let ndc_h = glyph.height as f32 / screen_h * 2.0;

        // UV coordinates
        let (u_min, v_min) = glyph.uv_min;
        let (u_max, v_max) = glyph.uv_max;

        // Create two triangles (quad) for the glyph background
        let _bg_ndc_x = x / screen_w * 2.0 - 1.0;
        let _bg_ndc_y = 1.0 - y / screen_h * 2.0;
        let _cell_w = glyph.advance_width / screen_w * 2.0;
        let _cell_h = self.atlas.font_size() / screen_h * 2.0;

        // Pack attributes into a vec4 for the shader
        let attr_flags = [
            if bold { 1.0 } else { 0.0 },
            if italic { 1.0 } else { 0.0 },
            if underline { 1.0 } else { 0.0 },
            if blink { 1.0 } else { 0.0 },
        ];

        // Apply italic shear transformation to x coordinate based on y position
        // This creates a slanted appearance for italic text
        let italic_shear = if italic { 0.2 } else { 0.0 };

        // Foreground quad vertices (two triangles)
        let vertices = [
            // Triangle 1
            TextVertex {
                position: [ndc_x + italic_shear * ndc_h, ndc_y],
                uv: [u_min, v_min],
                color: fg,
                attributes: attr_flags,
            },
            TextVertex {
                position: [ndc_x + ndc_w + italic_shear * ndc_h, ndc_y],
                uv: [u_max, v_min],
                color: fg,
                attributes: attr_flags,
            },
            TextVertex {
                position: [ndc_x, ndc_y - ndc_h],
                uv: [u_min, v_max],
                color: fg,
                attributes: attr_flags,
            },
            // Triangle 2
            TextVertex {
                position: [ndc_x + ndc_w + italic_shear * ndc_h, ndc_y],
                uv: [u_max, v_min],
                color: fg,
                attributes: attr_flags,
            },
            TextVertex {
                position: [ndc_x + ndc_w, ndc_y - ndc_h],
                uv: [u_max, v_max],
                color: fg,
                attributes: attr_flags,
            },
            TextVertex {
                position: [ndc_x, ndc_y - ndc_h],
                uv: [u_min, v_max],
                color: fg,
                attributes: attr_flags,
            },
        ];

        self.vertices.extend_from_slice(&vertices);

        // For bold, render the glyph again with a slight horizontal offset for a bolder appearance
        if bold {
            let bold_offset = 0.5 / screen_w * 2.0; // Small offset in NDC
            let bold_vertices = [
                // Triangle 1
                TextVertex {
                    position: [ndc_x + italic_shear * ndc_h + bold_offset, ndc_y],
                    uv: [u_min, v_min],
                    color: fg,
                    attributes: attr_flags,
                },
                TextVertex {
                    position: [ndc_x + ndc_w + italic_shear * ndc_h + bold_offset, ndc_y],
                    uv: [u_max, v_min],
                    color: fg,
                    attributes: attr_flags,
                },
                TextVertex {
                    position: [ndc_x + bold_offset, ndc_y - ndc_h],
                    uv: [u_min, v_max],
                    color: fg,
                    attributes: attr_flags,
                },
                // Triangle 2
                TextVertex {
                    position: [ndc_x + ndc_w + italic_shear * ndc_h + bold_offset, ndc_y],
                    uv: [u_max, v_min],
                    color: fg,
                    attributes: attr_flags,
                },
                TextVertex {
                    position: [ndc_x + ndc_w + bold_offset, ndc_y - ndc_h],
                    uv: [u_max, v_max],
                    color: fg,
                    attributes: attr_flags,
                },
                TextVertex {
                    position: [ndc_x + bold_offset, ndc_y - ndc_h],
                    uv: [u_min, v_max],
                    color: fg,
                    attributes: attr_flags,
                },
            ];
            self.vertices.extend_from_slice(&bold_vertices);
        }

        // For underline, render a horizontal line at the baseline
        if underline {
            let underline_y = ndc_y - ndc_h + (2.0 / screen_h * 2.0); // 2 pixels below baseline
            let underline_h = 1.0 / screen_h * 2.0; // 1 pixel height
            let underline_color = fg;

            let underline_vertices = [
                // Single quad for underline
                TextVertex {
                    position: [ndc_x, underline_y],
                    uv: [0.0, 0.0],
                    color: underline_color,
                    attributes: [0.0, 0.0, 0.0, 0.0], // No attributes for underline
                },
                TextVertex {
                    position: [ndc_x + ndc_w, underline_y],
                    uv: [0.0, 0.0],
                    color: underline_color,
                    attributes: [0.0, 0.0, 0.0, 0.0],
                },
                TextVertex {
                    position: [ndc_x, underline_y - underline_h],
                    uv: [0.0, 0.0],
                    color: underline_color,
                    attributes: [0.0, 0.0, 0.0, 0.0],
                },
                TextVertex {
                    position: [ndc_x + ndc_w, underline_y],
                    uv: [0.0, 0.0],
                    color: underline_color,
                    attributes: [0.0, 0.0, 0.0, 0.0],
                },
                TextVertex {
                    position: [ndc_x + ndc_w, underline_y - underline_h],
                    uv: [0.0, 0.0],
                    color: underline_color,
                    attributes: [0.0, 0.0, 0.0, 0.0],
                },
                TextVertex {
                    position: [ndc_x, underline_y - underline_h],
                    uv: [0.0, 0.0],
                    color: underline_color,
                    attributes: [0.0, 0.0, 0.0, 0.0],
                },
            ];
            self.vertices.extend_from_slice(&underline_vertices);
        }

        Ok(())
    }

    /// Clear queued vertices.
    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    /// Upload vertex data and prepare for rendering.
    pub fn prepare(&mut self, device: &Device, queue: &Queue) {
        // Upload atlas if needed
        self.atlas.upload(queue);

        // Upload vertex data
        if !self.vertices.is_empty() {
            let vertex_data: &[u8] = bytemuck::cast_slice(&self.vertices);
            
            // Re-create buffer if needed
            let needed_size = vertex_data.len() as u64;
            if let Some(ref buffer) = self.vertex_buffer {
                if buffer.size() < needed_size {
                    self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Text Vertex Buffer"),
                        size: needed_size * 2, // Double for growth
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }));
                }
            }

            if let Some(ref buffer) = self.vertex_buffer {
                queue.write_buffer(buffer, 0, vertex_data);
            }
        }
    }

    /// Render the queued text.
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, bind_group: &'a wgpu::BindGroup) {
        if self.vertices.is_empty() {
            return;
        }

        let Some(ref pipeline) = self.pipeline else { return };
        let Some(ref vertex_buffer) = self.vertex_buffer else { return };

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..self.vertices.len() as u32, 0..1);
    }

    /// Get the font size.
    pub fn font_size(&self) -> f32 {
        self.atlas.font_size()
    }

    /// Get the number of queued vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_creation() {
        // This test would need a wgpu device which is hard to create in unit tests
        // Just test that the error types work
        let err = TextError::FontLoad("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}
