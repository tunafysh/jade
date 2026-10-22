struct VertexInput
{
    float3 position : POSITION;
    float4 color    : COLOR0;
};

struct VertexOutput
{
    float4 position : SV_Position;
    float4 color    : COLOR0;
};

uniform float4x4 uModel;
uniform float4x4 uView;
uniform float4x4 uProjection;
uniform float4 color;

[shader("vertex")]
VertexOutput vertex(VertexInput input)
{
    VertexOutput output;

    output.position = mul(
        uProjection,
        mul(
            uView,
            mul(uModel, float4(input.position, 1.0))
        )
        );

    output.color = input.color;

    return output;
}

[shader("fragment")]
float4 fragment(VertexOutput input) : SV_Target
{
    return color;
}