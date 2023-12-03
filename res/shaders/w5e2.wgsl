// Vertex shader

const PI = 3.14159265359;
const ETA = 0.00001;

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
//@group(0) @binding(2)
//var<storage> jitter: array<vec2f>;

//@group(1) @binding(0)
//var sampler0: sampler;
//@group(1) @binding(1)
//var texture0: texture_2d<f32>;

// GPU will always align to 16, so this does not waste space
//@group(2) @binding(0)
//var<storage> vertexBuffer: array<vec4f>;
//// GPU will always align to 16, so this does not waste space
//@group(2) @binding(1)
//var<storage> indexBuffer: array<vec4u>;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>,
};

struct FragmentOutput {
    @location(0) frame: vec4f,
    @location(1) accum: vec4f,
}

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
    uv0: vec2f,
    // shader properties
    shader: Shader,
};

struct Shader {
    shader: ShaderType,
    use_texture: bool,
    base_color: vec3f,
    ior1_over_ior2: f32,
    specular: f32,
    shininess: f32,
};

fn shader_init(hit: ptr<function, HitRecord>, shader_type: ShaderType) -> Shader {
    return Shader(
        shader_type,
        false,
        vec3f(0.0),
        1.0,
        0.0,
        0.0,
    );
}

fn hit_record_init() -> HitRecord {
    let shader = Shader(
        SHADER_TYPE_NO_RENDER,
        false,
        vec3f(0.0),
        1.0,
        0.0,
        0.0,
    );
    return HitRecord(
        false,
        0,
        0.0, 
        vec3f(0.0), 
        vec3f(0.0),
        // color contribution
        vec3f(0.0),
        vec3f(0.0),
        vec2f(0.0),
        // shader properties
        shader,
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

fn get_camera_ray(uv: vec2f, sample: u32) -> Ray {
    let e = uniforms.camera_pos;
    let p = uniforms.camera_look_at;
    let u = uniforms.camera_up;
    let v = normalize(p - e);
    let d = uniforms.camera_constant;
    let aspect = uniforms.aspect_ratio;

    let b1 = normalize(cross(v, u));
    let b2 = cross(b1, v);

    let j_x = jitter[sample].x;
    let j_y = jitter[sample].y;
    let q = normalize(b1 * (uv.x + j_x) * aspect + b2 * (uv.y + j_y) + v*d);

    let ray = ray_init(q, e);
    return ray;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let bgcolor = vec4f(0.1, 0.3, 0.6, 1.0);
    let max_depth = MAX_DEPTH;
    let uv = in.coords * 0.5;
    let subdiv = uniforms.subdivision_level;
    
    var result = vec3f(0.0);
    var textured = vec3f(0.0);
    // each loop is one bounce
    for (var sample = 0u; sample < subdiv * subdiv; sample++) {
        var r = get_camera_ray(uv, sample);
        var hit = hit_record_init();
        for (var i = 0; i < max_depth; i++) {
            if (intersect_scene_loop(&r, &hit)) {
                if (hit.shader.use_texture) {
                    textured = shade(&r, &hit);
                } else {
                    result += shade(&r, &hit);
                }
            } else {
                result += bgcolor.rgb; break;
            }

            if (hit.has_hit) {
                result += textured * texture_sample(&hit);
                break;
            }
        }
    }
    let multiplier = 1.0 / f32(subdiv * subdiv);
    result = result * multiplier;
    let output = FragmentOutput(
        vec4f(pow(result, vec3f(1.5/1.0)), bgcolor.a),
        vec4f(0.0),
    );
    return output;
}

fn texture_sample(hit: ptr<function, HitRecord>) -> vec3f {
    // Note that we are ignoring the potential alpha channel within the texture here
    // TODO: Might want to multiply alpha here
    var uv0_scaled = fract((*hit).uv0 * uniforms.uv_scale);

    return textureSample(texture0, sampler0, uv0_scaled).xyz;
}

fn intersect_scene_loop(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    var has_hit = false;
    var shader = shader_init(hit, SHADER_TYPE_LAMBERTIAN);
    shader.base_color = vec3f(0.9);
    let num_of_tris = arrayLength(&indexBuffer);
    for (var i = 0u; i < num_of_tris; i++) {
        has_hit = has_hit || wrap_shader(intersect_triangle_indexed(r, hit, i), hit, shader);
    }
    return has_hit;
}

fn wrap_shader(has_hit: bool, hit: ptr<function, HitRecord>, shader: Shader) -> bool {
    if (has_hit) {
        (*hit).shader = shader;
        if (shader.use_texture) {
            (*hit).shader.base_color = vec3f(1.0);
        }
    }
    return has_hit;
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
    var denom = dot(w_i, normal);
    if (abs(denom) < 1e-6) {
        return false;
    }

    let beta = dot(nom, e1) / (denom);
    let gamma = -dot(nom, e0) / (denom);
    let distance = dot(o_to_v0, normal) / denom;
    if (beta < 0.0 || gamma < 0.0 || beta + gamma > 1.0 || distance > ray.tmax || distance < ray.tmin) {
        return false;
    }

    (*r).tmax = distance;
    (*hit).dist = distance;
    let pos = ray_at(ray, distance);
    (*hit).position = pos;
    (*hit).normal = normalize(normal);

    return true;
}

fn intersect_triangle_indexed(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, v: u32) -> bool {
    let v1 = indexBuffer[v].x;
    let v2 = indexBuffer[v].y;
    let v3 = indexBuffer[v].z;
    let arr = array<vec3f, 3>(vertexBuffer[v1].xyz, vertexBuffer[v2].xyz, vertexBuffer[v3].xyz);
    return intersect_triangle(r, hit, arr);
}



fn sample_directional_light(pos: vec3f) -> Light {
    // a directional light is much like a point light, but the intensity
    // is independent of the distance
    let light_direction = -normalize(vec3f(-1.0));
    let light_intensity = 5.0 * vec3f(PI, PI, PI);
    let distance = 1.0;
    var light = light_init();
    light.l_i = light_intensity;
    light.dist = distance;
    light.w_i = light_direction;
    return light;
}

fn shade(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    var hit_record = *hit;
    var color = vec3f(0.0, 0.0, 0.0);
    hit_record.has_hit = true;
    hit_record.depth += 1;
    *hit = hit_record;

    switch(hit_record.shader.shader) {
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
    let light = sample_directional_light(hit_record.position);
    let normal = hit_record.normal;

    let ray_dir = light.w_i;
    let ray_orig = hit_record.position + hit_record.normal * ETA;
    var ray = ray_init(ray_dir, ray_orig);

    let blocked = intersect_scene_loop(&ray, hit);
    let ambient = hit_record.shader.base_color;
    var diffuse = hit_record.shader.base_color * light_diffuse_contribution(light, normal, hit_record.shader.specular);

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
    let specular = (*hit).shader.specular;
    let s = (*hit).shader.shininess;
    let normal = (*hit).normal;
    let position = (*hit).position;

    let w_o = normalize(uniforms.camera_pos - position); // view direction
    let light = sample_directional_light(position);
    let w_r = normalize(reflect(-light.w_i, normal));
    let diffuse = saturate(vec3f(dot(normal, light.w_i))) * light.l_i / PI;
    let w_o_dot_w_r = dot(w_o, w_r);
    let coeff = specular * (s + 2.0) / (2.0 * PI);

    let phong_overall = coeff * pow(saturate(w_o_dot_w_r), s) * diffuse;
    return vec3f(phong_overall);
}


fn mirror(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    var hit_record = *hit;
    
    let normal = hit_record.normal;
    let ray_dir = reflect((*r).direction, normal);
    let ray_orig = hit_record.position + normal * ETA;
    *r = ray_init(ray_dir, ray_orig);

    hit_record.has_hit = false;

    *hit = hit_record;

    return vec3f(0.0, 0.0, 0.0);

}

fn glossy(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    return phong(r, hit) + transmit(r, hit);
}

fn transmit(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    var hit_record = *hit;
    let ray = *r;
    let w_i = -normalize(ray.direction);
    let normal = normalize(hit_record.normal);
    var out_normal = vec3f(0.0);

    var ior = hit_record.shader.ior1_over_ior2;
    // figure out if we are inside or outside
    let cos_thet_i = dot(w_i, normal);
    // normals point outward, so if this is positive
    // we are inside the object
    // and if this is negative, we are outside
    if (cos_thet_i < 0.0) {
        // outside
        out_normal = -normal;
    } else {
        // inside
        ior = 1.0 / ior;
        out_normal = normal;
    }

    let cos_thet_t_2 = (1.0 - (ior*ior) * (1.0 - cos_thet_i * cos_thet_i));
    if (cos_thet_t_2 < 0.0) {
        return error_shader();
    }
    let tangent = ((normal * cos_thet_i - w_i));
    
    let w_t = ior * tangent - (out_normal * sqrt(cos_thet_t_2));
    let orig = hit_record.position + w_t * ETA;

    *r = ray_init(w_t, orig); 
    hit_record.has_hit = false;

    *hit = hit_record;
    return vec3f(0.0, 0.0, 0.0);
}

fn shade_normal(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    return ((*hit).normal + 1.0) * 0.5;
}

fn shade_base_color(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f {
    return (*hit).shader.base_color;
}

fn error_shader() -> vec3f {
    return vec3f(0.7, 0.0, 0.7);
}
