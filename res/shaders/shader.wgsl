// Vertex shader

struct Uniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3f,
    camera_constant: f32,
    camera_look_at: vec3f,
    aspect_ratio: f32,
    camera_up: vec3f,
};

const PI = 3.14159265359;
const ETA = 0.00001;

const BACKGROUND_COLOR: vec3f = vec3f(0.0, 0.0, 0.5);

alias ShaderType = u32;
const SHADER_TYPE_LAMBERTIAN: u32 = 0u;
const SHADER_TYPE_PHONG: u32 = 1u;
const SHADER_TYPE_MIRROR: u32 = 2u;
const SHADER_TYPE_NO_RENDER: u32 = 255u;
const SHADER_TYPE_DEFAULT: u32 = 0u;

const MAX_DEPTH: i32 = 10;

@group(0) @binding(0)
var<uniform> uniforms: Uniform;

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
    l_i: vec3f,
    w_i: vec3f,
    dist: f32,
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
        SHADER_TYPE_DEFAULT,
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
    //let e = vec3f(2.0, 1.5, 2.0);
    //let p = vec3f(0.0, 0.5, 0.0);
    //let u = vec3f(0.0, 1.0, 0.0);
    let e = uniforms.camera_pos;
    let p = uniforms.camera_look_at;
    let u = uniforms.camera_up;
    let v = normalize(p - e);
    let d = 1.0; //uniforms.camera_constant;
    let aspect = uniforms.aspect_ratio;

    let b1 = normalize(cross(v, u));
    let b2 = cross(b1, v);

    let q = b1 * uv.x * aspect + b2 * uv.y + v*d;

    let ray = Ray(
        q,
        e,
        1000.0,
        0.0,
    );
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
    // each loop is one bounce
    for (var i = 0; i < 1; i++) {
        if (intersect_scene(&r, &hit)) {
            result += shade(&r, &hit);
        } else {
            result += bgcolor.rgb; break;
        }

        if (hit.has_hit) {
            break;
        }
    }

    
    //return vec4f(result, bgcolor.a);
    return vec4f(pow(result, vec3f(1.0/1.0)), bgcolor.a);
}

fn intersect_scene(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    var has_hit = false;
    has_hit = has_hit || intersect_plane(r, hit, vec3f(0.0, 0.0, 0.0), vec3f(0.0, 1.0, 0.0));
    let arr = array<vec3f, 3>(vec3f(-0.2, 0.1, 0.9), vec3f(0.2, 0.1, 0.9), vec3f(-0.2, 0.1, -0.1));
    has_hit = has_hit || intersect_triangle(r, hit, arr);
    has_hit = has_hit || intersect_sphere(r, hit, vec3f(0.0, 0.5, 0.5), 0.3);
    return has_hit;
}

fn intersect_plane(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, position: vec3f, normal: vec3f) -> bool {
    let ray = *r;
    let distance = dot((position - ray.origin), normal)/(dot(ray.direction, normal));
    
    if (distance < ray.tmin || distance > ray.tmax) {
        return false;
    }
    (*r).tmax = distance;
    (*hit).has_hit = true;
    (*hit).dist = distance;
    let pos = ray_at(ray, distance);
    (*hit).position = pos;
    (*hit).normal = normal;
    (*hit).diffuse = vec3f(0.1, 0.7, 0.0);
    return true;
}

fn intersect_triangle(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, v: array<vec3f, 3>) -> bool {
    let ray = *r;
    let e0 = v[1] - v[0];
    let e1 = v[2] - v[0];
    let normal = cross(e0, e1);
    let nom = cross(v[0] - ray.origin, ray.direction);
    let denom = dot(ray.direction, normal);
    let beta = dot(nom, e1) / denom;
    if (beta <= 0.0) {
        return false;
    }
    let gamma = - dot(nom, e0) / denom;
    if (gamma <= 0.0) {
        return false;
    }
    let alpha = 1.0 - beta - gamma;
    if (alpha <= 0.0) {
        return false;
    }

    let distance = dot(v[0] - ray.origin, normal) / denom;
    if (distance > ray.tmax || distance < ray.tmin) {
        return false;
    }

    (*r).tmax = distance;
    (*hit).has_hit = true;
    (*hit).dist = distance;
    let pos = ray_at(ray, distance);
    (*hit).position = pos;
    (*hit).normal = normal;
    (*hit).diffuse = vec3f(0.4, 0.3, 0.2);
    return true;
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
    (*hit).has_hit = true;
    (*hit).dist = root;
    let pos = ray_at(ray, root);
    let normal = normalize(pos - center);
    (*hit).position = pos;
    (*hit).normal = normal;
    (*hit).diffuse = vec3f(0.0, 0.5, 0.0);
    (*hit).shader = SHADER_TYPE_MIRROR;
    return true;
}

fn sample_point_light(pos: vec3f) -> Light {
    let light_pos = vec3f(0.0, 1.2, 0.0);
    let light_intensity = vec3f(PI, PI, PI);
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
    hit_record.has_hit = false;
    hit_record.depth += 1;

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
        default: {
            color = vec3f(0.0, 0.0, 0.0);
        }
    }
    *hit = hit_record;
    return color;
}

fn lambertian(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    var hit_record = *hit;
    (*hit).depth = MAX_DEPTH;
    let light = sample_point_light(hit_record.position);
    let normal = hit_record.normal;

    let ray_dir = light.w_i;
    let ray_orig = hit_record.position + hit_record.normal * ETA;
    var ray = ray_init(ray_dir, ray_orig);

    let blocked = intersect_scene(&ray, hit);
    let ambient = hit_record.ambient;
    var diffuse = vec3f(0.0);

    // ambient only
    if (blocked) {
        
    } else { // ambient and diffuse
        diffuse = light_diffuse_contribution(light, normal, hit_record.specular);
    }

    return diffuse * diffuse_and_ambient(hit_record.diffuse, ambient);
}

fn light_diffuse_contribution(light: Light, normal: vec3f, specular: f32) -> vec3f {
    let one_minus_specular = 1.0 - specular;
    var diffuse = vec3f(dot(normal, light.w_i));
    diffuse *= light.l_i;
    diffuse *= one_minus_specular / PI;
    return diffuse;
}

fn diffuse_and_ambient(diffuse: vec3f, ambient: vec3f) -> vec3f {
    return (ambient * 0.1 + diffuse * 0.9);
} 

fn phong(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    let hit_record = *hit;
    if (hit_record.has_hit) {
        return hit_record.diffuse * sample_point_light(hit_record.position).l_i;
    }
    return vec3f(0.0, 0.0, 0.0);
}


fn mirror(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    let hit_record = *hit;

    let normal = hit_record.normal;
    let ray_dir = reflect((*r).direction, normal);
    let ray_orig = hit_record.position + normal * ETA;
    var ray = ray_init(ray_dir, ray_orig);

    let mirrored = intersect_scene(&ray, hit);

    if (mirrored) {
        return lambertian(&ray, hit);
        //let light = sample_point_light(hit_record.position);
        //let diffuse = light_diffuse_contribution(light, (*hit).normal, (*hit).specular);
        //return  diffuse * diffuse_and_ambient((*hit).diffuse, (*hit).ambient);
    } else {
        return BACKGROUND_COLOR;
        //return vec3f(0.0, 0.0, 0.0);
    }


    if (hit_record.has_hit) {
        return hit_record.diffuse * sample_point_light(hit_record.position).l_i;
    }
    return vec3f(0.0, 0.0, 0.0);

}

fn transmit(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    return vec3f(0.0, 0.0, 0.0);
}