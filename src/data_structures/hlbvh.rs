use rayon::prelude::*;
use rdst::{RadixKey, RadixSort};
use std::{
    cmp::Ord,
    sync::atomic::{AtomicU32, AtomicUsize},
};

use crate::mesh::Mesh;

use super::{
    accobj::{AccObj, Split},
    bbox::Bbox,
    vector::Vec3f32,
};

/// Bounding Volume Hierarchy type
#[derive(Debug)]
pub struct Bvh {
    /// root node of the BVH
    root: BvhBuildNode,
    /// sorted array of primitives, we need to have primitives in the leaf
    /// nodes next to one another so we only have to store the pointer offset
    /// and number of elements, so we need a second storage buffer on the GPU
    /// that contains the actual indices we want to access
    primitives: Vec<AccObj>,
    // total number of nodes in the BVH
    total_nodes: u32,
}

impl Bvh {
    /// Construct a BVH using the Hierarchical Linear BVH method described in the PBR book:
    /// https://www.pbr-book.org/4ed/Primitives_and_Intersection_Acceleration/Bounding_Volume_Hierarchies
    ///
    pub fn new(model: &Mesh, max_prims: u32) -> Self {
        let primitives = model.bboxes();
        // calculate the overall boundary for morton code generation
        let mut bound = Bbox::new();
        for bbox in &primitives {
            bound.include_vertex(bbox.bbox.center());
        }
        // allocate and initialize the sorted primitive array
        let mut ordered_primitives = vec![AccObj::new(0, Bbox::new()); primitives.len()];

        // generate morton codes for the primitives
        // use 30 bit morton codes as described in the PBR book
        let morton_bits = 10;
        let morton_scale = 1 << morton_bits;

        // The idea is that we convert each primitive to a single coordinate between (0, 0, 0) and (1, 1, 1)
        // relative to the overall boundary of the object. Using 32-bit floats, we only need 10-bits of fixed
        // precision to represent this range, but this will degrade quality on very large scenes
        let mut morton_primitives = vec![MortonPrimitive::new(); primitives.len()];
        morton_primitives
            .iter_mut()
            .enumerate()
            .for_each(|(idx, morton)| {
                morton.index = idx as u32;
                let centroid_offset = bound.offset(primitives[idx].bbox.center());
                let offset = centroid_offset * morton_scale as f32;
                morton.morton_code = encode_morton_3(offset.0, offset.1, offset.2);
            });

        // Sort primitives using morton codes
        if cfg!(debug_assertions) {
            morton_primitives.sort_unstable();
        } else {
            // It appears that the rdst crate relies on well defined unsigned underflow behavior that panics
            // on Rust debug builds, since I can't do much about this without editing that crate's source
            // code, I am just going to put this behind the release flag since it does work in that case
            morton_primitives.radix_sort_unstable();
        }
        //println!("completed sort");

        // Initialize treelets by pooling primitives that have the same most significant 12-bits
        // in their morton code
        let mut treelets_to_build = vec![];
        let mask = 0b0011_1111_1111_1100_0000_0000_0000_0000u32;
        let mut start = 0;
        let mut end = 1;
        while end <= morton_primitives.len() {
            if end == morton_primitives.len()
                || ((morton_primitives[start].morton_code & mask)
                    != (morton_primitives[end].morton_code & mask))
            {
                let num_primitives = end - start;
                //let treelet = LBvhTreeLet::new(start, num_primitives);
                treelets_to_build.push((start, num_primitives));

                start = end;
            }
            end += 1;
        }

        //println!("Initialized treelets: {}", treelets_to_build.len());

        // Create subtrees from treelets in parallel.
        // As we are going to collapse the entire tree into a single array at the end, we need
        // treelets to agree which parts of the array belong to them. This can be coordinated
        // locklessly using a single atomic variable (as described in the PBR book)
        //
        // Normally, Rust does not allow multiple threads to have mutable
        let total_nodes = AtomicU32::new(0);
        let ordered_prims_offset = AtomicUsize::new(0);
        let treelets = treelets_to_build
            .par_iter_mut()
            .map(|treelet| {
                let mut nodes_created = 0;
                let first_bit_index = 29 - 12;
                // safety: ordered_primitives have the same length as morton_primitives
                let node = unsafe {
                    emit_lbvh(
                        &primitives,
                        &morton_primitives,
                        treelet.0,
                        treelet.1,
                        &mut nodes_created,
                        ordered_primitives.as_ptr(),
                        &ordered_prims_offset,
                        first_bit_index,
                        max_prims as usize,
                    )
                };
                total_nodes.fetch_add(nodes_created, std::sync::atomic::Ordering::Relaxed);
                //treelet.root = node;
                node
            })
            .collect();
        //println!("Built treelets");
        // Use SAH or some other method to collapse nodes into a single BVH
        let mut total_nodes = total_nodes.fetch_add(0, std::sync::atomic::Ordering::Relaxed);
        let root = build_upper_tree(treelets, &mut total_nodes, &mut ordered_primitives);

        //println!("Successfully built BVH");

        Self {
            root,
            primitives: ordered_primitives,
            total_nodes,
        }
    }

