use crate::{mesh::Mesh, data_structures::hlbvh::GpuNode};

use super::{bbox::Bbox, vector::Vec3f32};

#[derive(Debug)]
pub struct Bvh {
    root: Cluster,
    max_prims: u32,
    primitives: Vec<u32>,
    total_nodes: u32,
    //sorted_prims: Vec<MortonPrimitive>,
}

#[derive(Debug)]
struct Cluster {
    pub boxes: u32,
    pub bbox: Bbox,
    pub cluster_type: ClusterType,
}

#[derive(Debug)]
enum ClusterType {
    Leaf {
        start_idx: u32,
    },
    Interior {
        left: Box<Cluster>,
        right: Box<Cluster>,
    },
}


impl Cluster {
    fn singleton(bbox: &Bbox, index: u32) -> Self {
        Cluster {
            boxes: 1,
            bbox: *bbox,
            cluster_type: ClusterType::Leaf { start_idx: index }
        }
    }

    fn combine(left: Cluster, right: Cluster) -> Self {
        let mut bbox = left.bbox;
        let boxes = left.boxes + right.boxes;
        bbox.include_bbox(&right.bbox);
        Cluster {
            boxes,
            bbox,
            cluster_type: ClusterType::Interior { left: Box::new(left), right: Box::new(right) }
        }
    }

    fn collapse(&mut self) -> (u32, u32) {
        match &mut self.cluster_type {
            ClusterType::Leaf { start_idx } => (*start_idx, 1),
            ClusterType::Interior { left, right } => {
                let (start_idx, dropped_nodes_left) = left.collapse();
                let (_, dropped_nodes_right) = right.collapse();
                self.cluster_type = ClusterType::Leaf {
                    start_idx
                };
                (start_idx, dropped_nodes_left + dropped_nodes_right)
            },
        }
    }
}

impl Bvh {
    pub fn new(model: &Mesh, _max_prims: u32) -> Self {
        let primitives = (0..model.indices.len()).into_iter().map(|e| e as u32).collect();
        let mut objects: Vec<_> = model.bboxes().iter().zip(&primitives).map(|(accobj, &idx)| Cluster::singleton(&accobj.bbox, idx)).collect();
        let mut total_nodes = objects.len() as u32;
        
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
        let mut bvh = Bvh {
            root: objects.pop().unwrap(),
            max_prims: 1,
            primitives,
            total_nodes,
        };
        //bvh.collapse(4);
        bvh
    }

    pub fn collapse(&mut self, max_objects: u32) {
        self.max_prims = max_objects;
        fn collapse_recursive(node: &mut Cluster, max_objects: u32) -> u32 {
            if node.boxes <= max_objects {
                let (_, dropped) = node.collapse();
                dropped
            } else if let ClusterType::Interior {left, right} = &mut node.cluster_type {
                let left_dropped = collapse_recursive(left, max_objects);
                let right_dropped = collapse_recursive(right, max_objects);
                left_dropped + right_dropped
            } else {
                0
            }
        }
        let dropped = collapse_recursive(&mut self.root, max_objects);
        self.total_nodes -= dropped;
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
                ClusterType::Leaf { start_idx: primitive } => {
                    linear_node.number_of_prims = cluster.boxes;
                    linear_node.offset_ptr = *primitive;
                },
                ClusterType::Interior { left, right } => {
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


#[cfg(test)]
mod bvh_test {

    use super::*;

    #[test]
    fn bvh_new() {
        let model = Mesh::from_obj("res/models/test_object.obj").expect("Failed to load model");
        let bvh = Bvh::new(&model, 1);
        let flattened = bvh.flatten();
        println!("{:#?}", bvh);
        println!("{:#?}", flattened);
    }

    #[test]
    fn bvh_new2() {
        let model = Mesh::from_obj("res/models/CornellBox.obj").expect("Failed to load model");
        let bvh = Bvh::new(&model, 1);
    }
}