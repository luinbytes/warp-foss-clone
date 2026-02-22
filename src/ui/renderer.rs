use thiserror::Error;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureViewDescriptor};
use winit::window::Window;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Failed to create wgpu surface: {0}")]
    SurfaceCreation(String),
    
    #[error("Failed to request adapter: {0}")]
    AdapterRequest(String),
    
    #[error("Failed to request device: {0}")]
    DeviceRequest(String),
    
    #[error("Failed to configure surface")]
    SurfaceConfiguration,
    
    #[error("Failed to get current texture: {0}")]
    TextureAcquisition(String),
    
    #[error("Render error: {0}")]
    Render(String),
}

/// GPU-accelerated renderer using wgpu
pub struct Renderer<'window> {
    device: Device,
    queue: Queue,
    surface: Surface<'window>,
    config: SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
}

impl<'window> Renderer<'window> {
    /// Create a new renderer instance
    pub async fn new(window: &'window Window) -> Result<Self, RendererError> {
        // Create instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // Create surface
        let surface = instance
            .create_surface(window)
            .map_err(|e| RendererError::SurfaceCreation(e.to_string()))?;
        
        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| RendererError::AdapterRequest("No suitable adapter found".to_string()))?;
        
        // Get surface capabilities
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let size = window.inner_size();
        
        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| RendererError::DeviceRequest(e.to_string()))?;
        
        // Configure surface
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);
        
        // Create a basic render pipeline (placeholder for now)
        let render_pipeline = Self::create_render_pipeline(&device, config.format);
        
        Ok(Self {
            device,
            queue,
            surface,
            config,
            render_pipeline,
        })
    }
    
    /// Create a basic render pipeline
    fn create_render_pipeline(
        device: &Device,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        // Create empty shader for now (we'll add actual shaders later)
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/basic.wgsl").into()),
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
        })
    }
    
    /// Resize the renderer surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    /// Render a frame
    pub fn render(&mut self) -> Result<(), RendererError> {
        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| RendererError::TextureAcquisition(e.to_string()))?;
        
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // Note: We'll draw actual geometry here later
            // For now, just clearing to dark background
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
    
    /// Get reference to device
    pub fn device(&self) -> &Device {
        &self.device
    }
    
    /// Get reference to queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
    
    /// Get current surface configuration
    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = RendererError::SurfaceCreation("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}
