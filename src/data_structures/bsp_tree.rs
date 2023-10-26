use std::fmt::Debug;

use super::{
    bbox::{Bbox, BboxGpu},
    vector::Vec4u32,
};

const MAX_OBJECTS: u32 = 4;
const MAX_LEVEL: u32 = 20;
const F_EPS: f32 = 1e-6;
const D_EPS: f32 = 1e-12;

#[derive(Debug)]
pub struct BspTree {
    root: Node,
}

/// Intermediate data structure to pass
/// indexed bounding boxes to the BSP Tree
///
/// Each index points towards the primitive (in this case the triangle)
/// in the index buffer that corresponds to the bbox
#[derive(Debug, Copy, Clone)]
pub struct AccObj {
    idx: u32,
    bbox: Bbox,
}

impl AccObj {
    pub fn new(idx: u32, bbox: Bbox) -> Self {
        Self { idx, bbox }
    }
}

const NODE_TYPE_LEAF: u32 = 3u32;

impl BspTree {
    pub fn new(objects: Vec<AccObj>) -> Self {
        assert!(
            objects.len() < u32::MAX as usize,
            "We cannot deal with trees that contain more than 4 billion objects
        due to memory limitations"
        );
        let mut bbox = Bbox::new();
        //let max_level = MAX_LEVEL;
        //let count = objects.len() as u32;
        // Extend the root node bounding box to include every other box
        objects.iter().for_each(|elem| {
            bbox.include_bbox(&elem.bbox);
        });
        let obj_refer = objects.iter().map(|obj| obj).collect();
        let root = Node::subdivide_node(bbox, 0, &obj_refer, &mut Vec::new());

        Self { root }
    }

    pub fn count(&self) -> usize {
        self.root.count
    }

    pub fn primitive_ids(&self) -> Vec<u32> {
        let mut ids = Vec::with_capacity(self.count() as usize);

        fn primitive_ids_recursive(node: &Node, array: &mut Vec<u32>) {
            match &node.node_type {
                NodeType::Leaf { objects, id: _ } => {
                    objects.iter().for_each(|obj| array.push(obj.idx));
                }
                NodeType::Split {
                    left,
                    right,
                    split: _,
                    plane: _,
                } => {
                    primitive_ids_recursive(&left, array);
                    primitive_ids_recursive(&right, array);
                }
            }
        }

        primitive_ids_recursive(&self.root, &mut ids);
        ids
    }

    ///
    /// Constructs bsp_planes and bsp_array
    ///
    /// ```
    /// bsp_planes: Vec<f32> = vec![];
    /// ```
    ///
    /// ```
    /// bsp_array: Vec<Vec4u32> = vec![];
    ///  // .0 = (xxxx xx00) : node_type
    ///  // .0 = (0000 00xx) : node_count
    ///  // .1 = node_id
    ///  // .2 = left_node_id
    ///  // .3 = right_node_id
    ///  
    /// ```
    ///
    pub fn bsp_array(&self) -> (Vec<f32>, Vec<Vec4u32>) {
        const BSP_TREE_NODES: usize = (1 << (MAX_LEVEL + 1)) - 1;
        //let mut bsp_planes: [f32; BSP_TREE_NODES] = [0.0; BSP_TREE_NODES];
        //let mut bsp_array: [Vec4u32; BSP_TREE_NODES] = [Default::default(); BSP_TREE_NODES];
        let mut bsp_planes = vec![0.0; BSP_TREE_NODES];
        let mut bsp_array = vec![Default::default(); BSP_TREE_NODES];

        fn build_bsp_array_recursive(
            bsp_planes: &mut [f32],
            bsp_array: &mut [Vec4u32],
            node: &Node,
            level: u32,
            branch: u32,
            id: &mut u32,
        ) {
            if level > MAX_LEVEL {
                return;
            }
            let idx = ((1 << level) + branch - 1) as usize;
            bsp_array[idx].1 = 0;
            bsp_array[idx].2 = (1 << (level + 1)) + 2 * branch - 1;
            bsp_array[idx].3 = (1 << (level + 1)) + 2 * branch;
            bsp_planes[idx] = 0.0;
            match &node.node_type {
                NodeType::Leaf { objects, id } => {
                    bsp_array[idx].0 = NODE_TYPE_LEAF + (node.count << 2) as u32;
                    bsp_array[idx].1 = *id;
                    //*id = objects.len() as u32 + *id;
                    //println!("id: {id}");
                }
                NodeType::Split {
                    left,
                    right,
                    split,
                    plane,
                } => {
                    bsp_array[idx].0 = *split as u32 + (node.count << 2) as u32;
                    bsp_planes[idx] = *plane;
                    build_bsp_array_recursive(
                        bsp_planes,
                        bsp_array,
                        &left,
                        level + 1,
                        branch * 2,
                        id,
                    );
                    build_bsp_array_recursive(
                        bsp_planes,
                        bsp_array,
                        &right,
                        level + 1,
                        branch * 2 + 1,
                        id,
                    );
                }
            }
        }

        build_bsp_array_recursive(
            &mut bsp_planes.as_mut_slice(),
            &mut bsp_array.as_mut_slice(),
            &self.root,
            0,
            0,
            &mut 0,
        );

        (Vec::from(bsp_planes), Vec::from(bsp_array))
    }

