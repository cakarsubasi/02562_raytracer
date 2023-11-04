use wgpu::util::DeviceExt;

use crate::{
    bindings::WgslBindDescriptor,
    data_structures::vector::Vec4f32, mesh::Mesh,
};

use super::Bindable;

pub struct StorageMeshGpu {
    geometry: GeometryGpu,
    materials: MaterialsGpu,
}

impl StorageMeshGpu {
    pub fn new_split(device: &wgpu::Device, mesh: &Mesh) -> Self {
        Self {
            geometry: GeometryGpu::Split(GeometryGpuSplit::new(device, mesh)),
            materials: MaterialsGpu::new(device, mesh),
        }
    }

    pub fn new_combined(device: &wgpu::Device, mesh: &Mesh) -> Self {
        Self {
            geometry: GeometryGpu::Combined(GeometryGpuCombined::new(device, mesh)),
            materials: MaterialsGpu::new(device, mesh),
        }
    }
}

impl Bindable for StorageMeshGpu {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut layout_entries = self.geometry.get_layout_entries();
        layout_entries.append(&mut self.materials.get_layout_entries());
        layout_entries
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        let mut bind_group_entries = self.geometry.get_bind_group_entries();
        bind_group_entries.append(&mut self.materials.get_bind_group_entries());
        bind_group_entries
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        let mut bind_descriptors = self.geometry.get_bind_descriptor();
        bind_descriptors.append(&mut self.materials.get_bind_descriptor());
        bind_descriptors
    }
}

enum GeometryGpu {
    Split(GeometryGpuSplit),
    Combined(GeometryGpuCombined),
}

impl Bindable for GeometryGpu {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        match self {
            GeometryGpu::Split(split) => split.get_layout_entries(),
            GeometryGpu::Combined(combined) => combined.get_layout_entries(),
        }
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        match self {
            GeometryGpu::Split(split) => split.get_bind_group_entries(),
            GeometryGpu::Combined(combined) => combined.get_bind_group_entries(),
        }
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        match self {
            GeometryGpu::Split(split) => split.get_bind_descriptor(),
            GeometryGpu::Combined(combined) => combined.get_bind_descriptor(),
        }
    }
}

struct GeometryGpuSplit {
    vertex_buffer: wgpu::Buffer,
    vertex_normal_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl GeometryGpuSplit {
    pub fn new(device: &wgpu::Device, mesh: &Mesh) -> Self {
        let vertex_buffer_slice = mesh.vertices.as_slice();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_buffer_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        let vertex_normal_slice = mesh.normals.as_slice();
        let vertex_normal_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_normal_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let index_buffer_slice = mesh.indices.as_slice();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Index Buffer"),
            contents: bytemuck::cast_slice(&index_buffer_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        Self {
            vertex_buffer,
            vertex_normal_buffer,
            index_buffer,
        }
    }
}

impl Bindable for GeometryGpuSplit {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                // vertex position
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
                // vertex normal
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
                // index buffer
                binding: 2,
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
                resource: self.vertex_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: self.vertex_normal_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: self.index_buffer.as_entire_binding(),
            },
        ]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        // TODO: need to differentiate names
        vec![
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "vertexBuffer",
                var_type: "array<vec4f>",
                extra_code: None,
            },
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "normalBuffer",
                var_type: "array<vec4f>",
                extra_code: None,
            },
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "indexBuffer",
                var_type: "array<vec4u>",
                extra_code: None,
            },
        ]
    }
}

struct GeometryGpuCombined {
    combined_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct CombinedVertexNormal {
    vertex: Vec4f32,
    normal: Vec4f32,
}

impl GeometryGpuCombined {
    pub fn new(device: &wgpu::Device, mesh: &Mesh) -> Self {
        let combined_slice = mesh
            .vertices
            .iter()
            .zip(&mesh.normals)
            .map(|(vertex, normal)| CombinedVertexNormal {
                vertex: *vertex,
                normal: *normal,
            })
            .collect::<Vec<_>>();
        let combined_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Vertex Buffer"),
            contents: bytemuck::cast_slice(&combined_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        let index_buffer_slice = mesh.indices.as_slice();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Index Buffer"),
            contents: bytemuck::cast_slice(&index_buffer_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        Self {
            combined_buffer,
            index_buffer,
        }
    }
}

impl Bindable for GeometryGpuCombined {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                // vertex combined
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
                // index buffer
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
        vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: self.combined_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: self.index_buffer.as_entire_binding(),
            },
        ]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        let struct_def = "struct VertexNormal {
    position: vec4f,
    normal: vec4f,
};";
        vec![
            WgslBindDescriptor {
                struct_def: Some(struct_def),
                bind_type: Some("storage"),
                var_name: "combinedBuffer",
                var_type: "array<VertexNormal>",
                extra_code: None,
            },
            WgslBindDescriptor {
                struct_def: None,
                bind_type: Some("storage"),
                var_name: "indexBuffer",
                var_type: "array<vec4u>",
                extra_code: None,
            },
        ]
    }
}

struct MaterialsGpu {

}
impl MaterialsGpu {
    fn new(device: &wgpu::Device, mesh: &Mesh) -> Self {
        
        Self{}
    }
}

impl Bindable for MaterialsGpu {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> { 
        vec![]
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        vec![]
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        vec![]
    }
}