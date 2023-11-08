struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // We do this here as I do not want to abstract away the canvas creation
    // Since this project already got stupidly complicated
    out.coords = vec2f(model.position.x * 0.9, model.position.y * 0.9);
    out.clip_position = vec4<f32>(model.position * 0.9, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.coords;
    var color = vec3f(0.1, 0.3, 0.6);
    return vec4f(color, 1.0);
}