use crate::Camera;
use crate::buffer_set::BufferSet;
use crate::vertex::Vertex;

use crate::mesh::Mesh;
use glam::Mat4;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use winit::window::Window;

pub struct RenderContext {
    surface: Arc<wgpu::Surface<'static>>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    buffer_sets: HashMap<Uuid, BufferSet>,
}

impl RenderContext {
    pub fn new(window: &'static Window) -> Self {
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

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Mat4>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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

        Self {
            surface,
            device,
            queue,
            surface_config: config,
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            buffer_sets: HashMap::new(),
        }
    }

    pub fn register_mesh(&mut self, mesh: &Mesh) {
        if self.buffer_sets.contains_key(&mesh.id()) {
            return; // Mesh already registered
        }

        let buffer_set = mesh.buffer_set(&self.device);
        self.buffer_sets.insert(mesh.id(), buffer_set);
    }

    pub fn render(&mut self, camera: &Camera, mesh: &Mesh) {
        if !self.buffer_sets.contains_key(&mesh.id()) {
            self.register_mesh(mesh);
        }

        if self.surface_config.width == 0 || self.surface_config.height == 0 {
            return;
        }

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Failed to acquire next surface texture");
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let aspect_ratio = self.surface_config.width as f32 / self.surface_config.height as f32;

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(
                &camera
                    .view_projection_matrix(aspect_ratio)
                    .to_cols_array_2d(),
            ),
        );

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

            // Set vertex/index buffers
            let buffer_set = self.buffer_sets.get(&mesh.id()).unwrap();
            render_pass.set_vertex_buffer(0, buffer_set.vertex_buffer().slice(..));
            render_pass.set_index_buffer(
                buffer_set.index_buffer().slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            render_pass.draw_indexed(0..mesh.index_count() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        frame.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;

        if width == 0 || height == 0 {
            return; // Ignore zero-sized windows
        }
        self.surface.configure(&self.device, &self.surface_config);
    }
}
