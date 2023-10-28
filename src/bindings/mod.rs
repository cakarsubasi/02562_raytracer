pub mod uniform;
pub mod vertex;
pub mod texture;
pub mod mesh;
pub mod bsp_tree;

pub trait Bindable {
    fn get_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry>;
    fn get_bind_group_entries(&self, device: &wgpu::Device) -> Vec<wgpu::BindGroupEntry>;
    fn get_bind_descriptor(&self) -> Vec<WgslBindDescriptor>;
}

pub fn create_bind_group_layouts(device: &wgpu::Device, layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>>) -> Vec<wgpu::BindGroupLayout> {
    let mut layouts = Vec::with_capacity(layout_entries.len());
    for entries in layout_entries {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: entries.as_ref(),
            label: None // Some("uniform_bind_group_layout"),
        });
        layouts.push(layout);
    }
    layouts
}

pub fn create_bind_groups<'a, 'b>(device: &wgpu::Device, bind_group_entries: Vec<Vec<wgpu::BindGroupEntry>>, bind_group_layouts: Vec<wgpu::BindGroupLayout>) -> Vec<wgpu::BindGroup> {
    let mut bind_groups = Vec::new();

    for (entries, layout) in bind_group_entries.iter().zip(bind_group_layouts) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: entries,
            label: None // Some("uniform_bind_group"),
        });
        bind_groups.push(bind_group);
    }
    bind_groups
}

pub fn create_shader() {
    todo!()
}

trait BufferOwner {
    fn update_buffer(&self, queue: &wgpu::Queue); 
}

pub trait IntoGpu {
    type Output;

    fn into_gpu(&self, device: &wgpu::Device) -> Self::Output;
}

pub struct WgslBindDescriptor<'a> {
    struct_def: Option<&'a str>,
    bind_type: &'a str,
    var_name: &'a str,
    var_type: &'a str,
    extra_code: Option<&'a str>,
}

fn generate_wgsl_string(
    struct_def: Option<&str>, // user provided or macro
    bind_type: &str, // user provided
    var_name: &str, // user provided
    var_type: &str, // user provided
    group_id: u32, // auto pick
    binding_id: u32, // auto pick
    extra_code: Option<&str>, // user provided
) -> String {
    format!("
    {}\n
    @group({group_id}) @binding({binding_id})\n
    var<{bind_type}> {var_name}: {var_type};\n
    {}\n",
    struct_def.unwrap_or(""),
    extra_code.unwrap_or(""))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn example() {
        let struct_def =
"struct Uniform {
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
            Some(struct_def),
            bind_type,
            var_name,
            var_type,
            group_id,
            binding_id,
            None,
        );
    }
}