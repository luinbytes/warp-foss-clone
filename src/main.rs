//! Warp FOSS - A free terminal with AI integration
//!
//! Main entry point with event loop that ties together:
//! - winit window management
//! - wgpu rendering
//! - PTY session for shell I/O
//! - Terminal parsing and grid state

mod ai;
mod config;
mod plugin;
mod terminal;
mod ui;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use terminal::grid::TerminalGrid;
use terminal::parser::TerminalParser;
use terminal::pty::{PtyConfig, PtySession};
use ui::input::InputHandler;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// Configuration for the terminal application
struct AppConfig {
    /// Initial terminal columns
    cols: u16,
    /// Initial terminal rows  
    rows: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cols: 120,
            rows: 40,
        }
    }
}

/// Main application state
struct TerminalApp {
    /// The winit window
    window: Option<Arc<Window>>,
    /// GPU renderer - stored as raw parts to avoid lifetime issues
    renderer: Option<RendererHolder>,
    /// PTY session for shell communication
    pty: Option<Arc<Mutex<PtySession>>>,
    /// Terminal parser for escape sequences
    parser: TerminalParser,
    /// Terminal grid (screen buffer)
    grid: TerminalGrid,
    /// Input handler for keyboard events
    input_handler: InputHandler,
    /// Whether the app is running
    running: bool,
    /// Last frame time for FPS limiting
    last_frame: Instant,
    /// Target frame duration (60 FPS)
    frame_duration: Duration,
}

/// Type-erased renderer holder to work around lifetime issues
struct RendererHolder {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl RendererHolder {
    async fn new(window: Arc<Window>) -> Result<Self, ui::renderer::RendererError> {
        use ui::renderer::RendererError;
        
        // Create instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // Create surface - we need 'static lifetime, so we leak the Arc
        // This is safe because the window lives for the duration of the application
        let window_static: &'static Window = Box::leak(Box::new(window));
        
        let surface = instance
            .create_surface(window_static)
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
        
        let size = window_static.inner_size();
        
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
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // Use Fifo (vsync) for maximum cross-platform compatibility
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);
        
        Ok(Self {
            device,
            queue,
            surface,
            config,
        })
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    fn render(&mut self) -> Result<(), ui::renderer::RendererError> {
        use ui::renderer::RendererError;
        
        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| RendererError::TextureAcquisition(e.to_string()))?;
        
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
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
}

impl TerminalApp {
    fn new() -> Self {
        let config = AppConfig::default();
        
        Self {
            window: None,
            renderer: None,
            pty: None,
            parser: TerminalParser::with_size(config.cols as usize, config.rows as usize),
            grid: TerminalGrid::with_size(config.cols as usize, config.rows as usize),
            input_handler: InputHandler::new(),
            running: false,
            last_frame: Instant::now(),
            frame_duration: Duration::from_micros(16_667), // ~60 FPS
        }
    }
    
    /// Initialize the PTY session
    fn init_pty(&mut self, cols: u16, rows: u16) -> Result<()> {
        let config = PtyConfig {
            cols,
            rows,
            ..Default::default()
        };
        
        let pty = PtySession::spawn(config)?;
        self.pty = Some(Arc::new(Mutex::new(pty)));
        Ok(())
    }
    
    /// Read and process PTY output (non-blocking with batching)
    fn read_pty_output(&mut self) {
        // Batch read from PTY - accumulate multiple reads before processing
        let mut data = Vec::with_capacity(16384); // Start with 16KB capacity

        // Try to read multiple times to batch available data
        let mut has_data = false;
        for _ in 0..5 {
            // Attempt to read more data (limit to 5 attempts to avoid blocking)
            let read_result = {
                if let Some(ref pty) = self.pty {
                    if let Ok(mut session) = pty.lock() {
                        let mut buf = vec![0u8; 4096];
                        match session.read(&mut buf) {
                            Ok(0) => {
                                // EOF - PTY closed
                                tracing::info!("PTY closed");
                                self.running = false;
                                return;
                            }
                            Ok(n) => {
                                buf.truncate(n);
                                (true, Some(buf))
                            }
                            Err(e) => {
                                // Would block is expected when no data available
                                let err_str = e.to_string();
                                if !err_str.contains("Would block") && !err_str.contains("Resource temporarily unavailable") {
                                    tracing::debug!("PTY read error: {}", e);
                                }
                                // No more data available
                                break;
                            }
                        }
                    } else {
                        (false, None)
                    }
                } else {
                    (false, None)
                }
            };

            match read_result {
                (_, Some(buf)) if !buf.is_empty() => {
                    data.extend_from_slice(&buf);
                    has_data = true;
                }
                (_, None) => {
                    break; // No more data or error
                }
                _ => {
                    break;
                }
            }
        }

        // Process the data if we accumulated any
        if has_data && !data.is_empty() {
            self.process_terminal_output(&data);
        }
    }
    
