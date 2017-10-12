
#define RESOLUTION 512
#define RESOLUTION_LOG 9

static float pi = 3.1415926f;

groupshared float2 shared_row[2][RESOLUTION];

RWStructuredBuffer<float2> fft_data : register(u0);

float2 complex_add(float2 c0, float2 c1) {
    return float2(c0.x+c1.x, c0.y+c1.y);
}

float2 complex_sub(float2 c0, float2 c1) {
    return float2(c0.x-c1.x, c0.y-c1.y);
}

float2 complex_mul(float2 c0, float2 c1) {
    return float2(c0.x*c1.x - c0.y*c1.y, c0.y*c1.x + c0.x*c1.y);
}

void butterfly(uint block_size, uint src, uint dst) {
    const uint index = gl_GlobalInvocationID.x;
    const uint k = index & (block_size - 1u);
    const uint offset = gl_GlobalInvocationID.y * num_threads * 2;

    const float2 in0 = shared_row[src][index];
    const float2 in1 = shared_row[src][index + num_threads];

    const float theta = pi * float(k) / float(block_size); // NOTE: not 2 * pi as stated in the paper!
    const float2 c = float2(cos(theta), sin(theta));
    const float2 temp = complex_mul(in1, c);

    const uint dest = (index << 1) - k;

    shared_row[dst][dest] = complex_add(in0, temp);
    shared_row[dst][dest + block_size] = complex_sub(in0, temp);
}

[numthreads(RESOLUTION, 1, 1)]
void fft_row() {
    const uint index = gl_GlobalInvocationID.x + RESOLUTION * gl_GlobalInvocationID.y;
    shared_row[0][gl_GlobalInvocationID.x] = fft_data[index];
    GroupMemoryBarrierWithGroupSync();

    for(uint i = 0; i < RESOLUTION_LOG; i++) {
        const uint block_size = 1 << i;
        const uint src = i % 2;
        const uint dst = (i+1) % 2;
        butterfly(block_size, src, dst);
        GroupMemoryBarrierWithGroupSync();
    }

    fft_data[index] = shared_row[1][gl_GlobalInvocationID.x]
}
