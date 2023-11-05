use super::{Bindable, BufferOwner, WgslBindDescriptor};
use crate::camera::Camera;

use wgpu::util::DeviceExt;

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    /// World space camera position
    camera_pos: [f32; 3],
    /// Camera constant (affects FOV)
    camera_constant: f32,
    /// Camera focus point
    camera_look_at: [f32; 3],
    /// Aspect ratio of the window (for Horz+ correction)
    aspect_ratio: f32,
    /// Which way is "up"
    camera_up: [f32; 3],
    /// selection 1 for branching
    selection1: u32,
    /// selection 2 for branching
    selection2: u32,
    /// requested pixel subdivision level
    subdivision_level: u32,
    /// whether to use the texture
    use_texture: u32,
    /// Which iteration this frame is for ping pong rendering
    iteration: u32,
    /// uv scale for the texture
    uv_scale: [f32; 2],
    /// resolution of the canvas for ping pong rendering
    /// and random seeding
    canvas_resolution: [u32; 2],
}

pub const MAX_SUBDIVISION: u32 = 10;

pub struct UniformGpu {
    uniforms: Uniform,
    buffer: wgpu::Buffer,
    jitter_buffer: wgpu::Buffer,
    pub max_iterations: u32,
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
            max_iterations: 1,
        }
    }

    pub fn update(
        &mut self,
        camera: Option<&Camera>,
        selection1: Option<u32>,
        selection2: Option<u32>,
        _iteration: Option<u32>,
        canvas_resolution: Option<(u32, u32)>,
    ) {
        if let Some(camera) = camera {
            self.update_camera(camera);
        };
        if let Some(selection1) = selection1 {
            self.update_sphere_selection(selection1);
        };
        if let Some(selection2) = selection2 {
            self.update_other_selection(selection2);
        };
        if let Some(canvas_resolution) = canvas_resolution {
            self.update_resolution(canvas_resolution);
        };
    }

    // For ping pong rendering
    pub fn increase_iteration(&mut self) {
        self.uniforms.iteration += 1;
    }

    pub fn reset_iteration(&mut self) {
        self.uniforms.iteration = 0;
    }

    // For ping pong rendering
    pub fn get_iteration(&self) -> u32 {
        self.uniforms.iteration
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.uniforms.camera_pos = camera.eye.into();
        self.uniforms.camera_look_at = camera.target.into();
        self.uniforms.camera_up = camera.up.into();
        self.uniforms.camera_constant = camera.constant;
        self.uniforms.aspect_ratio = camera.aspect;
    }

    pub fn update_sphere_selection(&mut self, selection: u32) {
        self.uniforms.selection1 = selection;
    }

    pub fn update_other_selection(&mut self, selection: u32) {
        self.uniforms.selection2 = selection;
    }

    pub fn update_subdivision_level(&mut self, level: u32) {
        if level <= MAX_SUBDIVISION {
            self.uniforms.subdivision_level = level;
        } else {
            self.uniforms.subdivision_level = MAX_SUBDIVISION;
            eprintln!("Attempted raise subdivision level above maximum");
        }
    }

    pub fn update_use_texture(&mut self, use_texture: u32) {
        self.uniforms.use_texture = use_texture;
    }

    pub fn update_uv_scale(&mut self, uv_scale: (f32, f32)) {
        self.uniforms.uv_scale = uv_scale.into();
    }

    pub fn update_iteration(&mut self, iteration: u32) {
        self.uniforms.iteration = iteration;
    }

    pub fn update_resolution(&mut self, resolution: (u32, u32)) {
        self.uniforms.canvas_resolution = resolution.into()
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
            use_texture: 0,
            uv_scale: [1.0, 1.0],
            iteration: 0,
            canvas_resolution: [512, 512],
        }
    }
}

impl BufferOwner for UniformGpu {
    fn update_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
        let jitter_vec = compute_jitters(
            1.0 / self.uniforms.canvas_resolution[1] as f64,
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
    subdivision_level: u32,
    use_texture: u32,
    iteration: u32,
    uv_scale: vec2f,
    resolution: vec2f,
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
    assert!(subdivs <= MAX_SUBDIVISION && subdivs > 0 && pixel_size != 0.0);
    let mut jitter_vectors = vec![];
    use rand::prelude::*;
    use rand_pcg::Lcg64Xsh32;
    let mut rng = Lcg64Xsh32::new(0, 0);
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
