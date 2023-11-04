use wgpu::util::DeviceExt;

use super::vertex::Vertex;

/// This mesh only contains vertices and indices passed
/// directly to the Vertex shader
/// For our purposes, we only pass 4 vertices and 6 indexes
/// Which allows us to pass a canvas we can draw on in the fragment shader
pub struct MeshGpu {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl MeshGpu {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }
}