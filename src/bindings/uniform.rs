use crate::camera::Camera;
use super::{Bindable, BufferOwner, WgslBindDescriptor};

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    camera_pos: [f32; 3],
    camera_constant: f32,
    camera_look_at: [f32; 3],
    aspect_ratio: f32,
    camera_up: [f32; 3],
    _padding4: f32,
}

pub struct UniformGpu {
    pub uniforms: Uniform,
    pub buffer: wgpu::Buffer,
}

impl UniformGpu {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniforms = Uniform::new();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        Self {
            buffer,
            uniforms,
        }
    }
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

impl BufferOwner for UniformGpu {
    fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }
}

impl Bindable for UniformGpu {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]
    }

    fn get_bind_group_entries(&self, device: &wgpu::Device) -> Vec<wgpu::BindGroupEntry> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: self.buffer.as_entire_binding(),
        }]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        todo!()
    }
}

impl UniformGpu {
    fn create_bind_group(
        self,
        device: &wgpu::Device,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: self.get_layout_entries().as_ref(),
            label: Some("uniform_bind_group_layout"),
        });
        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: self.get_bind_group_entries(device).as_ref(),
            label: Some("uniform_bind_group"),
        });
        (self.buffer, layout, bindgroup)
    }
}

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
