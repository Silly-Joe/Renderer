pub struct BufferSet {
    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
}

impl BufferSet {
    pub fn new(index_buffer: wgpu::Buffer, vertex_buffer: wgpu::Buffer) -> Self {
        Self {
            index_buffer,
            vertex_buffer,
        }
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }
}
