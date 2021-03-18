#version 450

const int MAX_LIGHTS = 10;

struct PointLight {
    mat4 proj;
    vec3 pos;
    float inverseRadiusSquared;
    vec3 color;
    // float unused; // unused 4th element of vec4;
};

layout(location = 0) in vec3 Vertex_Position;

// layout(set = 0, binding = 0) uniform Camera {
//     mat4 ViewProj;
//     vec4 CameraPos;
// };

layout(set = 1, binding = 0) uniform Lights {
    vec3 AmbientColor;
    uvec4 NumLights;
    PointLight PointLights[MAX_LIGHTS];
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

layout(push_constant) uniform push_constants {
    int light_index;
};

void main() {
    PointLight light = PointLights[0];

    vec4 world_position = light.proj * Model * vec4(Vertex_Position, 1.0);
    gl_Position = world_position;
}
