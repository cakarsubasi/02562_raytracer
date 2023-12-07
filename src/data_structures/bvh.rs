use rdst::RadixKey;

use crate::mesh::Mesh;

use super::{bbox::Bbox, vector::Vec3f32, bsp_tree::AccObj};


pub struct Bvh {
    root: Cluster,
    max_prims: u32,
}

struct Cluster {
    pub bbox: Bbox,
    pub cluster_type: ClusterType,
}

struct MortonPrimitive {
    pub index: u32,
    pub morton_code: u32,
}

impl RadixKey for MortonPrimitive {
    const LEVELS: usize = 4;

    fn get_level(&self, level: usize) -> u8 {
        (self.morton_code >> (level * 8)) as u8
    }
}

enum ClusterType {
    Leaf {
        primitives: Vec<u32>,
    },
    Interior {
        boxes: u32,
        left: Box<Cluster>,
        right: Box<Cluster>,
    },
}

impl Cluster {
    fn singleton(obj: &AccObj) -> Self {
        Cluster {
            bbox: obj.bbox,
            cluster_type: ClusterType::Leaf { primitives: vec![obj.idx] }
        }
    }

    fn combine(mut left: Cluster, right: Cluster, max_prims: u32) -> Self {
        if let (ClusterType::Leaf {primitives: ref mut left_primitives}, ClusterType::Leaf {primitives: ref right_primitives}) = (&mut left.cluster_type, &right.cluster_type) {
            if left_primitives.len() + right_primitives.len() < max_prims as usize {
                for elem in right_primitives {
                    left_primitives.push(*elem);
                }
                left.bbox.include_bbox(&right.bbox);
                return left;
            }
        }
        let mut bbox = left.bbox;
        let boxes = left.boxes() + right.boxes();
        bbox.include_bbox(&right.bbox);
        Cluster {
            bbox,
            cluster_type: ClusterType::Interior { boxes, left: Box::new(left), right: Box::new(right) }
        }

    }

    fn boxes(&self) -> u32 {
        match &self.cluster_type {
            ClusterType::Leaf { primitives: _ } => 1,
            ClusterType::Interior { boxes, left, right } => *boxes,
        }
    }

    fn gpu_node(&self) -> GpuNode {
        let min = self.bbox.min;
        let max = self.bbox.max;
        let (offset_ptr, number_of_prims) = match &self.cluster_type {
            ClusterType::Leaf { primitives: triangle } => (0, triangle.len() as u32),
            ClusterType::Interior { boxes, left, right } => (*boxes, 0),
        };

        GpuNode { min, offset_ptr, max, number_of_prims }
    }
}

impl Bvh {
    pub fn new(model: &Mesh, max_prims: u32) -> Self {
        let mut objects: Vec<_> = model.bboxes().iter().map(|accobj| Cluster::singleton(accobj)).collect();

        while objects.len() > 1 {
            let mut best = f32::INFINITY;
            let mut left = objects.len();
            let mut right = objects.len();
            for (i, obj1) in objects.iter().enumerate() {
                for (j, obj2) in objects.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    let distance = obj1.bbox.distance_center(&obj2.bbox);
                    if distance < best {
                        best = distance;
                        left = i;
                        right = j;
                    }
                }
            }
            let (left_cluster, right_cluster) = if left > right {
                (objects.remove(left),
                objects.remove(right))
            } else {
                (objects.remove(right),
                objects.remove(left))
            };

            let combined = Cluster::combine(left_cluster, right_cluster, max_prims);
            objects.push(combined);
        }
        Bvh {
            root: objects.pop().unwrap(),
            max_prims
        }
    }

    pub fn flatten(&self) -> Vec<GpuNode> {
        let mut nodes = vec![];
        let mut stack = vec![];

        stack.push(&self.root);
        while !stack.is_empty() {
            let top = stack.pop().unwrap();
            match &top.cluster_type {
                ClusterType::Leaf { primitives: triangle } => {

                },
                ClusterType::Interior { boxes, left, right } => {
                    stack.push(&right);
                    stack.push(&left);
                },
            }
            nodes.push(top.gpu_node());

        }


        nodes
    }
}


#[repr(C, align(16))]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GpuNode {
    min: Vec3f32,
    offset_ptr: u32,
    max: Vec3f32,
    number_of_prims: u32,
}

static_assertions::assert_eq_size!(GpuNode, [u32; 8]);

mod bvh_test {
    use crate::mesh::Mesh;

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn bvh_new() {
        let model = Mesh::from_obj("res/models/test_object.obj").expect("Failed to load model");
        let bvh = Bvh::new(&model, 4);
    }

    #[test]
    fn bvh_new2() {
        let model = Mesh::from_obj("res/models/CornellBox.obj").expect("Failed to load model");
        let bvh = Bvh::new(&model, 4);
    }
}