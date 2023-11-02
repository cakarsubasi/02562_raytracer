use super::{Bindable, BufferOwner, WgslBindDescriptor};
use crate::camera::Camera;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    camera_pos: [f32; 3],
    camera_constant: f32,
    camera_look_at: [f32; 3],
    aspect_ratio: f32,
    camera_up: [f32; 3],
    selection1: u32,
    selection2: u32,
    subdivision_level: u32,
    _padding0: [u32; 2],
}

const MAX_SUBDIVISION: u32 = 7; 

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

        Self { buffer, uniforms }
    }

    pub fn update(&mut self, uniform: Uniform) {
        self.uniforms = uniform;
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
            selection1: 0,
            selection2: 0,
            subdivision_level: 1,
            _padding0: [0, 0],
        }
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        // self.view_proj = camera.build_view_projection_matrix().into();
        self.camera_pos = camera.eye.into();
        self.camera_look_at = camera.target.into();
        self.camera_up = camera.up.into();
        self.camera_constant = camera.constant;
        self.aspect_ratio = camera.aspect;
    }

    pub fn update_sphere_selection(&mut self, selection: u32) {
        self.selection1 = selection;
    }

    pub fn update_other_selection(&mut self, selection: u32) {
        self.selection2 = selection;
    }

    pub fn update_subdivision_level(&mut self, level: u32) {
        if level <= MAX_SUBDIVISION {
            self.subdivision_level = level;
        } else {
            self.subdivision_level = MAX_SUBDIVISION;
            eprintln!("Attempted raise subdivision level above maximum");
        }
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

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: self.buffer.as_entire_binding(),
        }]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        let struct_def = Some(
"struct Uniform {
    camera_pos: vec3f,
    camera_constant: f32,
    camera_look_at: vec3f,
    aspect_ratio: f32,
    camera_up: vec3f,
    selection1: u32,
    selection2: u32,
};");

        let bind_type = "uniform";
        let var_name = "uniforms";
        let var_type = "Uniform";

        vec![
            WgslBindDescriptor {
                struct_def,
                bind_type: Some(bind_type),
                var_name,
                var_type,
                extra_code: None,
            }
        ]
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
