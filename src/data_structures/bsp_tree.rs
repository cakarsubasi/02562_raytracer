/// Types for Binary Space Partitioning Tree
/// Adapted from Javascript/C++ code provided by Jeppe Revall Frisvad,
/// which was inspired by BSP tree in GEL (http://www.imm.dtu.dk/GEL/)
/// originally written by Bent Dalgaard Larsen
/// License unspecified (used with permission)

use std::fmt::Debug;

use super::{
    bbox::{Bbox, BboxGpu},
    vector::Vec4u32,
};

const NODE_TYPE_LEAF: u32 = 3u32;

const F_EPS: f32 = 1e-6;

#[derive(Debug)]
pub struct BspTree {
    root: Node,
    max_depth: u32,
    bbox: Bbox,
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
    node_type: NodeType,
}

#[derive(Debug)]
enum NodeType {
    Leaf {
        objects: Vec<AccObj>,
    },
    Split {
        split: Split,
        plane: f32,
        left: Box<Node>,
        right: Box<Node>,
    },
}

impl BspTree {
    pub fn new(objects: Vec<AccObj>, max_depth: u32, max_objects_on_leaf: u32) -> Self {
        assert!(
            objects.len() < u32::MAX as usize,
            "We cannot deal with trees that contain more than 4 billion objects
        due to memory limitations"
        );
        assert!(
            max_depth > 0 && max_depth < 32,
            "BspTree depth should be positive and smaller than 32, got: {max_depth}"
        );
        assert!(
            max_objects_on_leaf > 0,
            "Leaf objects must be positive, got: {max_objects_on_leaf}"
        );
        let mut bbox = Bbox::new();
        // Extend the root node bounding box to include every other box
        objects.iter().for_each(|elem| {
            bbox.include_bbox(&elem.bbox);
        });

        let obj_refer = objects.iter().map(|obj| obj).collect();
        let root = Node::subdivide_node(bbox, 0, max_depth, max_objects_on_leaf, &obj_refer);

        Self {
            root,
            bbox,
            max_depth,
        }
    }

    pub fn count(&self) -> usize {
        self.root.count
    }

    pub fn primitive_ids(&self) -> Vec<u32> {
        let mut ids = Vec::with_capacity(self.count() as usize);

        fn primitive_ids_recursive(node: &Node, array: &mut Vec<u32>) {
            match &node.node_type {
                NodeType::Leaf { objects } => {
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
    ///  // .0 = (xxxx xxTT) : node_type
    ///  // .0 = (CCCC CCxx) : node_count
    ///  // .1 = node_id
    ///  // .2 = left_node_id
    ///  // .3 = right_node_id
    ///  
    /// ```
    ///
    pub fn bsp_array(&self) -> (Vec<f32>, Vec<Vec4u32>) {
        let bsp_tree_nodes: usize = (1 << (self.max_depth + 1)) - 1;
        let mut bsp_planes = vec![0.0; bsp_tree_nodes];
        let mut bsp_array = vec![Default::default(); bsp_tree_nodes];

        fn build_bsp_array_recursive(
            bsp_planes: &mut [f32],
            bsp_array: &mut [Vec4u32],
            node: &Node,
            depth: u32,
            max_depth: u32,
            branch: u32,
            node_id: &mut u32,
        ) {
            if depth > max_depth {
                return;
            }
            let idx = ((1 << depth) + branch - 1) as usize;
            bsp_array[idx].1 = 0;
            bsp_array[idx].2 = (1 << (depth + 1)) + 2 * branch - 1;
            bsp_array[idx].3 = (1 << (depth + 1)) + 2 * branch;
            bsp_planes[idx] = 0.0;
            match &node.node_type {
                NodeType::Leaf { objects } => {
                    bsp_array[idx].0 = NODE_TYPE_LEAF + (node.count << 2) as u32;
                    bsp_array[idx].1 = *node_id;
                    *node_id += objects.len() as u32;
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
                        depth + 1,
                        max_depth,
                        branch * 2,
                        node_id,
                    );
                    build_bsp_array_recursive(
                        bsp_planes,
                        bsp_array,
                        &right,
                        depth + 1,
                        max_depth,
                        branch * 2 + 1,
                        node_id,
                    );
                }
            }
        }

        build_bsp_array_recursive(
            &mut bsp_planes.as_mut_slice(),
            &mut bsp_array.as_mut_slice(),
            &self.root,
            0,
            self.max_depth,
            0,
            &mut 0,
        );

        (bsp_planes, bsp_array)
    }
}

impl Node {
    ///
    /// Create a complete Node hierarchy using subdivision
    fn subdivide_node(
        bbox: Bbox,
        depth: u32,
        max_depth: u32,
        max_objects_on_leaf: u32,
        objects: &Vec<&AccObj>,
    ) -> Node {
        let tests = 4;

        if objects.len() as u32 <= max_objects_on_leaf || depth == max_depth {
            let node = Node {
                count: objects.len(),
                node_type: NodeType::Leaf {
                    objects: objects.iter().map(|elem| (*elem).clone()).collect(),
                },
            };
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
            Node {
                count: objects.len(),
                node_type: NodeType::Split {
                    left: Box::new(Self::subdivide_node(
                        left_bbox,
                        depth + 1,
                        max_depth,
                        max_objects_on_leaf,
                        &left_objects,
                    )),
                    right: Box::new(Self::subdivide_node(
                        right_bbox,
                        depth + 1,
                        max_depth,
                        max_objects_on_leaf,
                        &right_objects,
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
    pub bbox: BboxGpu,
    pub ids: Vec<u32>,
    pub bsp_tree: Vec<Vec4u32>,
    pub bsp_planes: Vec<f32>,
}

impl BspTreeIntermediate {
    pub fn new(bsp_tree: &BspTree) -> Self {
        let ids = bsp_tree.primitive_ids();
        let (bsp_planes, bsp_tree_vec) = bsp_tree.bsp_array();
        Self {
            bbox: bsp_tree.bbox.into(),
            ids,
            bsp_tree: bsp_tree_vec,
            bsp_planes,
        }
    }
}

#[cfg(test)]
mod bsp_tree_test {
    use crate::mesh::Mesh;

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn bsp_tree_new() {
        let model = Mesh::from_obj("res/models/test_object.obj").expect("Failed to load model");
        let bboxes = model.bboxes();
        let bsp_tree = BspTree::new(bboxes, 20, 4);

        let mut set = HashSet::new();
        fn recurse(node: &Node, set: &mut HashSet<u32>) {
            match &node.node_type {
                NodeType::Leaf { objects } => {
                    for obj in objects {
                        set.insert(obj.idx);
                    }
                }
                NodeType::Split {
                    left,
                    right,
                    split: _,
                    plane: _,
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
        let bsp_tree = BspTree::new(bboxes, 20, 4);
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
