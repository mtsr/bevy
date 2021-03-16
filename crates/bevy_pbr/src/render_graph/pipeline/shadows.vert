#version 450

const int MAX_LIGHTS = 10;

struct Light {
    mat4 proj;
    vec3 pos;
    float inverseRadiusSquared;
    vec3 color;
    float unused; // unused 4th element of vec4;
};

layout(location = 0) in vec3 Vertex_Position;

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

layout(set = 1, binding = 0) uniform Lights {
    vec3 AmbientColor;
    uvec4 NumLights;
    Light SceneLights[MAX_LIGHTS];
};

void main() {
    Light light = SceneLights[0];

    vec4 world_position = light.proj * Model * vec4(Vertex_Position, 1.0);
    gl_Position = world_position;
}
