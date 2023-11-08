//@group(3) @binding(1) var<storage> treeIds: array<u32>;
//@group(3) @binding(2) var<storage> bspTree: array<vec4u>;
//@group(3) @binding(3) var<storage> bspPlanes: array<f32>;

const MAX_LEVEL = 20u;
const BSP_LEAF = 3u;
var<private> branch_node: array<vec2u, MAX_LEVEL>;
var<private> branch_ray: array<vec2f, MAX_LEVEL>;

fn intersect_trimesh(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool
{
    var branch_lvl: u32 = 0u;
    var near_node: u32 = 0u;
    var far_node: u32 = 0u;
    var t: f32 = 0.0;
    var node: u32 = 0u;

    for (var i = 0u; i <= MAX_LEVEL; i++) {
        let tree_node: vec4u = bspTree[node];
        let node_axis_leaf = tree_node.x&3u;

        if (node_axis_leaf == BSP_LEAF) {
            // A leaf was found
            let node_count = tree_node.x>>2u;
            let node_id = tree_node.y;
            var found = false;
            
            for (var j = 0u; j < node_count; j++) {
                let obj_idx = treeIds[node_id + j];

                if (intersect_triangle_indexed(r, hit, obj_idx)) {
                    (*r).tmax = (*hit).dist;
                    found = true;
                }
            }

            if (found) {
                return true;
            } else if (branch_lvl == 0u) {
                return false;
            } else {
                branch_lvl--;
                i = branch_node[branch_lvl].x;
                node = branch_node[branch_lvl].y;
                (*r).tmin = branch_ray[branch_lvl].x;
                (*r).tmax = branch_ray[branch_lvl].y;
                continue;
            }
        }

        let axis_direction = (*r).direction[node_axis_leaf];
        let axis_origin = (*r).origin[node_axis_leaf];

        if (axis_direction >= 0.0f) {
            near_node = tree_node.z; // left
            far_node = tree_node.w; // right
        } else {
            near_node = tree_node.w; // right
            far_node = tree_node.z; // left
        }

        let node_plane = bspPlanes[node];
        let denom = select(axis_direction, 1.0e-8f, abs(axis_direction) < 1.0e-8f);
        t = (node_plane - axis_origin) / denom;

        if(t > (*r).tmax) { 
            node = near_node; 
        } else if (t < (*r).tmin) { 
            node = far_node; 
        } else {
            branch_node[branch_lvl].x = i;
            branch_node[branch_lvl].y = far_node;
            branch_ray[branch_lvl].x = t;
            branch_ray[branch_lvl].y = (*r).tmax;
            branch_lvl++;
            (*r).tmax = t;
            node = near_node;
        }
    }
    return false;
}

fn intersect_trimesh_immediate_return(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool
{
    var branch_lvl: u32 = 0u;
    var near_node: u32 = 0u;
    var far_node: u32 = 0u;
    var t: f32 = 0.0;
    var node: u32 = 0u;

    for (var i = 0u; i <= MAX_LEVEL; i++) {
        let tree_node: vec4u = bspTree[node];
        let node_axis_leaf = tree_node.x&3u;

        if (node_axis_leaf == BSP_LEAF) {
            // A leaf was found
            let node_count = tree_node.x>>2u;
            let node_id = tree_node.y;
            var found = false;
            
            for (var j = 0u; j < node_count; j++) {
                let obj_idx = treeIds[node_id + j];

                if (intersect_triangle_indexed(r, hit, obj_idx)) {
                    (*r).tmax = (*hit).dist;
                    return true;
                    //found = true;
                }
            }

            if (found) {
                return true;
            } else if (branch_lvl == 0u) {
                return false;
            } else {
                branch_lvl--;
                i = branch_node[branch_lvl].x;
                node = branch_node[branch_lvl].y;
                (*r).tmin = branch_ray[branch_lvl].x;
                (*r).tmax = branch_ray[branch_lvl].y;
                continue;
            }
        }

        let axis_direction = (*r).direction[node_axis_leaf];
        let axis_origin = (*r).origin[node_axis_leaf];

        if (axis_direction >= 0.0f) {
            near_node = tree_node.z; // left
            far_node = tree_node.w; // right
        } else {
            near_node = tree_node.w; // right
            far_node = tree_node.z; // left
        }

        let node_plane = bspPlanes[node];
        let denom = select(axis_direction, 1.0e-8f, abs(axis_direction) < 1.0e-8f);
        t = (node_plane - axis_origin) / denom;

        if(t > (*r).tmax) { 
            node = near_node; 
        } else if (t < (*r).tmin) { 
            node = far_node; 
        } else {
            branch_node[branch_lvl].x = i;
            branch_node[branch_lvl].y = far_node;
            branch_ray[branch_lvl].x = t;
            branch_ray[branch_lvl].y = (*r).tmax;
            branch_lvl++;
            (*r).tmax = t;
            node = near_node;
        }
    }
    return false;
}