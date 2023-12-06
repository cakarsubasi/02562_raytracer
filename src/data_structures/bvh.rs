use super::{bbox::Bbox, vector::Vec3f32};



pub struct Bvh {

}

struct Cluster {

}

enum ClusterType {
    Leaf {
        objects: Vec<Bbox>,
    },
    Interior {
        left: Cluster,
        right: Cluster,
    },
}

#[repr(C, align(16))]

struct GpuNode {
    min: Vec3f32,
    tri: u32,
    max: Vec3f32,
    right: u32,
}

static_assertions::assert_eq_size!(GpuNode, [u32; 8]);