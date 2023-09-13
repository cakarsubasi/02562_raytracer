// Vertex shader

struct Uniform {
    view_proj: mat4x4<f32>, // 64 bytes
    camera_pos: vec3f, // 12 bytes
    camera_constant: f32, // 4 bytes
    camera_look_at: vec3f, // 12 bytes
    aspect_ratio: f32, // 4 bytes
    camera_up: vec3f, // 12 bytes
    // 4 byte padding
};

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

struct HitRecord {
    has_hit: bool,
    dist: f32,
    position: vec3f,
    normal: vec3f,
    color: vec3f,
};

fn hit_record_init() -> HitRecord {
    return HitRecord(
        false, 
        0.0, 
        vec3f(0.0), 
        vec3f(0.0), 
        vec3f(0.0)
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
    let d = 1.0;
    let aspect = uniforms.aspect_ratio;

    let b1 = normalize(cross(v, u));
    let b2 = cross(b1, v);

    let q = b1 * uv.x + b2 * uv.y / aspect + v*d;

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
    has_hit = has_hit || intersect_sphere(r, hit, vec3f(0.0, 0.5, 0.0), 0.3);
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
    (*hit).color = vec3f(0.0, 0.0, 0.0);
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
    (*hit).normal = vec3f(0.0, 0.0, 0.0);
    (*hit).color = vec3f(0.3, 0.0, 0.0);
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
    (*hit).position = pos;
    (*hit).normal = normalize(pos - center);
    (*hit).color = vec3f(0.0, 0.5, 0.0);
    return true;
}

fn shade(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    if ((*hit).has_hit) {
        return (*hit).color;
    }
    return vec3f(0.0, 0.0, 0.0);
}

fn lambertian(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    return vec3f(0.0, 0.0, 0.0);
}

fn phong(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    return vec3f(0.0, 0.0, 0.0);

}
fn mirror(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    return vec3f(0.0, 0.0, 0.0);

}