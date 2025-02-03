cbuffer ItemDataBuffer : register (b0) {
    float2 tx_size; 
    float2 start_pos;
    float2 end_pos;
    float spin_speed;
    float scale;
    float duration;
    float elapsed_time;
}

struct VS_IN {
    float2 pos : POSITION;
    float2 tex : TEXCOORD;
};

struct PS_IN {
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD;
};

// Interpolate between two positions using an Arc of the provided height
float2 ArcInterpolation(float2 start, float2 end, float t, float height)
{
    // Linear interpolation between start and end
    float2 value = lerp(start, end, t);

    // Calculate the arc height using a parabola
    float arc = height * t * (1.0f - t);

    // Add the arc to the linear interpolation
    return float2(value.x, value.y + arc);
}

#define PI 3.14159265358979323846f

float YawInterpolation(float spin_speed, float elapsed_time) {
    // radians per millisecond
    float rotationSpeed = 2.0f * PI / spin_speed; 
    return rotationSpeed * elapsed_time;
}

// Apply yaw onto the provided input 
float2 ApplyYaw(float2 input, float yaw) {
    float sinYaw = sin(yaw);
    float cosYaw = cos(yaw);

    return float2(
        input.x * cosYaw - input.y * sinYaw,
        input.x * sinYaw + input.y * cosYaw
    );
}

PS_IN VSMain(VS_IN input) {
    PS_IN output;
    
    float item_time = clamp(elapsed_time / duration, 0.0f, 1.0f);
    float yaw = YawInterpolation(spin_speed, elapsed_time);

    float2 inputPosition = input.pos;

    // Apply yaw rotation to the input position
    float2 rotatedInputPosition = ApplyYaw(inputPosition, yaw);

    // Interpolate the current position in the throw arc
    float2 position = ArcInterpolation(
        start_pos,
        end_pos,
        item_time,
        0.5
    );
    
    // Adjust normalized texture scale by the item scale
    float2 size = tx_size * scale;

    // Multiply positioning
    float2 outputPosition = (rotatedInputPosition * size) + position;
    
    output.pos = float4(outputPosition.xy, 0.0, 1.0);
   
    // output.pos =  float4(input.pos, 1.0);
    output.tex = input.tex; 

    return output;
}

