#version 450

const int MAX_LIGHTS = 10;

layout(location = 0) in vec3 Vertex_Position;

// layout(set = 0, binding = 0) uniform Camera {
//     mat4 ViewProj;
//     vec4 CameraPos;
// };

layout(set = 1, binding = 0) uniform SingleLight {
    mat4 proj;
    vec3 pos;
    float inverseRadiusSquared;
    vec3 color;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    vec4 world_position = proj * Model * vec4(Vertex_Position, 1.0);
    gl_Position = world_position;
}
