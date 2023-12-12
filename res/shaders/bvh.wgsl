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

fn intersect_bb2(ray_dir_inv: vec3f, ray_orig: vec3f, bbox: BvhNode) -> bool {
    var t0 = 0.0;
    var t1 = F32_MAX;

    let near = (bbox.bbox_min - ray_orig) * ray_dir_inv;
    let far  = (bbox.bbox_max - ray_orig) * ray_dir_inv;
    // y
    var tNear = near.y;
    var tFar = far.y;
    if (tNear > tFar) {
        let temp = tNear;
        tNear = tFar;
        tFar = temp;
    }

    if (tNear > t0) {
        t0 = tNear;
    }
    if (tFar < t1) {
        t1 = tFar;
    }

    if (t0 > t1) {
        return false;
    }

    // x
    tNear = near.x;
    tFar = far.x;
    if (tNear > tFar) {
        let temp = tNear;
        tNear = tFar;
        tFar = temp;
    }

    if (tNear > t0) {
        t0 = tNear;
    }
    if (tFar < t1) {
        t1 = tFar;
    }

    if (t0 > t1) {
        return false;
    }
    
    // z
    tNear = near.z;
    tFar = far.z;
    if (tNear > tFar) {
        let temp = tNear;
        tNear = tFar;
        tFar = temp;
    }

    if (tNear > t0) {
        t0 = tNear;
    }
    if (tFar < t1) {
        t1 = tFar;
    }

    if (t0 > t1) {
        return false;
    }

    return true;
}


var<private> node_stack: array<u32, MAX_LEVEL>;
var<private> node_stack_top: u32;

fn stack_init() {
    node_stack_top = 0u;
}
fn stack_push_node(value: u32) {
    node_stack[node_stack_top] = value;
    node_stack_top += 1u;
}
fn stack_pop_node() -> u32 {
    if (node_stack_top == 0u) {
        // crash the GPU
        while (true) {

        }
    }
    node_stack_top -= 1u;
    return node_stack[node_stack_top];
}

fn stack_is_empty() -> bool {
    return (node_stack_top == 0u);
}

fn intersect_bvh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    let ray_dir_inv = 1.0 / (*r).direction;
    let ray_orig = (*r).origin;
    var t_max = F32_MAX;
    stack_init();
    var current_node_index = 0u;
    var found = false;
    stack_push_node(0u);
    for (var depth = 0u; depth < 1000u; depth++) {
        if (stack_is_empty()) {
            break;
        }
        current_node_index = stack_pop_node();
        let current_node = bvh_nodes[current_node_index];
        if (intersect_bb2(ray_dir_inv, ray_orig, current_node)) {
            // leaf node
            if (current_node.n_primitives > 0u) {
                let offset = current_node.offset_ptr;
                for (var i = 0u; i < current_node.n_primitives; i++) {
                    // get triangle
                    let obj_idx = bvh_triangles[offset+i];
                    // check intersection
                    if(intersect_triangle_indexed(r, hit, obj_idx)) {
                        (*r).tmax = (*hit).dist;
                        found = true;
                    }
                }
                
            // internal node
            } else {
                // TODO: Can store distance information to skip one of these nodes
                stack_push_node(current_node_index + 1u);
                // TODO: I HAVE TO SUBSTRACT 1 HERE BECAUSE I DID NOT
                // SET THE OFFSET_PTR CORRECTLY IN THE REFERENCE IMPL
                // GOTTA BE CAREFUL OF THAT
                stack_push_node(current_node.offset_ptr - 1u);
            }
        }
    }

    return found;
}

fn intersect_trimesh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    return intersect_bvh(r, hit);
}