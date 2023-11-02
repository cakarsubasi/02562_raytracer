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
    canvas_height: u32,
    buffer: wgpu::Buffer,
    jitter_buffer: wgpu::Buffer,
}

impl UniformGpu {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniforms = Uniform::new();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let jitter_array = vec![0.0 as f32; (2 * MAX_SUBDIVISION * MAX_SUBDIVISION) as usize];

        let jitter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Jitter buffer"),
            contents: bytemuck::cast_slice(jitter_array.as_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            jitter_buffer,
            uniforms,
            canvas_height: 512,
        }
    }

    pub fn update(
        &mut self,
        camera: Option<&Camera>,
        selection1: Option<u32>,
        selection2: Option<u32>,
        canvas_height: Option<u32>,
    ) {
        if let Some(camera) = camera {
            self.uniforms.update_camera(camera);
        };
        if let Some(selection1) = selection1 {
            self.uniforms.update_sphere_selection(selection1);
        };
        if let Some(selection2) = selection2 {
            self.uniforms.update_other_selection(selection2);
        };
        if let Some(canvas_height) = canvas_height {
            self.canvas_height = canvas_height;
        };
    }
}

impl Uniform {
    pub fn new() -> Self {
        Self {
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
        let jitter_vec = compute_jitters(
            1.0 / self.canvas_height as f64,
            self.uniforms.subdivision_level,
        );
        queue.write_buffer(
            &self.jitter_buffer,
            0,
            bytemuck::cast_slice(jitter_vec.as_slice()),
        )
    }
}

impl Bindable for UniformGpu {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: self.buffer.as_entire_binding(),
        }, wgpu::BindGroupEntry {
            binding: 1,
            resource: self.jitter_buffer.as_entire_binding(),
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
};",
        );

        //@group(0) @binding(2)
        //var<storage> jitter: array<vec2f>;

        vec![
            WgslBindDescriptor {
                struct_def,
                bind_type: Some("uniform"),
                var_name: "uniforms",
                var_type: "Uniform",
                extra_code: None,
            },
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "jitter",
                var_type: "array<vec2f>",
                extra_code: None,
            },
        ]
    }
}

fn compute_jitters(pixel_size: f64, subdivs: u32) -> Vec<[f32; 2]> {
    assert!(subdivs <= MAX_SUBDIVISION);
    let mut jitter_vectors = vec![];
    use rand::prelude::*;
    let mut rng = rand::thread_rng();
    let step = pixel_size / subdivs as f64;
    if subdivs == 1 {
        jitter_vectors.push([0.0, 0.0]);
        return jitter_vectors;
    } else {
        for i in 0..subdivs {
            for j in 0..subdivs {
                let u1 = rng.gen_range(0.0..1.0);
                let u2 = rng.gen_range(0.0..1.0);
                jitter_vectors.push([
                    ((u1 + j as f64) * step - pixel_size * 0.5) as f32,
                    ((u2 + i as f64) * step - pixel_size * 0.5) as f32,
                ]);
            }
        }
    }
    jitter_vectors
}
