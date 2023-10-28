// Vertex shader

const PI = 3.14159265359;
const ETA = 0.000001;

const BACKGROUND_COLOR: vec3f = vec3f(0.0, 0.0, 0.5);

alias ShaderType = u32;
const SHADER_TYPE_LAMBERTIAN: u32 = 0u;
const SHADER_TYPE_PHONG: u32 = 1u;
const SHADER_TYPE_MIRROR: u32 = 2u;
const SHADER_TYPE_TRANSMIT: u32 = 3u;
const SHADER_TYPE_GLOSSY: u32 = 4u;
const SHADER_TYPE_NORMAL: u32 = 5u;
const SHADER_TYPE_BASECOLOR: u32 = 6u;
const SHADER_TYPE_NO_RENDER: u32 = 255u;
const SHADER_TYPE_DEFAULT: u32 = 0u;

const MAX_DEPTH: i32 = 10;

//@group(0) @binding(1)
//var<uniform> selection: u32;
// Stratified jitter sampling array TODO
@group(0) @binding(2)
var<storage> jitter: array<vec2f>;

@group(1) @binding(0)
var sampler0: sampler;
@group(1) @binding(1)
var texture0: texture_2d<f32>;

// GPU will always align to 16, so this does not waste space
@group(2) @binding(0)
var<storage> vertexBuffer: array<vec4f>;
// GPU will always align to 16, so this does not waste space
@group(2) @binding(1)
var<storage> indexBuffer: array<vec4u>;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>,
};

struct Ray {
    direction: vec3f,
    origin: vec3f,
    tmax: f32,
    tmin: f32,
};

fn ray_init(direction: vec3f, origin: vec3f) -> Ray {
    return Ray(
        direction,
        origin,
        5000.0,
        ETA,
    );
}

fn ray_at(r: Ray, dist: f32) -> vec3f {
    return r.origin + r.direction * dist;
}

struct Camera {
    origin: vec3f,
    direction: vec3f,
    up: vec3f,
    constant: f32,
};

struct Light {
    l_i: vec3f, // intensity
    w_i: vec3f, // incidence
    dist: f32, // distance
};

fn light_init() -> Light {
    return Light(
        vec3f(0.0),
        vec3f(0.0),
        999999.0,
    );
}

struct HitRecord {
    has_hit: bool,
    depth: i32,
    dist: f32,
    position: vec3f,
    normal: vec3f,
    // color contribution
    ambient: vec3f,
    diffuse: vec3f,
    // shader properties
    shader: ShaderType,
    use_texture: bool, //
    uv0: vec2f,
    base_color: vec3f,
    ior1_over_ior2: f32,
    specular: f32,
    shininess: f32,
};

