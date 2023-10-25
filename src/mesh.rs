use std::path::Path;

use wgpu::util::DeviceExt;

use crate::data_structures::{vector::{Vec3f32, Vec3u32}, bsp_tree::{AccObj, BspTree}, bbox::Bbox};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelVertex {
    pub position: Vec3f32,
    _padding: f32,
}

impl From<Vec3f32> for ModelVertex {
    fn from(value: Vec3f32) -> Self {
        Self {
            position: [value.0, value.1, value.2].into(),
            _padding: 0.0,
        }
    }
}

impl From<(f32, f32, f32)> for ModelVertex {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            position: [value.0, value.1, value.2].into(),
            _padding: 0.0,
        }
    }
}

impl std::fmt::Display for ModelVertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("x: {}, y: {}, z: {}", self.position[0], self.position[1], self.position[2]))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelIndex {
    pub triangle: Vec3u32,
    _padding: u32,
}

impl From<(u32, u32, u32)> for ModelIndex {
    fn from(value: (u32, u32, u32)) -> Self {
        Self {
            triangle: [value.0, value.1, value.2].into(),
            _padding: 0u32,
        }
    }
}

impl std::fmt::Display for ModelIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}, {}, {}", self.triangle[0], self.triangle[1], self.triangle[2]))
    }
}

///
/// Mesh type containing vertices and indices in two vecs
pub struct Mesh {
    vertices: Vec<ModelVertex>,
    indices: Vec<ModelIndex>,
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Mesh: {{\n"))?;
        f.write_fmt(format_args!("vertices: {{ \n"))?;
        for v in self.vertices.iter() {
            f.write_fmt(format_args!("\t{v}\n"))?;
        }
        f.write_fmt(format_args!("}} \n"))?;
        f.write_fmt(format_args!("indices: {{ \n"))?;
        for i in self.indices.iter() {
            f.write_fmt(format_args!("\t{i}\n"))?;
        }
        f.write_fmt(format_args!("}} \n"))?;
        Ok(())
    }
}

impl Mesh {
    pub fn bboxes(&self) -> Vec<AccObj> {
        self.indices.iter().enumerate().map(
            |(idx, triangle)| {
                AccObj::new(
                    idx.try_into().unwrap(),
                    Bbox::from_triangle(
                        self.vertices[triangle.triangle[0] as usize].position.into(),
                        self.vertices[triangle.triangle[1] as usize].position.into(),
                        self.vertices[triangle.triangle[2] as usize].position.into(),
                    )
                )
            }
        ).collect()
    }

    pub fn bsp_tree(&self) -> BspTree {
        BspTree::new(self.bboxes())
    }

    pub fn scale(&mut self, factor: f32) {
        self.vertices.iter_mut().for_each(|vert| {
            vert.position[0] = vert.position[0] * factor;
            vert.position[1] = vert.position[1] * factor;
            vert.position[2] = vert.position[2] * factor;
        });
    }
}

pub struct MeshGPU {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
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
        let mut indices_flat = vec![];
        models.iter().enumerate().for_each(|(idx, m)| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    (
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    )
                        .into()
                })
                .collect::<Vec<_>>();
            let total: u32 = (0..idx)
                .map(|i| models[i].mesh.positions.len() / 3)
                .sum::<usize>() as u32;

            let indices = (0..m.mesh.indices.len() / 3)
                .map(|i| ModelIndex {
                    triangle: [
                        total + m.mesh.indices[i * 3],
                        total + m.mesh.indices[i * 3 + 1],
                        total + m.mesh.indices[i * 3 + 2],
                    ].into(),
                    _padding: 0,
                })
                .collect::<Vec<_>>();
            vertices_flat.push(vertices);
            indices_flat.push(indices);
        });
        let vertices_flat = vertices_flat.into_iter().flatten().collect::<Vec<_>>();
        let indices_flat = indices_flat.into_iter().flatten().collect::<Vec<_>>();

        Ok(Self {
            vertices: vertices_flat,
            indices: indices_flat,
        })
    }

    pub fn into_gpu(&self, device: &wgpu::Device) -> MeshGPU {
        let vertex_buffer_slice = self.vertices.as_slice();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_buffer_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let index_buffer_slice = self.indices.as_slice();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Index Buffer"),
            contents: bytemuck::cast_slice(&index_buffer_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
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
                },
            ],
            label: Some("vertex_index_bind_group_layout"),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: index_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        MeshGPU {
            vertex_buffer,
            index_buffer,
            layout,
            bind_group,
        }
    }
}
