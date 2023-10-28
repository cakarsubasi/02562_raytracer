//struct Aabb {
//    min: vec4f,
//    max: vec4f,
//};

//@group(3) @binding(0) var<uniform> aabb: Aabb;

fn intersect_min_max(r: ptr<function, Ray>) -> bool
{
    var tmin = 1.0e32f;
    var tmax = -1.0e32f;

    for(var i = 0u; i < 3u; i++) {
        if(abs((*r).direction[i]) > 1.0e-8f) {
            let p1 = (aabb.min[i] - (*r).origin[i])/(*r).direction[i];
            let p2 = (aabb.max[i] - (*r).origin[i])/(*r).direction[i];
            let pmin = min(p1, p2);
            let pmax = max(p1, p2);
            tmin = min(tmin, pmin);
            tmax = max(tmax, pmax);
        }
    }

    if (tmin > tmax || tmin > (*r).tmax || tmax < (*r).tmin) {
        return false;
    }

    (*r).tmin = max(tmin - 1.0e-4f, (*r).tmin);
    (*r).tmax = min(tmax + 1.0e-4f, (*r).tmax);
    return true;
}