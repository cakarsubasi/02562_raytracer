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

alias ShaderType = u32;
const SHADER_TYPE_LAMBERTIAN: u32 = 0u;
const SHADER_TYPE_PHONG: u32 = 1u;
const SHADER_TYPE_MIRROR: u32 = 2u;
const SHADER_TYPE_NO_RENDER: u32 = 255u;
const SHADER_TYPE_DEFAULT: u32 = 0u;

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
    // Shader properties
    color: vec3f,
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
        vec3f(0.0),
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
    let max_depth = 10;
    let uv = in.coords * 0.5;
    var r = get_camera_ray(uv);
    var hit = hit_record_init();

    var result = vec3f(0.0);
    // each loop is one bounce
    for (var i = 0; i < max_depth; i++) {
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
    (*hit).color = vec3f(0.1, 0.7, 0.0);
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
    (*hit).color = vec3f(0.4, 0.3, 0.2);
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
    (*hit).color = vec3f(normal.x, normal.y, normal.z);//vec3f(0.0, 0.5, 0.0);
    return true;
}

fn sample_point_light(pos: vec3f) -> Light {
    let light_pos = vec3f(0.0, 1.0, 0.0);
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
    hit_record.has_hit = true;
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
    let hit_record = *hit;
    if (hit_record.has_hit) {
        return hit_record.color * sample_point_light(hit_record.position).l_i;
    }
    return vec3f(0.0, 0.0, 0.0);
}

fn phong(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    let hit_record = *hit;
    if (hit_record.has_hit) {
        return hit_record.color;
    }
    return vec3f(0.0, 0.0, 0.0);

}
fn mirror(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    let hit_record = *hit;
    if (hit_record.has_hit) {
        return hit_record.color;
    }
    return vec3f(0.0, 0.0, 0.0);

}