struct Globals {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};


@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.position =
        globals.projection *
        globals.view *
        globals.model *
        vec4<f32>(input.position, 1.0);

    output.color = input.color;

    return output;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return globals.color;
}