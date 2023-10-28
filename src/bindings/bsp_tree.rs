use wgpu::util::DeviceExt;

use crate::{data_structures::bsp_tree::{BspTreeIntermediate, BspTree}, bindings::WgslBindDescriptor};

use super::{Bindable, IntoGpu, WgslSource};

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

        let aabb_code =
"fn intersect_min_max(r: ptr<function, Ray>) -> bool
{
    let p1 = (aabb.min - (*r).origin)/(*r).direction;
    let p2 = (aabb.max - (*r).origin)/(*r).direction;
    let pmin = min(p1, p2);
    let pmax = max(p1, p2);
    let tmin = max(pmin.x, max(pmin.y, pmin.z));
    let tmax = min(pmax.x, min(pmax.y, pmax.z));
    if (tmin > tmax || tmin > (*r).tmax || tmax < (*r).tmin) {
          return false;
    }
    (*r).tmin = max(tmin - 1.0e-4f, (*r).tmin);
    (*r).tmax = min(tmax + 1.0e-4f, (*r).tmax);
    return true;
}";

let bsp_tree_code =
" const MAX_LEVEL = 20u;
const BSP_LEAF = 3u;
var<private> branch_node: array<vec2u, MAX_LEVEL>;
var<private> branch_ray: array<vec2f, MAX_LEVEL>;

fn intersect_trimesh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool
{
   var branch_lvl: u32 = 0u;
   var near_node: u32 = 0u;
   var far_node: u32 = 0u;
   var t: f32 = 0.0;
   var node: u32 = 0u;

   for (var i = 0u; i <= MAX_LEVEL; i++) {
       let tree_node: vec4u = bspTree[node];
       let node_axis_leaf = tree_node.x&3u;

       if (node_axis_leaf == BSP_LEAF) {
           // A leaf was found
           let node_count = tree_node.x>>2u;
           let node_id = tree_node.y;
           var found = false;
           
           for (var j = 0u; j < node_count; j++) {
               let obj_idx = treeIds[node_id + j];

               if (intersect_triangle_indexed(r, hit, obj_idx)) {
                   (*r).tmax = (*hit).dist;
                   found = true;
               }
           }

           if (found) {
               return true;
           } else if (branch_lvl == 0u) {
               return false;
           } else {
               branch_lvl--;
               i = branch_node[branch_lvl].x;
               node = branch_node[branch_lvl].y;
               (*r).tmin = branch_ray[branch_lvl].x;
               (*r).tmax = branch_ray[branch_lvl].y;
               continue;
           }
       }

       let axis_direction = (*r).direction[node_axis_leaf];
       let axis_origin = (*r).origin[node_axis_leaf];

       if (axis_direction >= 0.0f) {
           near_node = tree_node.z; // left
           far_node = tree_node.w; // right
       } else {
           near_node = tree_node.w; // right
           far_node = tree_node.z; // left
       }

       let node_plane = bspPlanes[node];
       let denom = select(axis_direction, 1.0e-8f, abs(axis_direction) < 1.0e-8f);
       t = (node_plane - axis_origin) / denom;

       if(t > (*r).tmax) { 
           node = near_node; 
       } else if (t < (*r).tmin) { 
           node = far_node; 
       } else {
           branch_node[branch_lvl].x = i;
           branch_node[branch_lvl].y = far_node;
           branch_ray[branch_lvl].x = t;
           branch_ray[branch_lvl].y = (*r).tmax;
           branch_lvl++;
           (*r).tmax = t;
           node = near_node;
       }
   }
   return false;
}";
        
        vec![
            WgslBindDescriptor {
                struct_def: Some(aabb_definition),
                bind_type: Some("uniform"),
                var_name: "aabb",
                var_type: "Aabb",
                extra_code: Some(WgslSource::Str(aabb_code)),
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
                extra_code: Some(WgslSource::Str(bsp_tree_code)),
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