    /// Flatten the BVH into a compact GPU representation
    pub fn flatten(&self) -> Vec<GpuNode> {
        let mut nodes = vec![GpuNode::new(&self.root.bbox); self.total_nodes as usize];

        fn flatten_recursive(
            nodes: &mut Vec<GpuNode>,
            cluster: &BvhBuildNode,
            offset: &mut u32,
        ) -> u32 {
            let current_offset = *offset;
            let mut linear_node = nodes[*offset as usize];
            linear_node.max = cluster.bbox.max;
            linear_node.min = cluster.bbox.min;
            let node_offset = *offset;
            *offset += 1;
            match &cluster.node_type {
                BvhBuildNodeType::Leaf {
                    num_primitives,
                    first_prim_offset,
                } => {
                    linear_node.number_of_prims = *num_primitives;
                    linear_node.offset_ptr = *first_prim_offset;
                }
                // We do not use the split right now
                BvhBuildNodeType::Interior {
                    _split: _,
                    left,
                    right,
                } => {
                    let number_of_prims = 0;
                    linear_node.number_of_prims = number_of_prims;
                    flatten_recursive(nodes, left, offset);
                    linear_node.offset_ptr = flatten_recursive(nodes, right, offset);
                }
            }

            nodes[current_offset as usize] = linear_node;
            node_offset
        }
        flatten_recursive(&mut nodes, &self.root, &mut 0);

        nodes
    }

    /// Get the primitive indices for the GPU Nodes
    pub fn triangles(&self) -> Vec<u32> {
        self.primitives.iter().map(|accobj| accobj.idx).collect()
    }
}

#[inline]
fn build_upper_tree(
    build_nodes: Vec<BvhBuildNode>,
    total_nodes: &mut u32,
    _ordered_prims: &mut [AccObj],
) -> BvhBuildNode {
    collapse_build_nodes_recursive(build_nodes, total_nodes) //, ordered_prims, &mut 0)
}

/// Split in half implementation
fn collapse_build_nodes_recursive(
    mut build_nodes: Vec<BvhBuildNode>,
    total_nodes: &mut u32, /*, ordered_prims: &mut [AccObj], ordered_prims_offset: &mut u32 */
) -> BvhBuildNode {
    // compute overall bound
    let mut bound = Bbox::new();
    for node in build_nodes.iter() {
        bound.include_bbox(&node.bbox);
    }

    // create leaf
    if build_nodes.len() == 1 {
        build_nodes.pop().unwrap()
    } else {
        // create split using center
        let mut centroid_bound = Bbox::new();
        for node in build_nodes.iter() {
            centroid_bound.include_vertex(node.bbox.center());
        }
        let dimension = centroid_bound.longest_axis();

        let (child0_nodes, child1_nodes) = mid_partition(build_nodes, dimension);

        let child0 = collapse_build_nodes_recursive(child0_nodes, total_nodes); //, ordered_prims, ordered_prims_offset);
        let child1 = collapse_build_nodes_recursive(child1_nodes, total_nodes); //, ordered_prims, ordered_prims_offset);

        *total_nodes += 1;
        BvhBuildNode::new_internal(dimension.into(), child0, child1)
    }
}

#[inline]
fn mid_partition(
    mut nodes: Vec<BvhBuildNode>,
    dimension: u32,
) -> (Vec<BvhBuildNode>, Vec<BvhBuildNode>) {
    let mid = nodes.len() / 2;
    nodes.select_nth_unstable_by(mid, |a, b| {
        f32::total_cmp(&a.bbox.center()[dimension], &b.bbox.center()[dimension])
    });
    let nodes_right = nodes.split_off(mid);

    (nodes, nodes_right)
}

