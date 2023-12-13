use wgpu::util::DeviceExt;

use crate::{bindings::WgslSource, data_structures::{bvh::{self}, hlbvh::{self, GpuNode}}};

use super::{Bindable, WgslBindDescriptor, IntoGpu};

pub struct BvhGpu {
    pub bvh_buffer: wgpu::Buffer,
    pub bvh_triangles_buffer: wgpu::Buffer,
}

impl Bindable for BvhGpu {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
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
            }
        ]
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: self.bvh_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: self.bvh_triangles_buffer.as_entire_binding(),
            },
        ]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {

        let bvh_code = "res/shaders/bvh.wgsl";
        
        vec![
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "bvh_nodes",
                var_type: "array<BvhNode>",
                extra_code: Some(WgslSource::File(bvh_code)),
            },
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "bvh_triangles",
                var_type: "array<u32>",
                extra_code: None,
            }
        ]
    }
}

impl BvhGpu {
    pub fn new(device: &wgpu::Device, nodes: Vec<GpuNode>, triangles: &Vec<u32>) -> Self {
        let nodes_slice = nodes.as_slice();
        let bvh_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BVH nodes buffer"),
            contents: bytemuck::cast_slice(nodes_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        let triangles_slice = triangles.as_slice();
        let bvh_triangles_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BVH triangles buffer"),
            contents: bytemuck::cast_slice(triangles_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        BvhGpu {
            bvh_buffer,
            bvh_triangles_buffer,
        }
    }
}

impl IntoGpu for bvh::Bvh {
    type Output = BvhGpu;

    fn into_gpu(&self, device: &wgpu::Device) -> Self::Output {
        BvhGpu::new(device, self.flatten(), self.triangles())
    }
}

impl IntoGpu for hlbvh::Bvh {
    type Output = BvhGpu;

    fn into_gpu(&self, device: &wgpu::Device) -> Self::Output {
        BvhGpu::new(device, self.flatten(), &self.triangles())
    }
}