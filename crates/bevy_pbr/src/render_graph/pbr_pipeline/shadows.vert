#version 450

const int MAX_LIGHTS = 10;

layout(location = 0) in vec3 Vertex_Position;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

layout(push_constant) uniform CurrentLight {
    mat4 view_proj;
    int light_index;
    int face_index;
};

void main() {
    vec4 world_position = view_proj * Model * vec4(Vertex_Position, 1.0);
    gl_Position = world_position;
}