/// Node type
#[derive(Debug, Clone)]
pub struct BvhBuildNode {
    pub bbox: Bbox,
    node_type: BvhBuildNodeType,
}

/// BVH Nodes are either a leaf or internal
#[derive(Debug, Clone)]
enum BvhBuildNodeType {
    /// We are managing that primitives belonging to a
    /// leaf node are adjacent externally
    Leaf {
        num_primitives: u32,
        first_prim_offset: u32,
    },
    /// Interior nodes have ownership over child nodes
    Interior {
        _split: Split,
        left: Box<BvhBuildNode>,
        right: Box<BvhBuildNode>,
    },
}

impl BvhBuildNode {
    /// Create a new leaf nodes
    fn new_leaf(first_prim_offset: u32, num_primitives: u32, bbox: Bbox) -> Self {
        Self {
            bbox,
            node_type: BvhBuildNodeType::Leaf {
                first_prim_offset,
                num_primitives,
            },
        }
    }

    /// Create a new internal node
    fn new_internal(axis: Split, child0: BvhBuildNode, child1: BvhBuildNode) -> Self {
        let mut bbox = child0.bbox;
        bbox.include_bbox(&child1.bbox);
        Self {
            bbox,
            //num_primitives: child0.num_primitives + child1.num_primitives,
            node_type: BvhBuildNodeType::Interior {
                _split: axis,
                left: Box::new(child0),
                right: Box::new(child1),
            },
        }
    }
}

/// Safety:
/// ordered_primitives should have the same length as morton_primitives
unsafe fn emit_lbvh(
    primitives: &[AccObj],
    morton_primitives: &[MortonPrimitive],
    morton_offset: usize,
    num_primitives: usize,
    total_nodes: &mut u32,
    ordered_primitives: *const AccObj,
    ordered_prims_offset: &AtomicUsize,
    bit_index: i32,
    max_prims_in_node: usize,
) -> BvhBuildNode {
    // We cannot go further down or have few enough primitives to create a leaf
    if bit_index <= -1 || num_primitives < max_prims_in_node {
        let mut bbox = Bbox::new();
        // will need atomics here
        let first_prim_offset =
            ordered_prims_offset.fetch_add(num_primitives, std::sync::atomic::Ordering::Relaxed);

        for i in 0..num_primitives {
            let primitive_index = morton_primitives[morton_offset + i].index;
            // Safety: since ordered_prims_offset is atomic, the first_prim_offset value we receive is unique to this thread
            // and the next thread to peek into the value will receive first_prim_offset + num_primitives
            // this means that [first_prim_offset..first_prim_offset+num_primitives] range is unaliased and is only written
            // by this thread
            // the caller is responsible for ensuring that we do not overrun this pointer but ordered_primitives should have the
            // length as morton_primitives, which means we won't overrun the buffer
            unsafe {
                *ordered_primitives.cast_mut().add(first_prim_offset + i) =
                    primitives[primitive_index as usize];
            }
            bbox.include_bbox(&primitives[primitive_index as usize].bbox);
        }

        *total_nodes += 1;
        BvhBuildNode::new_leaf(first_prim_offset as u32, num_primitives as u32, bbox)
    } else {
        let mask = 1 << bit_index;
        if (morton_primitives[morton_offset].morton_code & mask)
            == (morton_primitives[morton_offset + num_primitives - 1].morton_code & mask)
        {
            // same call with bit index dropped by 1
            return emit_lbvh(
                primitives,
                morton_primitives,
                morton_offset,
                num_primitives,
                total_nodes,
                ordered_primitives,
                ordered_prims_offset,
                bit_index - 1,
                max_prims_in_node,
            );
        }

        // find LBVH split using binary search
        let pred = |i: usize| {
            (morton_primitives[morton_offset].morton_code & mask)
                == (morton_primitives[morton_offset + i].morton_code & mask)
        };

        let mut size_maybe = num_primitives.checked_sub(2);
        let mut first = 1;
        while size_maybe.is_some_and(|size| size > 0) {
            let size = size_maybe.unwrap();
            let half = size >> 1;
            let middle = first + half;
            let result = pred(middle);
            first = if result { middle + 1 } else { first };
            size_maybe = if result {
                size.checked_sub(half + 1)
            } else {
                Some(half)
            };
        }
        let offset = usize::clamp(first, 0, num_primitives.checked_sub(2).unwrap_or(0));
        let new_morton_offset = morton_offset + offset;

        // return interior LBVH node
        let left = emit_lbvh(
            primitives,
            morton_primitives,
            morton_offset,
            offset,
            total_nodes,
            ordered_primitives,
            ordered_prims_offset,
            bit_index - 1,
            max_prims_in_node,
        );
        let right = emit_lbvh(
            primitives,
            morton_primitives,
            new_morton_offset,
            num_primitives - offset,
            total_nodes,
            ordered_primitives,
            ordered_prims_offset,
            bit_index - 1,
            max_prims_in_node,
        );
        let axis = (bit_index % 3) as u32;

        *total_nodes += 1;
        BvhBuildNode::new_internal(axis.into(), left, right)
    }
}

