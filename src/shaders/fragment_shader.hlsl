Texture2D texture0 : register(t0);
SamplerState sampler0 : register(s0);

struct PS_IN {
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD;
};

float4 PSMain(PS_IN input) : SV_TARGET {
    return texture0.Sample(sampler0, input.tex);
}