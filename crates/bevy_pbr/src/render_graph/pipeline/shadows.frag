#version 450

const int MAX_LIGHTS = 10;

struct Light {
    mat4 proj;
    vec3 pos;
    float inverseRadiusSquared;
    vec3 color;
    float unused; // unused 4th element of vec4;
};

layout(location = 0) in vec4 FragPos;

layout(set = 1, binding = 0) uniform Lights {
    vec3 AmbientColor;
    uvec4 NumLights;
    Light SceneLights[MAX_LIGHTS];
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    Light light = SceneLights[0];

    // get distance between fragment and light source
    float lightDistance = length(FragPos.xyz - light.pos);

    // see https://www.gamedevs.org/uploads/fast-extraction-viewing-frustum-planes-from-world-view-projection-matrix.pdf
    float far = light.proj[3][3] - light.proj[2][3];
    // map to [0;1] range by dividing by far_plane
    lightDistance = lightDistance / far;

    // write this as modified depth
    gl_FragDepth = lightDistance;
}