/// Morton primitive just wraps an index with a morton code
#[derive(Copy, Clone, Debug)]
struct MortonPrimitive {
    pub index: u32,
    pub morton_code: u32, // use 30 bits
}

impl MortonPrimitive {
    fn new() -> MortonPrimitive {
        MortonPrimitive {
            index: 0,
            morton_code: 0,
        }
    }
}

/// Allow radix_sort
impl RadixKey for MortonPrimitive {
    const LEVELS: usize = 4;

    #[inline]
    fn get_level(&self, level: usize) -> u8 {
        (self.morton_code >> (level * 8)) as u8
    }
}

/// Also provide implementations for using nlogn sorting algorithms
/// since the radix_sort_unstable() function panics on Debug builds

impl Eq for MortonPrimitive {}

impl Ord for MortonPrimitive {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.morton_code.cmp(&other.morton_code)
    }
}

impl PartialEq for MortonPrimitive {
    fn eq(&self, other: &Self) -> bool {
        self.morton_code == other.morton_code
    }
}

impl PartialOrd for MortonPrimitive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.morton_code.cmp(&other.morton_code))
    }
}

/// From the PBR Book vol. 4
/// https://www.pbr-book.org/4ed/Utilities/Mathematical_Infrastructure#x7-MortonIndexing
/// Take a 10-bit number and tile it as follows:
/// xyzw -> --x--y--z--w
#[inline]
fn left_shift_3(mut x: u32) -> u32 {
    if x == 1 << 10 {
        x -= 1;
    }
    x = (x | (x << 16)) & 0b00000011000000000000000011111111;
    x = (x | (x << 8)) & 0b00000011000000001111000000001111;
    x = (x | (x << 4)) & 0b00000011000011000011000011000011;
    x = (x | (x << 2)) & 0b00001001001001001001001001001001;
    x
}

#[inline]
fn encode_morton_3(x: f32, y: f32, z: f32) -> u32 {
    (left_shift_3(z as u32) << 2) | (left_shift_3(y as u32) << 1) | left_shift_3(x as u32)
}

/// GPU Node
///

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct GpuNode {
    pub min: Vec3f32,
    pub offset_ptr: u32,
    pub max: Vec3f32,
    pub number_of_prims: u32,
}

impl GpuNode {
    pub fn new(bbox: &Bbox) -> Self {
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
        let bvh = Bvh::new(&model, 4);
        let _flattened = bvh.flatten();
        //println!("{:#?}", bvh);
        //println!("{:#?}", flattened);
    }

    #[test]
    fn bvh_new2() {
        let model = Mesh::from_obj("res/models/CornellBox.obj").expect("Failed to load model");
        let bvh = Bvh::new(&model, 4);
        let _flattened = bvh.flatten();
    }

    #[test]
    fn bvh_new3() {
        let model = Mesh::from_obj("res/models/teapot.obj").expect("Failed to load model");
        let bvh = Bvh::new(&model, 4);
        let _flattened = bvh.flatten();
    }

    #[test]
    fn bvh_new4() {
        let model = Mesh::from_obj("res/models/bunny.obj").expect("Failed to load model");
        let bvh = Bvh::new(&model, 4);
        let _flattened = bvh.flatten();
    }

    #[test]
    fn bvh_new5() {
        let model = Mesh::from_obj("res/models/dragon.obj").expect("Failed to load model");
        let start = std::time::Instant::now();
        let bvh = Bvh::new(&model, 4);
        let _flattened = bvh.flatten();
        let passed = start.elapsed();
        println!("built BVH in {} ms", passed.as_micros() as f64 / 1000.0);
    }
}
