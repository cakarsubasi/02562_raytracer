const MAX_LEVEL = 50u;
const F32_MAX = 1e27;

//var<storage> bvh_nodes: array<BvhNode>;
//@group(0) @binding(2)
//var<storage> bvh_triangles: array<u32>;

struct BvhNode {
    bbox_min: vec3f,
    offset_ptr: u32, // offset pointer
    bbox_max: vec3f,
    n_primitives: u32, // number of primitives
};

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

var<private> node_stack: array<u32, MAX_LEVEL>;

fn intersect_bvh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    let ray_dir_inv = 1.0 / (*r).direction;
    let ray_orig = (*r).origin;
    var t_max = F32_MAX;
    var to_visit = 0u;
    var current_node_index = 0u;
    var found = false;
    node_stack[0] = 0;
    while (true) {
        current_node_index = node_stack[to_visit];
        let current_node = bvh_nodes[current_node_index];
        if (intersect_bb(ray_dir_inv, ray_orig, current_node)) {
            // leaf node
            if (current_node.n_primitives > 0) {
                let offset = current_node.offset_ptr;
                for (var i = 0; i < current_node.n_primitives; i++) {
                    // get triangle
                    let obj_idx = bvh_triangles[offset+i];
                    // check intersection
                    if(intersect_triangle_indexed(r, hit, obj_idx)) {
                        (*r).tmax = (*hit).dist;
                        found = true;
                    }
                }
                if (to_visit == 0u) {
                    break;
                }
                to_visit--;
                //current_node_index = node_stack[to_visit];
            // internal node
            } else {
                node_stack[to_visit] = current_node_index + 1;
                to_visit++;
                node_stack[to_visit] = current_node.offset_ptr;
            }
        } else {
            if (to_visit == 0u) {
                break;
            }
            to_visit--;
            //current_node_index = node_stack[to_visit];
        }
        break;
        
    }

    return found;
}

fn intersect_trimesh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    return intersect_bvh(r, hit);
}