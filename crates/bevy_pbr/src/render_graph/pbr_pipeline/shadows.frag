#version 450

const int MAX_LIGHTS = 10;

struct PointLight {
    vec3 pos;
    float near;
    vec3 color;
    float far;
};

layout(location = 0) in vec3 v_WorldPosition;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Lights {
    vec3 AmbientColor;
    uvec4 NumLights;
    PointLight PointLights[MAX_LIGHTS];
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
    PointLight light = PointLights[light_index];

    // get distance between fragment and light source
    float lightDistance = length(v_WorldPosition - light.pos);

    // map to [0;1] range
    lightDistance = (lightDistance - light.near) / (light.far - light.near);

    // write this as modified depth
    gl_FragDepth = lightDistance;
}
