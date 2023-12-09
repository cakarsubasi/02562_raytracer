use rdst::RadixKey;

use crate::mesh::Mesh;

use super::{bbox::Bbox, vector::Vec3f32, bsp_tree::AccObj};

#[derive(Debug)]
pub struct Bvh {
    root: Cluster,
    max_prims: u32,
    primitives: Vec<u32>,
    total_nodes: u32,
    //sorted_prims: Vec<MortonPrimitive>,
}

pub struct BvhBuildNode {
    
}

#[derive(Debug)]
struct Cluster {
    pub bbox: Bbox,
    pub cluster_type: ClusterType,
}

struct MortonPrimitive {
    pub index: u32,
    pub morton_code: u32, // use 30 bits
}

impl RadixKey for MortonPrimitive {
    const LEVELS: usize = 4;

    fn get_level(&self, level: usize) -> u8 {
        (self.morton_code >> (level * 8)) as u8
    }
}

#[derive(Debug)]
enum ClusterType {
    Leaf {
        primitive: u32,
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
            cluster_type: ClusterType::Leaf { primitive: obj.idx }
        }
    }

    fn combine(mut left: Cluster, right: Cluster) -> Self {
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
            ClusterType::Leaf { primitive: _ } => 1,
            ClusterType::Interior { boxes, left, right } => *boxes,
        }
    }

}

impl Bvh {
    pub fn new_n3(model: &Mesh) -> Self {
        let mut objects: Vec<_> = model.bboxes().iter().map(|accobj| Cluster::singleton(accobj)).collect();
        let mut total_nodes = objects.len() as u32;
        let primitives = (0..model.indices.len()).into_iter().map(|e| e as u32).collect();
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

            let combined = Cluster::combine(left_cluster, right_cluster);
            total_nodes += 1;
            objects.push(combined);
        }
        Bvh {
            root: objects.pop().unwrap(),
            max_prims: 1,
            primitives,
            total_nodes,
        }
    }

    pub fn flatten(&self) -> Vec<GpuNode> {
        let mut nodes = vec![GpuNode::new(&self.root.bbox); self.total_nodes as usize];

        fn flatten_recursive(nodes: &mut Vec<GpuNode>, cluster: &Cluster, offset: &mut u32) -> u32 {
            let current_offset = *offset;
            let mut linear_node = nodes[*offset as usize];
            linear_node.max = cluster.bbox.max;
            linear_node.min = cluster.bbox.min;
            *offset += 1;
            let node_offset = *offset;
            match &cluster.cluster_type {
                ClusterType::Leaf { primitive } => {
                    linear_node.number_of_prims = 1;
                    linear_node.offset_ptr = *primitive;
                },
                ClusterType::Interior { boxes: _, left, right } => {
                    linear_node.number_of_prims = 0;
                    flatten_recursive(nodes, left, offset);
                    linear_node.offset_ptr = flatten_recursive(nodes, right, offset);
                },
            }
            nodes[current_offset as usize] = linear_node;
            node_offset
        }
        flatten_recursive(&mut nodes, &self.root, &mut 0);


        nodes
    }

    pub fn triangles(&self) -> &Vec<u32> {
        // TODO
        &self.primitives
    }
}


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GpuNode {
    min: Vec3f32,
    offset_ptr: u32,
    max: Vec3f32,
    number_of_prims: u32,
}

impl GpuNode {
    fn new(bbox: &Bbox) -> Self{
        GpuNode {
            min: bbox.min,
            offset_ptr: 9999,
            max: bbox.max,
            number_of_prims: 9999,
        }
    }
}



static_assertions::assert_eq_size!(GpuNode, [u32; 8]);

#[cfg(test)]
mod bvh_test {

    use super::*;

    #[test]
    fn bvh_new() {
        let model = Mesh::from_obj("res/models/test_object.obj").expect("Failed to load model");
        let bvh = Bvh::new_n3(&model);
        let flattened = bvh.flatten();
        println!("{:#?}", bvh);
        println!("{:#?}", flattened);
    }

    #[test]
    fn bvh_new2() {
        let model = Mesh::from_obj("res/models/CornellBox.obj").expect("Failed to load model");
        let bvh = Bvh::new_n3(&model);
    }
}