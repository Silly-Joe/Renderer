use wgpu::util::DeviceExt;

use crate::vertex::Vertex;

use crate::buffer_set::BufferSet;

pub struct Mesh {
    id: uuid::Uuid,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            vertices,
            indices,
            id: uuid::Uuid::new_v4(),
        }
    }

    pub fn buffer_set(&self, device: &wgpu::Device) -> BufferSet {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        BufferSet::new(index_buffer, vertex_buffer)
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn id(&self) -> uuid::Uuid {
        self.id
    }
}
