const PI = 3.14159265359;
const ETA = 0.0001;

const BACKGROUND_COLOR: vec3f = vec3f(0.0, 0.0, 0.0);

alias ShaderType = u32;
const SHADER_TYPE_LAMBERTIAN: u32 = 0u;
const SHADER_TYPE_PHONG: u32 = 1u;
const SHADER_TYPE_MIRROR: u32 = 2u;
const SHADER_TYPE_TRANSMIT: u32 = 3u;
const SHADER_TYPE_GLOSSY: u32 = 4u;
const SHADER_TYPE_NORMAL: u32 = 5u;
const SHADER_TYPE_BASECOLOR: u32 = 6u;
const SHADER_TYPE_TRANSPARENT: u32 = 7u;
const SHADER_TYPE_NO_RENDER: u32 = 255u;
const SHADER_TYPE_DEFAULT: u32 = 0u;

const MAX_DEPTH: i32 = 50;

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
    factor: vec3f,
    extinction: vec3f,
    emit: bool,
    uv0: vec2f,
    material: u32,
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
        vec3f(1.0),
        vec3f(1.0),
        true,
        vec2f(0.0),
        0u,
        // shader properties
        SHADER_TYPE_NO_RENDER,
        1.0,
        0.9,
        42.0,
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

// PRNG xorshift seed generator by NVIDIA
fn prng_xorshift_seed_generator(val0: u32, val1: u32) -> u32 {
      let N = 16u; // User specified number of iterations
      var v0: u32 = val0;
      var v1: u32 = val1;
      var s0: u32 = 0u;

      for(var n = 0u; n < N; n++) {
        s0 += 0x9e3779b9u;
        v0 += ((v1<<4u)+0xa341316cu)^(v1+s0)^((v1>>5u)+0xc8013ea4u);
        v1 += ((v0<<4u)+0xad90777du)^(v0+s0)^((v0>>5u)+0x7e95761eu);
      }

      return v0;
}

 // Generate random unsigned int in [0, 2^31)
 fn mcg31(prev: ptr<function, u32>) -> u32 {
    let LCG_A = 1977654935u; // Multiplier from Hui-Ching Tang [EJOR 2007]
    *prev = (LCG_A * (*prev)) & 0x7FFFFFFFu;
    return *prev;
}
// Generate random float in [0, 1)
fn rnd(prev: ptr<function, u32>) -> f32
{
    return f32(mcg31(prev)) / f32(0x80000000u);
}

// Generate random float in [0, 1)
fn rnd_int(prev: ptr<function, u32>) -> u32
{
    return mcg31(prev);
}


// Given a direction vector v sampled around the z-axis of a
// local coordinate system, this function applies the same
// rotation to v as is needed to rotate the z-axis to the
// actual direction n that v should have been sampled around
// [Frisvad, Journal of Graphics Tools 16, 2012;
//  Duff et al., Journal of Computer Graphics Techniques 6, 2017].
fn rotate_to_normal(normal: vec3f, v: vec3f) -> vec3f
{
    let signbit = sign(normal.z + 1.0e-16);
    let a = -1.0/(1.0 + abs(normal.z));
    let b = normal.x*normal.y*a;
    return vec3f(1.0 + normal.x*normal.x*a, b, -signbit*normal.x)*v.x
      + vec3f(signbit*b, signbit*(1.0 + normal.y*normal.y*a), -normal.y)*v.y
      + normal*v.z;
}

// Given spherical coordinates, where theta is the
// polar angle and phi is the azimuthal angle, this
// function returns the corresponding direction vector
fn spherical_direction(sin_theta: f32, cos_theta: f32, phi: f32) -> vec3f
{
    let sin_phi = sin(phi);
    let cos_phi = cos(phi);
    return vec3f(sin_theta*cos_phi, sin_theta*sin_phi, cos_theta);
}

