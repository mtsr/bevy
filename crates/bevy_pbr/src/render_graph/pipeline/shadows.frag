#version 450

const int MAX_LIGHTS = 10;

layout(location = 0) in vec4 FragPos;

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
    // get distance between fragment and light source
    float lightDistance = length(FragPos.xyz - pos);

    // see https://www.gamedevs.org/uploads/fast-extraction-viewing-frustum-planes-from-world-view-projection-matrix.pdf
    float far = proj[3][3] - proj[2][3];
    // map to [0;1] range by dividing by far_plane
    lightDistance = lightDistance / far;

    // write this as modified depth
    gl_FragDepth = lightDistance;
}
