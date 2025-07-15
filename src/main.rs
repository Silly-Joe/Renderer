mod buffer_set;
mod camera;
mod mesh;
mod render_context;
mod vertex;

use camera::Camera;
use glam::Quat;
use glam::Vec3;
use render_context::RenderContext;

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
    meshes: Vec<mesh::Mesh>,
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

    fn add_mesh(&mut self, vertices: Vec<vertex::Vertex>, indices: Vec<u16>) {
        let mesh = mesh::Mesh::new(vertices, indices);
        self.meshes.push(mesh);
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

            let render_context = RenderContext::new(window);
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
                for mesh in &self.meshes {
                    self.render_context
                        .as_mut()
                        .expect("Render Context not initialized")
                        .render(&self.camera, mesh);
                }
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
    app.add_mesh(
        vec![
            vertex::Vertex {
                position: [-0.5, -0.5, 1.0],
            },
            vertex::Vertex {
                position: [0.5, -0.5, 1.0],
            },
            vertex::Vertex {
                position: [0.0, 0.5, 1.0],
            },
        ],
        vec![0, 1, 2],
    );

    app.add_mesh(
        vec![
            vertex::Vertex {
                position: [1.5, -0.5, 1.0],
            },
            vertex::Vertex {
                position: [2.5, -0.5, 1.0],
            },
            vertex::Vertex {
                position: [2.0, 0.5, 1.0],
            },
        ],
        vec![0, 1, 2],
    );

    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    pollster::block_on(run());
}
