const MAX_LEVEL = 50u;

struct BvhNode {
    bbox_min: vec3f,
    offset_ptr: u32, // offset pointer, 0 if leaf
    bbox_max: vec3f,
    triangle: u32, // triangle id, offset_ptr must be zero for a meaningful value
};

var<private> node_stack: array<BvhNode, MAX_LEVEL>;

fn intersect_bb(r: ptr<function, Ray>, bbox: BvhNode) -> bool {
    return false;
}

fn intersect_bvh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {

    return false;
}