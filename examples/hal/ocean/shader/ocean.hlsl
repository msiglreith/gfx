
// TODO Texture2D u_Texture: register(c0, space0);

Texture2D u_Texture: register(t1, space0);
SamplerState u_Sampler: register(s2, space1);

struct VsOutput {
    float4 pos: SV_POSITION;
};

VsOutput ocean_vs(float3 pos: ATTRIB0, float2 uv: ATTRIB1, float2 offset: ATTRIB2) {
    // TODO
}

float4 ocean_ps(VsOutput input) : SV_TARGET {
    return float4(1.0);
}
