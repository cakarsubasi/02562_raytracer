
## Worksheet 9 - Environment Mapping

To support environment mapping, we need to extend the rendering framework to support multiple textures. That I did. What I forgot to do was also abstract away being able to set textures to sRGB or linear. Oops. I am sure linear textures look fine on worksheet 3, right?

![](./img/w3_e4_linear_texture.png)

Nevertheless, for part 1, I used an sRGB texture, and for part 2 and 3, I used linear. 

### 1. SDR Pixar Campus

This part is relatively straightforward. We just need a function to convert a direction to a sample in the environment map.

```rs
fn environment_map(direction: vec3f) -> vec3f {
    let d_x = direction.x;
    let d_y = direction.y;
    let d_z = direction.z;
    let u = 0.5 * (1.0 + (1.0 / PI) * atan2(d_x, -d_z)); // atan instead of atan2 breaks this
    let v = 1.0 / PI * acos(-d_y);
    return textureSample(hdri0, hdri0_sampler, vec2f(u, 1.0 - v)).rgb;
}
```

I was mildly irritated when I used `atan` instead of `atan2` and had one side of the scene mirrored and inverted on the other side and did not know what was wrong.

We then call it when our rays miss, and we are done:

```rs
    for (var i = 0; i < max_depth; i++) {
        if (intersect_scene_bsp(&r, &hit)) {
            result += shade(&r, &hit, &t);
        } else {
            result += environment_map(r.direction) * hit.factor; // new
            break;
        }

        if (hit.has_hit) {
            break;
        }
    }
```

Base color looks just fine:

![](./img/w9_e1_teapot_basecolor.png)

And so does the mirror:

![](./img/w9_e1_teapot_mirror.png)

And the lambertian:

![](./img/w9_e1_teapot_diffuse.png)

Wait no, that is not right at all. Well, the render is correct but an sRGB environment map cannot create physically correct lighting for lambertian materials.

I have also taken images of the bunny:

![](./img/w9_e1_bunny_basecolor.png)

![](./img/w9_e1_bunny_mirror.png)

![](./img/w9_e1_bunny_diffuse.png)

### 2. HDR Pixar Campus

In this part we are using the high dynamic range version of the same environment map in png format where the alpha channel contains an exponent for the brightness of a particular pixel. So our mapping function is modified:

```rs
fn environment_map(direction: vec3f) -> vec3f {
    let d_x = direction.x;
    let d_y = direction.y;
    let d_z = direction.z;
    let u = 0.5 * (1.0 + (1.0 / PI) * atan2(d_x, -d_z));
    let v = 1.0 / PI * acos(-d_y);

    let sample = textureSample(hdri0, hdri0_sampler, vec2f(u, 1.0 - v));
    var color = sample.rgb;
    let exponent = sample.a * 255.0 - 128.0;
    color = color * pow(2.0, f32(exponent));
    return color;
}
```

We are not done yet, we also want simulated AO with the ground plane.

For that we create a holdout shader:

```rs
fn holdout_shader(r: ptr<function, Ray>, hit: ptr<function, HitRecord>, rand: ptr<function, u32>) -> vec3f {
    let normal = normalize((*hit).normal);
    let xi1 = rnd(rand);
    let xi2 = rnd(rand);
    let thet = acos(sqrt(1.0-xi1));
    let phi = 2.0 * PI * xi2;
    let tang_dir = spherical_direction(sin(thet), cos(thet), phi);
    let direct_dir = rotate_to_normal(normal, tang_dir);

    var ray = ray_init(direct_dir, (*hit).position);
    ray.tmin = ETA;
    ray.tmax = 5000.0;
    
    var hit_info = hit_record_init();
    let blocked = intersect_trimesh_immediate_return(&ray, &hit_info);

    // if blocked, return zero contribution (AO)
    if (blocked) {
        return vec3f(0.0);
    }

    // else just return the environment map
    (*hit).has_hit = true;
    return environment_map((*r).direction) * (*hit).factor;
}
```

This makes it so that any ray reflected from our object to the ground plane samples a random direction and if the ray hits our object again, we return a zero contribution. This adds a subtle shadow at the bottom of our kettle.

![](./img/w9_e2_teapot_mirror.png)

![](./img/w9_e2_teapot_diffuse.png)

Although the mirror looks good, the lambertian is quite noisy. Without importance sampling, this scene will take a pretty long time to converge since the sun is a tiny part of the sky we are pretty unlikely to hit.

Bonus picture where I switched the shader midway through the render:

![](./img/w9_e2_teapot_mixture.png)

I have also taken pictures of the bunny, however the bunny is floating and I did not update the position of the ground plane:

![](./img/w9_e2_bunny_mirror.png)

![](./img/w9_e2_bunny_diffuse.png)

### 3. Directional Light

Time for some more fakery.

Not done.

### 4. Another HDRI

I did not do this part, sorry.