use crate::{bindings::WgslSource, data_structures::bvh::Bvh};

use super::{Bindable, WgslBindDescriptor, IntoGpu};

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

        let bvh_code = "res/shaders/bvh.wgsl";
        
        vec![
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "bspPlanes",
                var_type: "array<f32>",
                extra_code: Some(WgslSource::File(bvh_code)),
            }
        ]
    }
}

impl BvhGpu {
    pub fn new(device: &wgpu::Device) -> Self {
        BvhGpu {}
    }
}

impl IntoGpu for Bvh {
    type Output = BvhGpu;

    fn into_gpu(&self, device: &wgpu::Device) -> Self::Output {
        BvhGpu {}
    }
}