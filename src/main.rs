mod vertex;
use vertex::Vertex;

use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    instance: Option<wgpu::Instance>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
}

impl App {
    fn initialize_window_and_surface(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let instance = wgpu::Instance::default();

        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Renderer"))
                .expect("Failed to create window"),
        );

        self.window = Some(window.clone());

        let window_size = window.inner_size();

        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("Failed to request adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to request device");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        surface.configure(&device, &config);

        self.surface = Some(surface);
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.instance = Some(instance);
    }

    fn initialize_pipeline(&mut self) {
        let pipeline_layout = self
            .device
            .as_ref()
            .expect("Device not initialized")
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let shader = self
            .device
            .as_ref()
            .expect("Device not initialized")
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let render_pipeline = self
            .device
            .as_ref()
            .expect("Device not initialized")
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.config.as_ref().unwrap().format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        self.render_pipeline = Some(render_pipeline);
    }

    fn is_visible(&self) -> bool {
        match self.config {
            Some(ref config) => config.width != 0 && config.height != 0,
            None => false,
        }
    }

    fn init_vetex_buffer_with_data(&mut self) {
        let vertices = [
            Vertex {
                position: [-0.5, -0.5],
            },
            Vertex {
                position: [0.5, -0.5],
            },
            Vertex {
                position: [0.0, 0.5],
            },
        ];

        let vertex_buffer = self
            .device
            .as_ref()
            .expect("Device not initialized")
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        self.vertex_buffer = Some(vertex_buffer);
    }

    fn render(&self) {
        let surface = self.surface.as_ref().expect("Surface not initialized");
        let device = self.device.as_ref().expect("Device not initialized");
        let queue = self.queue.as_ref().expect("Queue not initialized");

        let frame = surface
            .get_current_texture()
            .expect("Failed to get frame texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(
                self.render_pipeline
                    .as_ref()
                    .expect("Pipeline not initialized"),
            );

            render_pass.set_vertex_buffer(
                0,
                self.vertex_buffer
                    .as_ref()
                    .expect("Vertex buffer not initialized")
                    .slice(..),
            );

            render_pass.draw(0..3, 0..1); // 3 Vertices, 1 Instanz
        }

        queue.submit(std::iter::once(encoder.finish()));

        frame.present();
    }
}

impl winit::application::ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            self.initialize_window_and_surface(event_loop);
            self.initialize_pipeline();
            self.init_vetex_buffer_with_data();
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.window.is_none() {
            return;
        }
        match event {
            WindowEvent::RedrawRequested => {
                if !self.is_visible() {
                    return;
                }
                self.render();
            }
            WindowEvent::Resized(size) => {
                let config = self.config.as_mut().expect("Config not initialized");
                config.width = size.width;
                config.height = size.height;

                if size.width == 0 || size.height == 0 {
                    return; // Ignore zero-sized windows
                }

                let surface = self.surface.as_ref().expect("Surface not initialized");
                surface.configure(
                    self.device.as_ref().expect("Device not initialized"),
                    config,
                );
            }
            WindowEvent::CloseRequested => std::process::exit(0),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    pollster::block_on(run());
}