    /// Process terminal output bytes through the parser to the grid.
    ///
    /// This is the main pipeline: PTY bytes → Parser (escape sequences) → Grid (screen buffer)
    ///
    /// Uses batch mode to optimize grid updates when processing large amounts of data.
    fn process_terminal_output(&mut self, data: &[u8]) {
        // Sync grid colors/attributes from parser state before processing
        self.grid.set_foreground(self.parser.state.fg_color);
        self.grid.set_background(self.parser.state.bg_color);
        self.grid.set_attributes(self.parser.state.attributes);

        // Use batch mode for grid updates to reduce overhead
        // This buffers all cell updates and applies them in a single pass
        self.grid.begin_batch();

        // Parse bytes and output directly to the grid
        // This handles escape sequences and writes characters to the grid
        self.parser.parse_bytes_with_output(data, &mut self.grid);

        // Flush batched updates and calculate dirty region
        self.grid.flush_batch();
    }
    
    /// Send input to the PTY
    fn send_pty_input(&mut self, data: &[u8]) {
        if !data.is_empty() {
            if let Some(ref pty) = self.pty {
                if let Ok(mut session) = pty.lock() {
                    if let Err(e) = session.write(data) {
                        tracing::error!("Failed to write to PTY: {}", e);
                    }
                }
            }
        }
    }
    
    /// Handle window resize
    fn handle_resize(&mut self, width: u32, height: u32) {
        // Resize the renderer
        if let Some(ref mut renderer) = self.renderer {
            renderer.resize(width, height);
        }
        
        // Calculate new terminal dimensions (assuming 10x20 pixel cells)
        let cell_width = 10u32;
        let cell_height = 20u32;
        let new_cols = (width / cell_width) as usize;
        let new_rows = (height / cell_height) as usize;
        
        // Resize the terminal grid
        if new_cols > 0 && new_rows > 0 {
            self.grid.resize(new_cols, new_rows);
            self.parser.resize(new_cols, new_rows);
            
            // Resize the PTY
            if let Some(ref pty) = self.pty {
                if let Ok(mut session) = pty.lock() {
                    if let Err(e) = session.resize(new_cols as u16, new_rows as u16) {
                        tracing::error!("Failed to resize PTY: {}", e);
                    }
                }
            }
        }
    }
    
    /// Render a frame
    fn render(&mut self) {
        if let Some(ref mut renderer) = self.renderer {
            if let Err(e) = renderer.render() {
                tracing::error!("Render error: {}", e);
            }
        }
    }
}

impl ApplicationHandler for TerminalApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_title("Warp FOSS")
                .with_inner_size(PhysicalSize::new(1200, 800))
        ) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                tracing::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        
        // Get initial size
        let size = window.inner_size();
        let cols = (size.width / 10) as u16;
        let rows = (size.height / 20) as u16;
        
        // Initialize renderer
        let renderer = match pollster::block_on(RendererHolder::new(Arc::clone(&window))) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to initialize renderer: {}", e);
                event_loop.exit();
                return;
            }
        };
        
        // Initialize PTY
        if let Err(e) = self.init_pty(cols.max(40), rows.max(10)) {
            tracing::error!("Failed to initialize PTY: {}", e);
            event_loop.exit();
            return;
        }
        
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.running = true;
        
        tracing::info!("Terminal application started");
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.running = false;
                event_loop.exit();
            }
            
            WindowEvent::Resized(physical_size) => {
                self.handle_resize(physical_size.width, physical_size.height);
            }
            
            WindowEvent::KeyboardInput { event, .. } => {
                let input = self.input_handler.handle_key_event(&event);
                let data = input.to_bytes();
                self.send_pty_input(&data);
            }
            
            WindowEvent::ModifiersChanged(modifiers) => {
                self.input_handler.modifiers_mut().update_from_state(modifiers.state());
            }
            
            WindowEvent::RedrawRequested => {
                // Read and process any pending PTY output
                self.read_pty_output();
                
                // Render
                self.render();
                
                // Request next frame
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            
            _ => {}
        }
    }
    
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Process PTY output periodically
        self.read_pty_output();
        
        // Limit frame rate
        let elapsed = self.last_frame.elapsed();
        if elapsed < self.frame_duration {
            let wait = self.frame_duration - elapsed;
            std::thread::sleep(wait.min(Duration::from_millis(1)));
        }
        self.last_frame = Instant::now();
        
        // Request redraw if running
        if self.running {
            if let Some(ref window) = self.window {
                window.request_redraw();
            }
        }
        
        // Set control flow
        event_loop.set_control_flow(ControlFlow::Wait);
    }
    
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        tracing::info!("Terminal application exiting");
    }
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    tracing::info!("Warp FOSS v0.1.0");
    tracing::info!("Starting terminal application...");
    
    // Create event loop
    let event_loop = EventLoop::new()?;
    
    // Create app
    let mut app = TerminalApp::new();
    
    // Run event loop
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
