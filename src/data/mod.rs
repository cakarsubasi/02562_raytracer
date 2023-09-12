use cgmath::SquareMatrix;
use crate::camera::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    view_proj: [[f32; 4]; 4],
}

impl Uniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    } 
}

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0] }, // A
    Vertex { position: [-1.0, 1.0, 0.0] }, // B
    Vertex { position: [1.0, 1.0, 0.0] }, // C
    Vertex { position: [1.0, -1.0, 0.0] }, // D
];

pub const INDICES: &[u16] = &[
    2, 1, 0,
    0, 3, 2,
];