fn fresnel_r(cos_thet_i: f32, cos_thet_t: f32, ni_over_nt: f32) -> f32 {
    let ii = ni_over_nt * cos_thet_i;
    let tt = 1.0 * cos_thet_t;
    let ti = 1.0 * cos_thet_i;
    let it = ni_over_nt * cos_thet_t;

    let r1 = (ii - tt) / (ii + tt);
    let r2 = (ti - it) / (ti + it);
    let R = 0.5 * (r1 * r1 + r2 * r2);
    return R;
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

fn get_camera_ray(uv: vec2f, jitter: vec2f) -> Ray {
    let e = uniforms.camera_pos;
    let p = uniforms.camera_look_at;
    let u = uniforms.camera_up;
    let v = normalize(p - e);
    let d = uniforms.camera_constant;
    let aspect = uniforms.aspect_ratio;

    let b1 = normalize(cross(v, u));
    let b2 = cross(b1, v);

    let j_x = jitter.x;
    let j_y = jitter.y;
    let q = normalize(b1 * (uv.x + j_x) * aspect + b2 * (uv.y + j_y) + v*d);

    let ray = ray_init(q, e);
    return ray;
}

fn environment_map(r: ptr<function, Ray>) -> vec3f {
    let direction = (*r).direction;
    let d_x = direction.x;
    let d_y = direction.y;
    let d_z = direction.z;
    let u = 0.5 * (1.0 + (1.0 / PI) * atan2(d_x, -d_z)); // atan instead of atan2 breaks this
    let v = 1.0 / PI * acos(-d_y);
    return textureSample(hdri0, hdri0_sampler, vec2f(u, 1.0 - v)).rgb;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let bgcolor = vec4f(BACKGROUND_COLOR, 1.0);
    let max_depth = MAX_DEPTH;
    let uv = in.coords * 0.5;

    let coord_y: u32 = u32(in.clip_position.y);
    let coord_x: u32 = u32(in.clip_position.x);
    let res_x: u32 = uniforms.resolution.x;
    let launch_idx = coord_y*uniforms.resolution.x + coord_x;
    var t = prng_xorshift_seed_generator(launch_idx, uniforms.iteration);
    let jitter = vec2f(rnd(&t), rnd(&t))/f32(uniforms.resolution.y);
    
    var result = vec3f(0.0);
    // each loop is one bounce
    var r = get_camera_ray(uv, jitter);
    var hit = hit_record_init();
    for (var i = 0; i < max_depth; i++) {
        if (intersect_scene_bsp(&r, &hit)) {
            result += shade(&r, &hit, &t);
        } else {
            result += environment_map(&r); break;
        }

        if (hit.has_hit) {
            break;
        }
    }
    
    let curr_sum = textureLoad(renderTexture, vec2u(in.clip_position.xy), 0).rgb*f32(uniforms.iteration);
    let accum_color = (result + curr_sum)/f32(uniforms.iteration + 1u);

    var output = FragmentOutput(
        vec4f(saturate(pow(accum_color, vec3f(1.5/1.0))), bgcolor.a),
        max(vec4f(accum_color, 1.0), vec4f(0.0)),
    );
    if (any(result < vec3f(0.0)) || any(accum_color < vec3f(0.0))) {
        output.frame = vec4f(error_shader(), bgcolor.a);
    }
    return output;
}

fn intersect_scene_bsp(r: ptr<function, Ray>, hit: ptr<function, HitRecord>) -> bool {
    var has_hit = false;
    var current = false;
    current = intersect_trimesh(r, hit);
    if (current) {
        (*hit).shader = SHADER_TYPE_BASECOLOR;
    }
    has_hit = has_hit || current;
    return has_hit;
}

fn intersect_triangle_indexed(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, v: u32) -> bool {
    let v0_i = indexBuffer[v].x;
    let v1_i = indexBuffer[v].y;
    let v2_i = indexBuffer[v].z;
    let material = indexBuffer[v].w;
    let v0 = combinedBuffer[v0_i].position.xyz;
    let v1 = combinedBuffer[v1_i].position.xyz;
    let v2 = combinedBuffer[v2_i].position.xyz;
    //let n0 = combinedBuffer[v0_i].normal.xyz;
    //let n1 = combinedBuffer[v1_i].normal.xyz; 
    //let n2 = combinedBuffer[v2_i].normal.xyz;

    let ray = *r;
    let w_i = ray.direction;
    let o = ray.origin;

    let e0 = v1 - v0;
    let e1 = v2 - v0;
    let o_to_v0 = v0 - o;
    let normal = cross(e0, e1);

    let nom = cross(o_to_v0, w_i);
    var denom = dot(w_i, normal);
    if (abs(denom) < 1e-10) {
        return false;
    }

    let n0 = normal; // Our box does not have vertex normals, so we have to use this
    let n1 = normal; // 
    let n2 = normal; // 

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
    return true;
}

fn sample_area_light(pos: vec3f, idx: u32, rand: ptr<function, u32>) -> Light {
    let light_tri_idx: u32 = lightIndices[idx];
    let light_triangle: vec4u = indexBuffer[light_tri_idx];
    let v0 = combinedBuffer[light_triangle.x].position.xyz;
    let v1 = combinedBuffer[light_triangle.y].position.xyz;
    let v2 = combinedBuffer[light_triangle.z].position.xyz;
    let area = triangle_area(v0, v1, v2);
    let light_mat = materials[light_triangle.w];
    let l_e = light_mat.ambient.xyz;
    let psi1 = sqrt(rnd(rand));
    let psi2 = rnd(rand);
    let alpha = 1.0 - psi1;
    let beta = (1.0 - psi2) * psi1;
    let gamma = psi2 * psi1;
    let normal = normalize(cross((v0 - v1), (v0 - v2)));

    let sampled_point = (v0 * alpha + v1 * beta + v2 * gamma);

    let light_direction = sampled_point - pos;
    let cos_l = max(dot(normalize(-light_direction), normal), 0.0);
    let distance = sqrt(dot(light_direction, light_direction));
    let light_intensity = (l_e * area) * cos_l / (distance * distance);
    var light = light_init();
    light.l_i = light_intensity;
    light.w_i = normalize(light_direction);
    light.dist = distance;
    return light;
}

fn shade(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, rand: ptr<function, u32>) -> vec3f {
    var hit_record = *hit;
    var color = vec3f(0.0, 0.0, 0.0);
    hit_record.has_hit = true;
    hit_record.depth += 1;
    *hit = hit_record;

    switch(hit_record.shader) {
        case 0u: {
            color = lambertian(r, hit, rand);
        }
        case 2u: {
            color = mirror(r, hit, rand);
        }
        case 5u: {
            color = shade_normal(r, hit, rand);
        }
        case 6u: {
            color = shade_base_color(r, hit, rand);
        }
        case 7u: {
            color = transparent(r, hit, rand);
        }
        default: {
            color = error_shader();
        }
    }
    return color;
}

fn lambertian(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, rand: ptr<function, u32>) -> vec3f {
    let material = get_material(hit);
    let brdf = material.diffuse.rgb / PI;
    let emission = material.ambient.rgb;
    var diffuse = vec3f(0.0);
    var ambient = vec3f(0.0);

    let normal = (*hit).normal;

    // Pick a random area light to sample
    let light_tris = arrayLength(&lightIndices) - 1u;
    let idx = (rnd_int(rand) % light_tris) + 1u;
    let light = sample_area_light((*hit).position, idx, rand);

    // Trace shadow rays to area light
    let ray_dir = light.w_i;
    let ray_orig = (*hit).position;
    var ray = ray_init(ray_dir, ray_orig);
    ray.tmax = light.dist - ETA;
    ray.tmin = ETA;

    var hit_info = hit_record_init();
    let blocked = intersect_scene_bsp(&ray, &hit_info);

    if (!blocked) {
        diffuse = brdf * vec3f(saturate(dot(normal, light.w_i))) * light.l_i * f32(light_tris);
    }  
    // Add emission only during direct lighting pass 
    if ((*hit).emit) { 
        ambient = emission * (*hit).factor;
    }

    // Scale diffuse and hit factor and Russian Roulette to decide to trace more
    diffuse = diffuse * (*hit).factor;
    (*hit).factor *=  brdf * PI;
    let prob_reflection = (brdf.r + brdf.g + brdf.b) / 3.0;
    let step = rnd(rand);
    if (step < prob_reflection) {
        setup_indirect(r, hit, rand);
        (*hit).factor /= prob_reflection;
    }

    return diffuse + ambient;
}

fn setup_indirect(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, rand: ptr<function, u32>) {
    // Indirect contribution
    let normal = normalize((*hit).normal);
    let xi1 = rnd(rand);
    let xi2 = rnd(rand);
    let thet = acos(sqrt(1.0-xi1));
    let phi = 2.0 * PI * xi2;
    let tang_dir = spherical_direction(sin(thet), cos(thet), phi);
    let indirect_dir = rotate_to_normal(normal, tang_dir);

    (*r).direction = indirect_dir;
    (*r).origin = (*hit).position;
    (*r).tmin = ETA;
    (*r).tmax = 5000.0;

    (*hit).has_hit = false; 
    (*hit).emit = false;
}

fn mirror(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, rand: ptr<function, u32>) -> vec3f { 
    var hit_record = *hit;
    
    let normal = hit_record.normal;
    let ray_dir = reflect((*r).direction, normal);
    let ray_orig = hit_record.position + normal * ETA;
    *r = ray_init(ray_dir, ray_orig);

    hit_record.has_hit = false;

    *hit = hit_record;
    (*hit).emit = true;
    return vec3f(0.0, 0.0, 0.0);
}

fn transparent(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, rand: ptr<function, u32>) -> vec3f {
    let w_i = -normalize((*r).direction);
    let normal = normalize((*hit).normal);
    var out_normal = vec3f(0.0);
    var ior = (*hit).ior1_over_ior2;
    // figure out if we are inside or outside
    var cos_thet_i = dot(w_i, normal);
    // normals point outward, so if this is positive
    // we are inside the object
    // and if this is negative, we are outside
    var absorption = 0.0;
    var T_r = vec3f(1.0);
    if (cos_thet_i < 0.0) {
        // entering
        cos_thet_i = dot(w_i, -normal); 
        out_normal = -normal;
    } else {
        // exiting
        ior = 1.0 / ior;
        out_normal = normal;
        let s = length((*hit).position - (*r).origin);
        let rho_t = (*hit).extinction;
        T_r = exp(-rho_t*s);
        absorption = 1.0 - (T_r.r + T_r.g + T_r.b) / 3.0;
    }

    let cos_thet_t_2 = (1.0 - (ior*ior) * (1.0 - cos_thet_i * cos_thet_i));
    var reflection_prob = 0.0;
    if (cos_thet_t_2 < 0.0) {
        // total internal reflection
        reflection_prob = 1.0;
    } else {
        reflection_prob = fresnel_r(cos_thet_i, sqrt(cos_thet_t_2), ior);
    }
    let tangent = (out_normal * cos_thet_i - w_i);
    
    let w_t = ior * tangent - (normalize(out_normal) * sqrt(cos_thet_t_2));
    let orig = (*hit).position;

    *r = ray_init(w_t, orig); 
    (*hit).has_hit = false;
    (*hit).emit = true;

    let step = rnd(rand);
    if (step < reflection_prob) {
        (*hit).normal = out_normal;
        return mirror(r, hit, rand);
    } else {
        let step1 = rnd(rand);
        if (step1 < absorption) {
            (*hit).factor *= (*hit).extinction / absorption;
        }
        return vec3f(0.0);
    }
}


fn shade_normal(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, rand: ptr<function, u32>) -> vec3f {
    return ((*hit).normal + 1.0) * 0.5;
}

fn shade_base_color(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, rand: ptr<function, u32>) -> vec3f {
    let index = (*hit).material;
    let color = materials[index].diffuse.xyz + materials[index].ambient.xyz;
    return color;
}

fn error_shader() -> vec3f {
    return vec3f(0.7, 0.0, 0.7);
}
