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
    material: u32,
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
        0u,
        // shader properties
        shader,
    );
}

fn set_material(hit: ptr<function, HitRecord>, index: u32) {
    (*hit).material = index;
}

fn get_material(hit: ptr<function, HitRecord>) -> Material {
    return materials[(*hit).material];
}

fn triangle_area(v0: vec3f, v1: vec3f, v2: vec3f) -> f32 {
    let e0 = v0 - v1;
    let e1 = v0 - v2;
    let cr = cross(e0, e1);
    return 0.5 * sqrt(dot(cr, cr));
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
        if (!intersect_min_max(&r)) {
            result = bgcolor.rgb;
            break;
        } 
        for (var i = 0; i < max_depth; i++) {
            if (intersect_scene_bsp(&r, &hit)) {
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

fn intersect_scene_bsp(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    var shader = shader_init(hit, SHADER_TYPE_LAMBERTIAN);
    let has_hit = wrap_shader(intersect_trimesh(r, hit), hit, shader);
    return has_hit;
}

fn intersect_scene_loop(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    var has_hit = false;
    let num_of_tris = arrayLength(&indexBuffer);
    for (var i = 0u; i < num_of_tris; i++) {
        has_hit = has_hit || intersect_triangle_indexed(r, hit, i);
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

fn intersect_triangle_indexed(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, v: u32) -> bool {
    let v0_i = indexBuffer[v].x;
    let v1_i = indexBuffer[v].y;
    let v2_i = indexBuffer[v].z;
    let material = indexBuffer[v].w;
    let v0 = vertexBuffer[v0_i].xyz;
    let v1 = vertexBuffer[v1_i].xyz;
    let v2 = vertexBuffer[v2_i].xyz;


    let ray = *r;
    let w_i = ray.direction;
    let o = ray.origin;

    let e0 = v1 - v0;
    let e1 = v2 - v0;
    let o_to_v0 = v0 - o;
    let normal = cross(e0, e1);

    let nom = cross(o_to_v0, w_i);
    var denom = dot(w_i, normal);
    if (abs(denom) < 1e-6) {
        return false;
    }

    let n0 = normal;//normalBuffer[v0_i].xyz;
    let n1 = normal;//normalBuffer[v1_i].xyz;
    let n2 = normal;//normalBuffer[v2_i].xyz;

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
    (*hit).normal = normalize(n0 * (1.0 - beta - gamma) + n1 * beta + n2 * gamma);
    set_material(hit, material);

    return true;
}



fn sample_directional_light(pos: vec3f) -> Light {
    // a directional light is much like a point light, but the intensity
    // is independent of the distance
    let light_direction = -normalize(vec3f(-1.0));
    let light_intensity = 5.0 * vec3f(PI, PI, PI);
    let distance = 1.0e+10;
    var light = light_init();
    light.l_i = light_intensity;
    light.dist = distance;
    light.w_i = light_direction;
    return light;
}

fn sample_area_light(pos: vec3f, idx: u32) -> Light {
    let light_tri_idx: u32 = lightIndices[idx];
    let light_triangle: vec4u = indexBuffer[light_tri_idx];
    let v0 = vertexBuffer[light_triangle.x].xyz;
    let v1 = vertexBuffer[light_triangle.y].xyz;
    let v2 = vertexBuffer[light_triangle.z].xyz;
    let area = triangle_area(v0, v1, v2);
    let light_mat = materials[light_triangle.w];
    let l_e = light_mat.ambient.xyz;
    let center = (v0 + v1 + v2) / 3.0;

    let light_direction = center - pos;
    let distance = sqrt(dot(light_direction, light_direction));
    let light_intensity = l_e * area;
    var light = light_init();
    light.l_i = light_intensity;
    light.w_i = normalize(light_direction);
    light.dist = distance;
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
    let normal = hit_record.normal;
    let material = get_material(hit);
    let bdrf = material.diffuse.rgb;
    
    var diffuse = vec3f(0.0);

    let light_tris = arrayLength(&lightIndices);
    for (var idx = 0u; idx < light_tris; idx++) {
        let light = sample_area_light(hit_record.position, idx);

        //let ray_dir = light.w_i;
        //let ray_orig = hit_record.position + hit_record.normal * ETA;
        //var ray = ray_init(ray_dir, ray_orig);

        //let blocked = intersect_scene_bsp(&ray, hit);
        // too lazy to fix the shadowing issue for now, we will just skip it

        diffuse = diffuse + bdrf * light_diffuse_contribution(light, normal);
    }
    let blocked = false;
    let ambient = material.ambient.rgb;

    // ambient only
    //if (blocked) {
    //    return ambient * 0.1;
    //}

    return diffuse_and_ambient(diffuse, ambient);
}

fn light_diffuse_contribution(light: Light, normal: vec3f) -> vec3f {
    var diffuse = vec3f(dot(normal, light.w_i));
    diffuse = diffuse / (light.dist * light.dist);
    diffuse *= light.l_i;
    diffuse = diffuse / PI;
    return diffuse;
}

fn diffuse_and_ambient(diffuse: vec3f, ambient: vec3f) -> vec3f {
    return 0.9 * diffuse + 0.1 * ambient;
} 

fn phong(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> vec3f { 
    let hit_record = *hit;
    let ray = *r;

    let specular = hit_record.shader.specular;
    let s = hit_record.shader.shininess;
    let normal = hit_record.normal;

    let w_i = ray.direction;
    let w_o = normalize(uniforms.camera_pos - hit_record.position); // view direction
    let w_r = reflect(-w_i, normal);

    let light = sample_directional_light(hit_record.position);
    let light_dir = light.w_i;
    let light_intensity = light.l_i;
    let refl_dir = normalize(reflect(-light_dir, normal));

    let coeff = specular * (s + 2.0) / (2.0 * PI);
    let phong_total = pow(saturate(dot(w_o, refl_dir)), s);

    return coeff * phong_total * vec3f(1.0);
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
    let index = (*hit).material;
    let color = materials[index].diffuse.xyz + materials[index].ambient.xyz;
    return color;
}

fn error_shader() -> vec3f {
    return vec3f(0.7, 0.0, 0.7);
}