    pub fn into_gpu(&self, device: &wgpu::Device) -> BspTreeGpu {
        BspTreeIntermediate::new(self).into_gpu(&device)
    }
}

#[derive(Debug, Copy, Clone)]
enum Split {
    AxisX = 0,
    AxisY = 1,
    AxisZ = 2,
}

impl From<u32> for Split {
    fn from(value: u32) -> Self {
        match value {
            0 => Split::AxisX,
            1 => Split::AxisY,
            2 => Split::AxisZ,
            _ => panic!("unexpected input {value}"),
        }
    }
}

#[derive(Debug)]
struct Node {
    count: usize,
    bbox: Bbox,
    node_type: NodeType,
}

#[derive(Debug)]
enum NodeType {
    Leaf {
        id: u32,
        objects: Vec<AccObj>,
    },
    Split {
        split: Split,
        plane: f32,
        left: Box<Node>,
        right: Box<Node>,
    },
}

impl Node {
    ///
    /// Create a complete Node hierarchy using subdivision
    fn subdivide_node(
        bbox: Bbox,
        level: u32,
        objects: &Vec<&AccObj>,
        tree_objects: &mut Vec<AccObj>,
    ) -> Node {
        let tests = 4;

        if objects.len() as u32 <= MAX_OBJECTS || level == MAX_LEVEL {
            let node = Node {
                count: objects.len(),
                bbox,
                node_type: NodeType::Leaf {
                    objects: objects.iter().map(|elem| (*elem).clone()).collect(),
                    id: tree_objects.len() as u32,
                },
            };
            for obj in objects {
                tree_objects.push(*obj.to_owned());
            }

            node
        } else {
            // split the objects
            let mut axis_leaf = 0;
            let mut plane: f32 = 0.0;
            let mut left_node_count = 0;
            let mut right_node_count = 0;
            let mut _debug = false;
            let mut min_cost = 1E+27;
            for i in 0..3 {
                for k in 1..tests {
                    let mut left_bbox = bbox.clone();
                    let mut right_bbox = bbox.clone();
                    let max_corner = bbox.max[i];
                    let min_corner = bbox.min[i];
                    let center =
                        (max_corner - min_corner) * (k as f32) / (tests as f32) + min_corner;
                    left_bbox.max[i] = center;
                    right_bbox.min[i] = center;

                    let mut left_count = 0;
                    let mut right_count = 0;
                    for obj in objects {
                        left_count += left_bbox.intersects(&obj.bbox) as i32;
                        right_count += right_bbox.intersects(&obj.bbox) as i32;
                    }

                    let cost = left_count as f32 * left_bbox.area()
                        + right_count as f32 * right_bbox.area();
                    if cost < min_cost {
                        min_cost = cost;

                        axis_leaf = i;
                        plane = center;
                        left_node_count = left_count;
                        right_node_count = right_count;
                    }
                }
            }

            // Choose the splitting plane
            let max_corner = bbox.max[axis_leaf];
            let min_corner = bbox.min[axis_leaf];
            let size = max_corner - min_corner;
            let diff = if F_EPS < (size / 8.0) {
                size / 8.0
            } else {
                F_EPS
            };
            let mut center = plane;

            if left_node_count == 0 {
                center = max_corner;
                for obj in objects {
                    let obj_min_corner = obj.bbox.min[axis_leaf];
                    if obj_min_corner < center {
                        center = obj_min_corner;
                    }
                }
                center -= diff;
            }
            if right_node_count == 0 {
                center = min_corner;
                for obj in objects {
                    let obj_max_corner = obj.bbox.max[axis_leaf];
                    if obj_max_corner > center {
                        center = obj_max_corner;
                    }
                }

                center += diff;
            }

            plane = center;
            let mut left_bbox = bbox.clone();
            let mut right_bbox = bbox.clone();
            left_bbox.max[axis_leaf] = center;
            right_bbox.min[axis_leaf] = center;

            let mut left_objects = vec![];
            let mut right_objects = vec![];

            for obj in objects {
                if left_bbox.intersects(&obj.bbox) {
                    left_objects.push(*obj);
                }
                if right_bbox.intersects(&obj.bbox) {
                    right_objects.push(*obj);
                }
            }
            log::debug!("Hello");
            Node {
                count: objects.len(),
                bbox: bbox,
                node_type: NodeType::Split {
                    left: Box::new(Self::subdivide_node(
                        left_bbox,
                        level + 1,
                        &left_objects,
                        tree_objects,
                    )),
                    right: Box::new(Self::subdivide_node(
                        right_bbox,
                        level + 1,
                        &right_objects,
                        tree_objects,
                    )),
                    split: axis_leaf.into(),
                    plane: plane,
                },
            }
        }
    }
}

