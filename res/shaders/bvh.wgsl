//@group(3) @binding(1) var<storage> treeIds: array<u32>;
//@group(3) @binding(2) var<storage> bspTree: array<vec4u>;
//@group(3) @binding(3) var<storage> bspPlanes: array<f32>;

const MAX_LEVEL = 20u;
const BSP_LEAF = 3u;
var<private> branch_node: array<vec2u, MAX_LEVEL>;
var<private> branch_ray: array<vec2f, MAX_LEVEL>;

struct BvhNode {
    bbox_min: vec3f,
    offset_ptr: u32, // offset pointer, 0 if leaf
    bbox_max: vec3f,
    triangle: u32, // triangle id, offset_ptr must be zero for a meaningful value
};


fn intersect_bb(r: ptr<function, Ray>, bbox: BvhNode) -> bool {
    return false;
}

fn intersect_bvh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {

    return false;
}