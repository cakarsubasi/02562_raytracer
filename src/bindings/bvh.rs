use crate::bindings::WgslSource;

use super::{Bindable, WgslBindDescriptor};

pub struct BvhGpu {
}

impl Bindable for BvhGpu {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![]
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        vec![]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        let aabb_definition =       
"struct Aabb {
    min: vec3f,
    _padding: f32,
    max: vec3f,
    _padding2: f32,
};";

        let aabb_code = "res/shaders/aabb.wgsl";

        let bsp_tree_code = "res/shaders/bvh.wgsl";
        
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
                extra_code: None,
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

impl BvhGpu {
    pub fn new(device: &wgpu::Device) -> Self {

        BvhGpu {}
    }
}