fn hit_record_init() -> HitRecord {
    return HitRecord(
        false,
        0,
        0.0, 
        vec3f(0.0), 
        vec3f(0.0),
        // color contribution
        vec3f(0.0),
        vec3f(0.0),
        // shader properties
        SHADER_TYPE_NO_RENDER,
        false,
        vec2f(0.0),
        vec3f(0.0),
        1.0,
        0.0,
        0.0,
    );
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.coords = vec2f(model.position.x, model.position.y);
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

fn get_camera_ray(uv: vec2f) -> Ray {
    let e = uniforms.camera_pos;
    let p = uniforms.camera_look_at;
    let u = uniforms.camera_up;
    let v = normalize(p - e);
    let d = uniforms.camera_constant;
    let aspect = uniforms.aspect_ratio;

    let b1 = normalize(cross(v, u));
    let b2 = cross(b1, v);

    let q = normalize(b1 * uv.x * aspect + b2 * uv.y + v*d);

    let ray = ray_init(q, e);
    return ray;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let bgcolor = vec4f(0.1, 0.3, 0.6, 1.0);
    let max_depth = MAX_DEPTH;
    let uv = in.coords * 0.5;
    var r = get_camera_ray(uv);
    var hit = hit_record_init();

    var result = vec3f(0.0);
    var result_textured = vec3f(1.0);
    // each loop is one bounce
//    if (intersect_trimesh(&r, &hit)) {
//        return vec4f(error_shader(), bgcolor.a);
//    } else {
//        return bgcolor;
//    }
    if (!intersect_min_max(&r)) {
        result = bgcolor.rgb;
    } else {
        for (var i = 0; i < max_depth; i++) {
            if (intersect_scene_bsp(&r, &hit)) {
                result += shade(&r, &hit);
                if (hit.use_texture) {
                    result_textured = texture_sample(&hit);
                }
            } else {
                result += bgcolor.rgb; break;
            }

            if (hit.has_hit) {
                break;
            }
        }
        result = result * result_textured;
    }
    //return vec4f(result, bgcolor.a);
    return vec4f(pow(result, vec3f(1.0/1.0)), bgcolor.a);
}

fn texture_sample(hit: ptr<function, HitRecord>) -> vec3f {
    // Note that we are ignoring the potential alpha channel within the texture here
    // TODO: Might want to multiply alpha here
    return textureSample(texture0, sampler0, (*hit).uv0).xyz;
}

fn intersect_scene_bsp(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    let has_hit = intersect_trimesh(r, hit);
    return has_hit;
}

fn intersect_scene(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    var has_hit = false;
    let num_of_tris = arrayLength(&indexBuffer);
    for (var i = 0u; i < num_of_tris; i++) {
        has_hit = has_hit || intersect_triangle_indexed(r, hit, i);
    }

    //let arr = array<vec3f, 3>(vec3f(-0.2, 0.1, 0.9), vec3f(0.2, 0.1, 0.9), vec3f(-0.2, 0.1, -0.1));
    //has_hit = has_hit || intersect_triangle(r, hit, arr);
    //has_hit = has_hit || intersect_sphere(r, hit, arr[0], 0.05);
    //has_hit = has_hit || intersect_sphere(r, hit, arr[1], 0.05);
    //has_hit = has_hit || intersect_sphere(r, hit, arr[2], 0.05);
//
    //let arr2 = array<vec3f, 3>(vec3f(0.1, 0.1, 0.1), vec3f(0.1, 0.5, 0.1), vec3f(0.6, 0.5, 0.1));
    //has_hit = has_hit || intersect_triangle(r, hit, arr2);
    //has_hit = has_hit || intersect_sphere(r, hit, arr2[0], 0.05);
    //has_hit = has_hit || intersect_sphere(r, hit, arr2[1], 0.05);
    //has_hit = has_hit || intersect_sphere(r, hit, arr2[2], 0.05);

    //has_hit = has_hit || intersect_plane(r, hit, vec3f(0.0, 0.0, 0.0), vec3f(0.0, 1.0, 0.0));
    //has_hit = has_hit || intersect_sphere(r, hit, vec3f(0.0, 0.5, 0.0), 0.3);
    return has_hit;
}

fn intersect_plane(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, position: vec3f, normal: vec3f) -> bool {
    let ray = *r;
    let distance = dot((position - ray.origin), normal)/(dot(ray.direction, normal));
    
    if (distance < ray.tmin || distance > ray.tmax) {
        return false;
    }
    (*r).tmax = distance;
    (*hit).dist = distance;
    let pos = ray_at(ray, distance);
    (*hit).position = pos;
    (*hit).normal = normal;
    (*hit).base_color = vec3f(0.1, 0.7, 0.0);
    (*hit).shader = SHADER_TYPE_LAMBERTIAN;
    (*hit).use_texture = true;
    (*hit).uv0 = pos.xy;
    return true;
}

fn intersect_triangle(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, v: array<vec3f, 3>) -> bool {
    let ray = *r;
    let w_i = ray.direction;
    let o = ray.origin;

    let e0 = v[1] - v[0];
    let e1 = v[2] - v[0];
    let o_to_v0 = v[0] - o;
    let normal = cross(e0, e1);

    let nom = cross(o_to_v0, w_i);
    let denom = dot(w_i, normal);
    if (abs(denom) < 1e-10) {
        return false;
    }

    var eta = ETA;
    if (denom > 0.0) {
        eta = eta * -1.0;
    }

    let beta = dot(nom, e1) / denom;
    if (beta < 0.0) {
        return false;
    }
    let gamma = -dot(nom, e0) / denom;
    if (gamma < 0.0) {
        return false;
    }
    if (beta + gamma > 1.0) {
        return false;
    }

    let distance = dot(o_to_v0, normal) / denom;
    if (distance > ray.tmax || distance < ray.tmin) {
        return false;
    }

    (*r).tmax = distance;
    (*hit).dist = distance;
    let pos = ray_at(ray, distance);
    (*hit).position = pos;
    (*hit).normal = normalize(normal);
    (*hit).base_color = vec3f(0.4, 0.3, 0.2);
    (*hit).shader = SHADER_TYPE_LAMBERTIAN;
    return true;
}

fn intersect_triangle_indexed(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, v: u32) -> bool {
    let v1 = indexBuffer[v].x;
    let v2 = indexBuffer[v].y;
    let v3 = indexBuffer[v].z;
    let arr = array<vec3f, 3>(vertexBuffer[v1].xyz, vertexBuffer[v2].xyz, vertexBuffer[v3].xyz);
    return intersect_triangle(r, hit, arr);
}

fn intersect_sphere(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, center: vec3f, radius: f32) -> bool {
    let ray = *r;
    let oc = ray.origin - center;
    let a = dot(ray.direction, ray.direction);
    let b_over_2 = dot(oc, ray.direction);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b_over_2 * b_over_2 - a * c;
    if (discriminant < 0.0) {
        return false;
    }
    let disc_sqrt = sqrt(discriminant);
    var root = (- b_over_2 - disc_sqrt) / a;
    if (root < ray.tmin || root > ray.tmax) {
        root = (- b_over_2 + disc_sqrt) / a;
        if (root < ray.tmin || root > ray.tmax) {
            return false;
        }
    }

    (*r).tmax = root;
    (*hit).dist = root;
    let pos = ray_at(ray, root);
    let normal = normalize(pos - center);
    (*hit).position = pos;
    (*hit).normal = normal;
    (*hit).base_color = vec3f(0.0, 0.5, 0.0);

    let shader_type = SHADER_TYPE_PHONG;
    if (shader_type == SHADER_TYPE_TRANSMIT) {
        setup_shader_transmissive(r, hit, 1.4);
    } else if (shader_type == SHADER_TYPE_GLOSSY) {
        setup_shader_transmissive(r, hit, 1.4);
        (*hit).specular = 0.2;
        (*hit).shininess = 42.0;
    }
    (*hit).shader = shader_type;
    return true;
}

fn setup_shader_transmissive(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, material_ior: f32) {
    (*hit).shader = SHADER_TYPE_TRANSMIT;
    if (dot((*hit).normal, (*r).direction) < 0.0) {
        (*hit).ior1_over_ior2 = 1.0 / material_ior;
    } else {
        (*hit).ior1_over_ior2 = material_ior / 1.0;
    }
}

fn sample_point_light(pos: vec3f) -> Light {
    let light_pos = vec3f(0.0, 1.2, 0.0);
    let light_intensity = 5.0 * vec3f(PI, PI, PI);
    var light = light_init();
    
    let dir = light_pos - pos;
    let dist = dot(dir, dir);

    light.dist = dist;
    light.l_i = light_intensity / (dist * dist);
    light.w_i = dir;

    return light;
}

fn shade(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    var hit_record = *hit;
    var color = vec3f(0.0, 0.0, 0.0);
    hit_record.has_hit = true;
    hit_record.depth += 1;
    *hit = hit_record;

    switch(hit_record.shader) {
        case 0u: {
            color = lambertian(r, hit);
        }
        case 1u: {
            color = phong(r, hit);
        }
        case 2u: {
            color = mirror(r, hit);
        }
        case 3u: {
            color = transmit(r, hit);
        }
        case 4u: {
            color = glossy(r, hit);
        }
        case 5u: {
            color = shade_normal(r, hit);
        }
        case 6u: {
            color = shade_base_color(r, hit);
        }
        default: {
            color = error_shader();
        }
    }
    return color;
}

fn lambertian(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    var hit_record = *hit;
    let light = sample_point_light(hit_record.position);
    let normal = hit_record.normal;

    let ray_dir = light.w_i;
    let ray_orig = hit_record.position + hit_record.normal * ETA;
    var ray = ray_init(ray_dir, ray_orig);

    let blocked = intersect_scene(&ray, hit);
    let ambient = hit_record.base_color;
    var diffuse = hit_record.base_color * light_diffuse_contribution(light, normal, hit_record.specular);

    // ambient only
    if (blocked) {
        return ambient * 0.1;
    } else { // ambient and diffuse
        return diffuse_and_ambient(diffuse, ambient);
    }

    return diffuse_and_ambient(diffuse, ambient);
}

fn light_diffuse_contribution(light: Light, normal: vec3f, specular: f32) -> vec3f {
    let one_minus_specular = 1.0 - specular;
    var diffuse = vec3f(dot(normal, light.w_i));
    diffuse *= light.l_i;
    diffuse *= one_minus_specular / PI;
    return diffuse;
}

fn diffuse_and_ambient(diffuse: vec3f, ambient: vec3f) -> vec3f {
    return 0.9 * diffuse + 0.1 * ambient;
} 

fn phong(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    let hit_record = *hit;
    let ray = *r;

    let specular = hit_record.specular;
    let o_m_s = 1.0 - hit_record.specular;
    let s = hit_record.shininess;
    let normal = hit_record.normal;

    let w_i = ray.direction;
    let w_o = normalize(uniforms.camera_pos - hit_record.position);

    let light = sample_point_light(hit_record.position);
    let light_dir = light.w_i;
    let refl_dir = reflect(-light_dir, normal);

    let w_r = reflect(-w_i, normal);
    let refl = light.l_i * dot(ray.direction, hit_record.normal);
    let fixed = specular * (s + 2.0) / (2.0 * PI) * pow(dot(w_o, w_r), 5.0);
    //let phong_total = light.l_i * fixed * dot(-w_i, normal);
    //let phong_coeff = o_m_s / PI + specular * (s + 2.0) / (2.0 * PI) * pow(dot(w_o, w_r), 100f);
    let phong_total = vec3f(1.0) * pow(dot(w_o, refl_dir), 10.0);
    return phong_total; // * vec3f(1.0);
}


fn mirror(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    var hit_record = *hit;
    
    let normal = hit_record.normal;
    let ray_dir = reflect((*r).direction, normal);
    let ray_orig = hit_record.position + normal * ETA;
    *r = ray_init(ray_dir, ray_orig);

    hit_record.has_hit = false;
    hit_record.shader = SHADER_TYPE_NO_RENDER;

    *hit = hit_record;

    return vec3f(0.0, 0.0, 0.0);

}

fn glossy(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    return phong(r, hit) + transmit(r, hit);
    //return phong(r, hit);
}

fn transmit(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    var hit_record = *hit;
    let ray = *r;
    let w_i = -normalize(ray.direction);
    var normal = normalize(hit_record.normal);
    let ior = hit_record.ior1_over_ior2;

    if (dot(w_i, normal) < 0.0) {
        normal = -normal;
    }

    let cos_thet_i = dot(w_i, normal);
    let cos_thet_t_2 = (1.0 - (ior*ior) * (1.0 - cos_thet_i * cos_thet_i));
    if (cos_thet_t_2 < 0.0) {
        return error_shader();
        //return mirror(r, hit);
    }
    //let sin_thet_i = sqrt(1.0 - cos_thet_i * cos_thet_i);
    let tangent = ((normal * cos_thet_i - w_i));
    
    let w_t = ior * tangent - (normal * sqrt(cos_thet_t_2));
    let orig = hit_record.position + w_t * ETA;

    *r = ray_init(w_t, orig); 

    hit_record.has_hit = false;
    hit_record.shader = SHADER_TYPE_NO_RENDER;

    *hit = hit_record;
    return vec3f(0.0, 0.0, 0.0);
}

fn shade_normal(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    return ((*hit).normal + 1.0) * 0.5;
}

fn shade_base_color(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    return (*hit).base_color;
}

fn error_shader() -> vec3f {
    return vec3f(0.7, 0.0, 0.7);
}


struct Aabb {
    min: vec3f,
    _padding: f32,
    max: vec3f,
    _padding2: f32,
};

@group(3) @binding(0) var<uniform> aabb: Aabb;

fn intersect_min_max(r: ptr<function, Ray>) -> bool
{
    let p1 = (aabb.min - (*r).origin)/(*r).direction;
    let p2 = (aabb.max - (*r).origin)/(*r).direction;
    let pmin = min(p1, p2);
    let pmax = max(p1, p2);
    let tmin = max(pmin.x, max(pmin.y, pmin.z));
    let tmax = min(pmax.x, min(pmax.y, pmax.z));
    if (tmin > tmax || tmin > (*r).tmax || tmax < (*r).tmin) {
          return false;
    }
    (*r).tmin = max(tmin - 1.0e-4f, (*r).tmin);
    (*r).tmax = min(tmax + 1.0e-4f, (*r).tmax);
    return true;
}

 @group(3) @binding(1) var<storage> treeIds: array<u32>;
 @group(3) @binding(2) var<storage> bspTree: array<vec4u>;
 @group(3) @binding(3) var<storage> bspPlanes: array<f32>;

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
