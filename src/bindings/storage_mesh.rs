use std::path::Path;

use wgpu::util::DeviceExt;

use crate::{
    bindings::WgslBindDescriptor,
    data_structures::{
        bbox::Bbox,
        bsp_tree::{AccObj, BspTree},
        vector::{vec3f32, vec3u32, Vec4f32, Vec4u32},
    },
};

use super::Bindable;

type ModelVertex = Vec4f32;
type ModelNormal = Vec4f32;
type ModelIndex = Vec4u32;

///
/// Mesh type containing vertices and indices in two vecs
pub struct Mesh {
    vertices: Vec<ModelVertex>,
    normals: Vec<ModelNormal>,
    indices: Vec<ModelIndex>,
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Mesh: {{\n"))?;
        f.write_fmt(format_args!("vertices: {{ \n"))?;
        for v in self.vertices.iter() {
            f.write_fmt(format_args!("\t{v:?}\n"))?;
        }
        f.write_fmt(format_args!("}} \n"))?;
        f.write_fmt(format_args!("indices: {{ \n"))?;
        for i in self.indices.iter() {
            f.write_fmt(format_args!("\t{i:?}\n"))?;
        }
        f.write_fmt(format_args!("}} \n"))?;
        Ok(())
    }
}

impl Mesh {
    pub fn bboxes(&self) -> Vec<AccObj> {
        self.indices
            .iter()
            .enumerate()
            .map(|(idx, triangle)| {
                AccObj::new(
                    idx.try_into().unwrap(),
                    Bbox::from_triangle(
                        self.vertices[triangle.0 as usize].xyz().into(),
                        self.vertices[triangle.1 as usize].xyz().into(),
                        self.vertices[triangle.2 as usize].xyz().into(),
                    ),
                )
            })
            .collect()
    }

    pub fn bsp_tree(&self) -> BspTree {
        BspTree::new(self.bboxes(), 20, 4)
    }

    #[allow(dead_code)]
    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn scale(&mut self, factor: f32) {
        self.vertices.iter_mut().for_each(|vert| {
            vert.0 = vert.0 * factor;
            vert.1 = vert.1 * factor;
            vert.2 = vert.2 * factor;
        });
    }
}

pub enum StorageMeshGpu {
    Split(StorageMeshGpuSplit),
    Combined(StorageMeshGpuCombined),
}

impl Bindable for StorageMeshGpu {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        match self {
            StorageMeshGpu::Split(split) => split.get_layout_entries(),
            StorageMeshGpu::Combined(combined) => combined.get_layout_entries(),
        }
    }

    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        match self {
            StorageMeshGpu::Split(split) => split.get_bind_group_entries(),
            StorageMeshGpu::Combined(combined) => combined.get_bind_group_entries(),
        }
    }

    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor> {
        match self {
            StorageMeshGpu::Split(split) => split.get_bind_descriptor(),
            StorageMeshGpu::Combined(combined) => combined.get_bind_descriptor(),
        }
    }
}

pub struct StorageMeshGpuSplit {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_normal_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl StorageMeshGpuSplit {
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

impl Bindable for StorageMeshGpuSplit {
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

pub struct StorageMeshGpuCombined {
    pub combined_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct CombinedVertexNormal {
    vertex: Vec4f32,
    normal: Vec4f32,
}

impl StorageMeshGpuCombined {
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

impl Bindable for StorageMeshGpuCombined {
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

impl Mesh {
    pub fn from_obj<P>(file_name: P) -> anyhow::Result<Mesh>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let (models, _materials_maybe) = tobj::load_obj(
            file_name,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )?;

        let mut vertices_flat = vec![];
        let mut normals_flat: Vec<Vec<Vec4f32>> = vec![];
        let mut indices_flat = vec![];
        models.iter().enumerate().for_each(|(idx, m)| {
            let size = m.mesh.positions.len() / 3;
            let mut vertices = Vec::with_capacity(size);
            let mut normals = Vec::with_capacity(size);
            for i in 0..size {
                vertices.push(
                    vec3f32(
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    )
                    .vec4(),
                );
                // TODO: allow optional normals
                normals.push(
                    vec3f32(
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    )
                    .vec4(),
                )
            }

            let total: u32 = (0..idx)
                .map(|i| models[i].mesh.positions.len() / 3)
                .sum::<usize>() as u32;

            let indices = (0..m.mesh.indices.len() / 3)
                .map(|i| {
                    vec3u32(
                        total + m.mesh.indices[i * 3],
                        total + m.mesh.indices[i * 3 + 1],
                        total + m.mesh.indices[i * 3 + 2],
                    )
                    .vec4()
                })
                .collect::<Vec<_>>();
            vertices_flat.push(vertices);
            normals_flat.push(normals);
            indices_flat.push(indices);
        });
        let vertices_flat = vertices_flat.into_iter().flatten().collect::<Vec<_>>();
        let normals_flat = normals_flat.into_iter().flatten().collect::<Vec<_>>();
        let indices_flat = indices_flat.into_iter().flatten().collect::<Vec<_>>();

        Ok(Self {
            vertices: vertices_flat,
            normals: normals_flat,
            indices: indices_flat,
        })
    }

    pub fn into_gpu_split(&self, device: &wgpu::Device) -> StorageMeshGpu {
        StorageMeshGpu::Split(StorageMeshGpuSplit::new(device, self))
    }

    pub fn into_gpu_combined(&self, device: &wgpu::Device) -> StorageMeshGpu {
        StorageMeshGpu::Combined(StorageMeshGpuCombined::new(device, self))
    }
}
