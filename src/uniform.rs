use crate::camera::Camera;

use wgpu::util::DeviceExt;

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
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    // view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    //_padding0: f32,
    camera_constant: f32,
    camera_look_at: [f32; 3],
    //_padding1: f32,
    aspect_ratio: f32,
    camera_up: [f32; 3],
    //_padding2: f32,
    //_padding3: f32,
    _padding4: f32,
}

pub trait BindGroup {
    fn create_bind_group(
        self,
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup);
}

impl Uniform {
    pub fn new() -> Self {
        Self {
            // view_proj: cgmath::Matrix4::identity().into(),
            camera_pos: [0.0, 0.0, 0.0],
            camera_look_at: [0.0, 0.0, 0.0],
            camera_up: [0.0, 0.0, 0.0],
            camera_constant: 1.0,
            aspect_ratio: 1.0,
            //_padding0: 0.0,
            //_padding1: 0.0,
            //_padding2: 0.0,
            //_padding3: 0.0,
            _padding4: 0.0,
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        // self.view_proj = camera.build_view_projection_matrix().into();
        self.camera_pos = camera.eye.into();
        self.camera_look_at = camera.target.into();
        self.camera_up = camera.up.into();
        self.camera_constant = camera.constant;
        self.aspect_ratio = camera.aspect;
    }
}

impl BindGroup for Uniform {
    fn create_bind_group(
        self,
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });
        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });
        (buffer, layout, bindgroup)
    }
}

#[allow(dead_code)]

//fn compute_jitters(jitter: f32, pixel_size: f32, subdivs: u32) -> Vec<(f32, f32)> {
//    assert!(subdivs <= 10);
//    let mut jitter_vectors = vec![];
//    use rand::prelude::*;
//    let mut rng = rand::thread_rng();
//    if subdivs == 1 {
//        jitter_vectors.push((0.0, 0.0));
//        return jitter_vectors;
//    } else {
//        for i in 0..subdivs {
//            for j in 0..subdivs {
//                let u = rng.gen();
//                jitter_vectors.push((u, u));
//            }
//        }
//    }
//
//    todo!();
//    jitter_vectors
//}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
    }, // A
    Vertex {
        position: [-1.0, 1.0, 0.0],
    }, // B
    Vertex {
        position: [1.0, 1.0, 0.0],
    }, // C
    Vertex {
        position: [1.0, -1.0, 0.0],
    }, // D
];

pub const INDICES: &[u16] = &[2, 1, 0, 0, 3, 2];
