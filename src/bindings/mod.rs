use std::{fs::File, path::Path, io::prelude::*};

pub mod bsp_tree;
pub mod mesh;
pub mod storage_mesh;
pub mod texture;
pub mod uniform;
pub mod vertex;

pub trait Bindable {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry>;
    fn get_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry>;
    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor>;
}

pub fn create_bind_group_layouts(
    device: &wgpu::Device,
    mut layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
) -> wgpu::BindGroupLayout {
    let huge_flattened_layout_entry = layout_entries.iter_mut()
        .enumerate()
        .map(|(idx, item)| { let mut item = *item;
            item.binding = idx as u32;
            item
        }).collect::<Vec<_>>();
    

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: huge_flattened_layout_entry.as_slice(),
        label: None
    })
}

pub fn create_bind_groups(
    device: &wgpu::Device,
    bind_group_entries: &mut Vec<wgpu::BindGroupEntry>,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    bind_group_entries.iter_mut()
    .enumerate()
    .for_each(|(idx, item)| {
        item.binding = idx as u32;
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &bind_group_entries,
        label: None, // Some("uniform_bind_group"),
    })
}
pub fn create_shader_definitions(vec_of_descriptors: &Vec<Vec<WgslBindDescriptor>>) -> String {
    let mut string = String::new();
    vec_of_descriptors
        .iter()
        .flat_map(|v| v)
        .enumerate()
        .for_each(|(idx, descriptor)| {
            string.push_str(&generate_wgsl_string(
                0,
                idx as u32,
                descriptor,
            ));
        });
    string
}

pub trait BufferOwner {
    fn update_buffer(&self, queue: &wgpu::Queue);
}

pub trait IntoGpu {
    type Output;

    fn into_gpu(&self, device: &wgpu::Device) -> Self::Output;
}

#[derive(Clone, Copy)]
pub enum WgslSource<'a> {
    #[allow(dead_code)]
    Str(&'a str),
    File(&'a str), // important to avoid redefinitions
}

pub struct WgslBindDescriptor<'a> {
    pub struct_def: Option<&'a str>,
    pub bind_type: Option<&'a str>,
    pub var_name: &'a str,
    pub var_type: &'a str,
    pub extra_code: Option<WgslSource<'a>>,
}

fn generate_wgsl_string(
    group_id: u32,   // auto pick
    binding_id: u32, // auto pick
    bind_descriptor: &WgslBindDescriptor,
) -> String {
    assert!(group_id < 4);

    let WgslBindDescriptor {
        struct_def, // user provided or macro
        bind_type,  // user provided
        var_name,   // user provided
        var_type,   // user provided
        extra_code,
    } = *bind_descriptor; // user provided

    let bind_type = if let Some(bind_type) = bind_type {
        format!("<{bind_type}>")
    } else {
        "".into()
    };
    let mut string: String;
    let extra_code = match extra_code {
        None => "",
        Some(name) => match name {
            WgslSource::Str(string) => string,
            WgslSource::File(path) => {
                let mut file = File::open(Path::new(path)).expect(format!("File path {path} is invalid").as_str());
                string = String::new();
                file.read_to_string(&mut string).expect(format!("failed to read {path}").as_str());
                string.as_str()
            }
        },
    };

    format!(
        "
    {}\n
    @group({group_id}) @binding({binding_id})\n
    var{bind_type} {var_name}: {var_type};\n
    {}\n",
        struct_def.unwrap_or(""),
        extra_code
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn example() {
        let struct_def = "struct Uniform {
    camera_pos: vec3f,
    camera_constant: f32,
    camera_look_at: vec3f,
    aspect_ratio: f32,
    camera_up: vec3f,
};";

        let bind_type = "uniform";
        let var_name = "uniforms";
        let var_type = "Uniform";
        let group_id = 0;
        let binding_id = 0;

        generate_wgsl_string(
            group_id,
            binding_id,
            &WgslBindDescriptor {
                struct_def: Some(struct_def),
                bind_type: Some(bind_type),
                var_name,
                var_type,
                extra_code: None,
            },
        );
    }
}
