cbuffer ConstantBuffer : register(b0) {
    float2 size;
    float2 position;
    float yaw; // Yaw rotation angle in radians
};

struct VS_IN {
    float3 pos : POSITION;
    float2 tex : TEXCOORD;
};

struct PS_IN {
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD;
};

PS_IN VSMain(VS_IN input) {
    PS_IN output;
    
    // Calculate sine and cosine of the yaw angle
    float sinYaw = sin(yaw);
    float cosYaw = cos(yaw);
    
    // Apply yaw rotation to the input position
    float2 rotatedPos;
    rotatedPos.x = input.pos.x * cosYaw - input.pos.y * sinYaw;
    rotatedPos.y = input.pos.x * sinYaw + input.pos.y * cosYaw;
    
    // Scale the rotated rectangle to the desired size and position
    output.pos = float4(
        rotatedPos.x * size.x + position.x,
        rotatedPos.y * size.y + position.y,
        0.0,
        1.0
    );
    
    output.tex = input.tex;
    return output;
}