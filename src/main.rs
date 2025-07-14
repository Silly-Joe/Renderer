mod camera;
mod vertex;

use camera::Camera;
use glam::Quat;
use glam::Vec3;
use vertex::Vertex;

use std::sync::Arc;

use wgpu::util::DeviceExt;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

#[derive(Default)]
struct App {
    window: Option<&'static Window>,
    render_context: Option<RenderContext>,
    camera: Camera,
}

struct RenderContext {
    surface: Arc<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl RenderContext {
    fn new(window: &'static Window, camera: &Camera) -> Self {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = Arc::new(
            instance
                .create_surface(window)
                .expect("Failed to create surface"),
        );

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

        let camera_data = camera.projection_matrix().to_cols_array_2d();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&camera_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: config.format,
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

        let vertices = [
            Vertex {
                position: [-0.5, -0.5, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 1.0],
            },
            Vertex {
                position: [0.0, 0.5, 1.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    fn render(&self, camera: &Camera) {
        if self.config.width == 0 || self.config.height == 0 {
            return;
        }

        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to get frame texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            render_pass.draw(0..3, 0..1); // 3 Vertices, 1 Instanz
        }

        // Kamera setzen
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&camera.view_projection_matrix().to_cols_array_2d()),
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        frame.present();
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;

        if width == 0 || height == 0 {
            return; // Ignore zero-sized windows
        }
        self.surface.configure(&self.device, &self.config);
    }
}

impl App {
    fn keyboard_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Escape => std::process::exit(0),
            KeyCode::KeyW => {
                self.camera.translate(Vec3::new(0.0, 0.0, -0.1));
            }
            KeyCode::KeyS => {
                self.camera.translate(Vec3::new(0.0, 0.0, 0.1));
            }
            KeyCode::KeyA => {
                self.camera.translate(Vec3::new(-0.1, 0.0, 0.0));
            }
            KeyCode::KeyD => {
                self.camera.translate(Vec3::new(0.1, 0.0, 0.0));
            }
            KeyCode::KeyQ => {
                self.camera.rotate(Quat::from_rotation_y(0.1));
            }
            KeyCode::KeyE => {
                self.camera.rotate(Quat::from_rotation_y(-0.1));
            }
            _ => {}
        }
    }
}

impl winit::application::ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = Box::leak(Box::new(
                event_loop
                    .create_window(WindowAttributes::default().with_title("Renderer"))
                    .unwrap(),
            ));

            let render_context = RenderContext::new(window, &self.camera);
            self.window = Some(window);
            self.render_context = Some(render_context);
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
                self.render_context
                    .as_mut()
                    .expect("Render Context not initialized")
                    .render(&self.camera);
            }
            WindowEvent::Resized(size) => {
                self.render_context
                    .as_mut()
                    .expect("Render Context not initialized")
                    .resize(size.width, size.height);
            }
            WindowEvent::CloseRequested => std::process::exit(0),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.keyboard_input(code);
                }
            }
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