#[derive(Debug)]
pub struct BspTreeIntermediate {
    bbox: BboxGpu,
    ids: Vec<u32>,
    bsp_tree: Vec<Vec4u32>,
    bsp_planes: Vec<f32>,
}

impl BspTreeIntermediate {
    fn new(bsp_tree: &BspTree) -> Self {
        let ids = bsp_tree.primitive_ids();
        let (bsp_planes, bsp_tree_vec) = bsp_tree.bsp_array();
        Self {
            bbox: bsp_tree.root.bbox.into(),
            ids,
            bsp_tree: bsp_tree_vec,
            bsp_planes,
        }
    }

    fn into_gpu(self, device: &wgpu::Device) -> BspTreeGpu {
        use wgpu::util::DeviceExt;
        println!("{:?}", self.bbox);
        let bbox_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bounding Box Uniform"),
            contents: bytemuck::cast_slice(&[self.bbox]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let ids_slice = self.ids.as_slice();
        let ids_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BSP id buffer"),
            contents: bytemuck::cast_slice(ids_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let bsp_tree_slice = self.bsp_tree.as_slice();
        let bsp_tree_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BSP tree buffer"),
            contents: bytemuck::cast_slice(bsp_tree_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let bsp_planes_slice = self.bsp_planes.as_slice();
        let bsp_planes_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BSP plane buffer"),
            contents: bytemuck::cast_slice(bsp_planes_slice),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
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
            ],
            label: Some("vertex_index_bind_group_layout"),
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: bbox_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: ids_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bsp_tree_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: bsp_planes_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        BspTreeGpu {
            _intermediates: self,
            ids: ids_buffer,
            bsp_tree: bsp_tree_buffer,
            bsp_planes: bsp_planes_buffer,
            layout,
            bind_group,
        }
    }
}

pub struct BspTreeGpu {
    // need to hold intermediates so they don't get dropped
    _intermediates: BspTreeIntermediate,
    pub ids: wgpu::Buffer,
    pub bsp_tree: wgpu::Buffer,
    pub bsp_planes: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

//impl BspTreeGpu {
//    pub fn intermediate(&self) -> &BspTreeIntermediate {
//        &self._intermediates
//    }
//}

#[cfg(test)]
mod bsp_tree_test {
    use crate::mesh::Mesh;

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn bsp_tree_new() {
        let mut model = Mesh::from_obj("res/models/test_object.obj").expect("Failed to load model");
        //model.scale(1.0 / 500.0);
        let bboxes = model.bboxes();
        let bsp_tree = BspTree::new(bboxes);
        println!("{bsp_tree:#?}");
        println!("{:?}", bsp_tree.root.bbox);

        let mut set = HashSet::new();
        fn recurse(node: &Node, set: &mut HashSet<u32>) {
            match &node.node_type {
                NodeType::Leaf { objects, id } => {
                    for obj in objects {
                        set.insert(obj.idx);
                    }
                }
                NodeType::Split {
                    left,
                    right,
                    split,
                    plane,
                } => {
                    recurse(&left, set);
                    recurse(&right, set);
                }
            }
        }
        recurse(&bsp_tree.root, &mut set);

        for i in 0..model.index_count() {
            assert!(set.contains(&i));
        }

        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::create("example_output_rust.txt").unwrap();
        write!(file, "{:#?}", bsp_tree.root).unwrap();
    }

    #[test]
    fn bsp_tree_ids() {
        use std::collections::HashSet;
        let mut model = Mesh::from_obj("res/models/CornellBox.obj").expect("Failed to load model");
        model.scale(1.0 / 500.0);
        let bboxes = model.bboxes();
        let bsp_tree = BspTree::new(bboxes);
        let (_, bsp_array) = bsp_tree.bsp_array();
        let mut test_map: HashSet<u32> = HashSet::new();
        let mut id: usize = 0;
        loop {
            if id == bsp_array.len() {
                break;
            }
            let bsp_elem = bsp_array[id];
            id += 1;
            if bsp_elem.0 == u32::MAX {
                continue;
            }
            let node_type = bsp_elem.0 & 3u32;
            if node_type == NODE_TYPE_LEAF {
                if !test_map.insert(bsp_elem.1) {
                    assert!(false, "num: {id}\n {:?}", bsp_array.split_at(id + 1).0)
                }
            }
        }
    }
}
