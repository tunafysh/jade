struct Vertex {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>
) -> Vertex {
    return Vertex(
        vec4<f32>(position, 0.0, 1.0),
        color
    );
}

@fragment
fn fragment(input: Vertex) -> @location(0) vec4<f32> {
    return input.color;
}