use wgpu::util::DeviceExt;

use crate::{data_structures::bsp_tree::{BspTreeIntermediate, BspTree}, bindings::WgslBindDescriptor};

use super::{Bindable, IntoGpu, WgslSource, bvh::BvhGpu};

pub enum TraversalStructure {
    Bsp(BspTreeGpu),
    Bvh(BvhGpu),
    None,
}

impl Bindable for TraversalStructure {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        match self {
            TraversalStructure::Bsp(bsp_tree) => bsp_tree.get_layout_entries(),
            TraversalStructure::Bvh(bvh) => bvh.get_layout_entries(),
            TraversalStructure::None => vec![],
        }
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        match self {
            TraversalStructure::Bsp(bsp_tree) => bsp_tree.get_bind_group_entries(),
            TraversalStructure::Bvh(bvh) => bvh.get_bind_group_entries(),
            TraversalStructure::None => vec![],
        }
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        match self {
            TraversalStructure::Bsp(bsp_tree) => bsp_tree.get_bind_descriptor(),
            TraversalStructure::Bvh(bvh) => bvh.get_bind_descriptor(),
            TraversalStructure::None => vec![],
        }
    }
}

pub struct BspTreeGpu {
    // need to hold intermediates so they don't get dropped
    _intermediates: BspTreeIntermediate,
    pub bbox_buffer: wgpu::Buffer,
    pub ids_buffer: wgpu::Buffer,
    pub bsp_tree_buffer: wgpu::Buffer,
    pub bsp_planes_buffer: wgpu::Buffer,
}

impl Bindable for BspTreeGpu {
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
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
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
        vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.bbox_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.ids_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.bsp_tree_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.bsp_planes_buffer.as_entire_binding(),
                },
            ]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        let aabb_definition =       
"struct Aabb {
    min: vec3f,
    _padding: f32,
    max: vec3f,
    _padding2: f32,
};";

let max_depth = 
format!("
const MAX_LEVEL = {}u;
", self._intermediates.max_depth);

        let aabb_code = "res/shaders/aabb.wgsl";

        let bsp_tree_code = "res/shaders/bsp.wgsl";
        
        vec![
            WgslBindDescriptor {
                struct_def: Some(aabb_definition),
                bind_type: Some("uniform"),
                var_name: "aabb",
                var_type: "Aabb",
                extra_code: Some(WgslSource::File(aabb_code)),
            },
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "treeIds",
                var_type: "array<u32>",
                extra_code: None,
            },
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "bspTree",
                var_type: "array<vec4u>",
                extra_code: Some(WgslSource::Str(max_depth)),
            },
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "bspPlanes",
                var_type: "array<f32>",
                extra_code: Some(WgslSource::File(bsp_tree_code)),
            }
        ]
    }
}

impl BspTreeGpu {
    pub fn new(device: &wgpu::Device, bsp_tree_data: BspTreeIntermediate) -> Self {
        let bbox_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bounding Box Uniform"),
            contents: bytemuck::cast_slice(&[bsp_tree_data.bbox]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let ids_slice = bsp_tree_data.ids.as_slice();
        let ids_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BSP id buffer"),
            contents: bytemuck::cast_slice(ids_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let bsp_tree_slice = bsp_tree_data.bsp_tree.as_slice();
        let bsp_tree_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BSP tree buffer"),
            contents: bytemuck::cast_slice(bsp_tree_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let bsp_planes_slice = bsp_tree_data.bsp_planes.as_slice();
        let bsp_planes_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BSP plane buffer"),
            contents: bytemuck::cast_slice(bsp_planes_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        BspTreeGpu {
            _intermediates: bsp_tree_data,
            bbox_buffer,
            ids_buffer,
            bsp_tree_buffer,
            bsp_planes_buffer,
        }
    }
}

impl IntoGpu for BspTree {
    type Output = BspTreeGpu;

    fn into_gpu(&self, device: &wgpu::Device) -> Self::Output {
        Self::Output::new(&device, BspTreeIntermediate::new(&self))
    }
}