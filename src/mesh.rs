use std::path::Path;

use crate::{
    bindings::storage_mesh::StorageMeshGpu,
    data_structures::{
        bbox::Bbox,
        bsp_tree::{AccObj, BspTree},
        vector::{vec3f32, Vec4f32, Vec4u32, vec4u32},
    },
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Material {
    diffuse: Vec4f32,
    ambient: Vec4f32,
    specular: Vec4f32,
    emissive: u32,
    _padding0: [u32; 3],
}

///
/// Mesh type containing vertices and indices in two vecs
pub struct Mesh {
    pub vertices: Vec<Vec4f32>,
    pub normals: Vec<Vec4f32>,
    /// last index in the indices contains material type
    pub indices: Vec<Vec4u32>,
    pub materials: Vec<Material>,
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

    pub fn from_obj<P>(file_name: P) -> anyhow::Result<Mesh>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let (models, materials_maybe) = tobj::load_obj(
            file_name,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )?;

        let materials = 
        if let Ok(materials_obj) = materials_maybe {
            materials_obj.iter().map( |m| {
                let diffuse = if let Some(diffuse) = m.diffuse {
                    diffuse.into()
                } else {
                    vec3f32(1.0, 1.0, 1.0)
                }.vec4();
                let ambient = if let Some(ambient) = m.ambient {
                    ambient.into()
                } else {
                    vec3f32(0.0, 0.0, 0.0)
                }.vec4();
                let specular = if let Some(specular) = m.specular {
                    specular.into()
                } else {
                    vec3f32(0.0, 0.0, 0.0)
                }.vec4();
                let emissive = if let Some(illumination) = m.illumination_model {
                    illumination as u32
                } else {
                    0
                };

                Material {
                    diffuse,
                    ambient,
                    specular,
                    emissive,
                    _padding0: [0, 0, 0],
                }
            }).collect()
        } else {
            vec![]
        };

        let mut vertices_flat = vec![];
        let mut normals_flat: Vec<Vec<Vec4f32>> = vec![];
        let mut indices_flat = vec![];
        models.iter().enumerate().for_each(|(idx, m)| {
            let position_number = m.mesh.positions.len() / 3;
            let normal_number = m.mesh.normals.len() / 3;
            let mut vertices = Vec::with_capacity(position_number);
            let mut normals = Vec::with_capacity(normal_number);
            for i in 0..position_number {
                vertices.push(
                    vec3f32(
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    )
                    .vec4(),
                );
            }
            if normal_number == position_number {
                for i in 0..normal_number {
                    normals.push(
                        vec3f32(
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        )
                        .vec4(),
                    );
                }
            } else {
                for _ in 0..position_number {
                    normals.push(
                        vec3f32(0.0, 0.0, 0.0)
                        .vec4(),
                    );
                }
            }


            let total: u32 = (0..idx)
                .map(|i| models[i].mesh.positions.len() / 3)
                .sum::<usize>() as u32;
            

            let indices = (0..m.mesh.indices.len() / 3)
                .map(|i| {
                    vec4u32(
                        total + m.mesh.indices[i * 3],
                        total + m.mesh.indices[i * 3 + 1],
                        total + m.mesh.indices[i * 3 + 2],
                        m.mesh.material_id.unwrap_or(u32::MAX as usize) as u32,
                    )
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
            materials,
        })
    }

    pub fn into_gpu_split(&self, device: &wgpu::Device) -> StorageMeshGpu {
        StorageMeshGpu::new_split(device, self)
    }

    pub fn into_gpu_combined(&self, device: &wgpu::Device) -> StorageMeshGpu {
        StorageMeshGpu::new_combined(device, self)
    }

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

#[cfg(test)]
mod mesh_test {

    use super::*;

    #[test]
    fn bsp_tree_new() {
        let _model = Mesh::from_obj("res/models/CornellBox.obj").expect("Failed to load model");
        println!("{:?}", _model.materials);
    }

}