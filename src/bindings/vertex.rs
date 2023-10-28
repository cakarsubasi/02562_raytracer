use crate::data_structures::vector::{Vec3f32, vec3f32};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: Vec3f32,
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: vec3f32(-1.0, -1.0, 0.0),
    }, // A
    Vertex {
        position: vec3f32(-1.0, 1.0, 0.0),
    }, // B
    Vertex {
        position: vec3f32(1.0, 1.0, 0.0),
    }, // C
    Vertex {
        position: vec3f32(1.0, -1.0, 0.0),
    }, // D
];

pub const INDICES: &[u16] = &[2, 1, 0, 0, 3, 2];