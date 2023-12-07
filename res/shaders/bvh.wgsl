const MAX_LEVEL = 50u;
const F32_MAX = 1e27;

struct BvhNode {
    bbox_min: vec3f,
    offset_ptr: u32, // offset pointer
    bbox_max: vec3f,
    n_primitives: u32, // number of primitives
};

var<private> node_stack: array<u32, MAX_LEVEL>;

fn intersect_bb(ray_dir_inv: vec3f, ray_orig: vec3f, bbox: BvhNode) -> bool {
    let t0 = vec3f(0.0);
    let t1 = vec3f(F32_MAX);

    let near = (bbox.bbox_min - ray_orig) * ray_dir_inv;
    let far  = (bbox.bbox_max - ray_orig) * ray_dir_inv;

    let near1 = select(near, far, near > far);
    let far1 = select(far, near, near > far);

    let t0_test: vec3<bool> = near1 > t0;
    let t1_test: vec3<bool> = far1 < t1;

    let t0_res = near * vec3f(t0_test);
    let t1_res = far * vec3f(t1_test);
    if ( any(t0_res > t1_res) ) {
        return false;
    }

    return true;
}

fn intersect_bvh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    let ray_dir_inv = 1.0 / (*r).direction;
    let ray_orig = (*r).origin;
    var to_visit = 0u;
    var current_node_index = 0u;
    while (true) {
        let current_node = bvh_nodes[current_node_index];
        //if (intersect)
        break;
    }


    return false;
}

fn intersect_trimesh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    return intersect_bvh(r, hit);
}