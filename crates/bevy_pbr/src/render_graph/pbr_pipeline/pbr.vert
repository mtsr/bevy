#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 2) in vec2 Vertex_Uv;
layout(location = 1) in vec3 Vertex_Normal;

#ifdef STANDARDMATERIAL_NORMAL_MAP
layout(location = 3) in vec4 Vertex_Tangent;
#endif

layout(location = 0) out vec3 v_WorldPosition;
layout(location = 2) out vec2 v_Uv;

#ifndef STANDARDMATERIAL_NORMAL_MAP
layout(location = 1) out vec3 v_WorldNormal;
#else
layout(location = 3) out mat3 v_TBN;
#endif

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 ModelMatrix;
};

void main() {
    vec4 world_position = ModelMatrix * vec4(Vertex_Position, 1.0);
    v_WorldPosition = world_position.xyz / world_position.w;

    v_Uv = Vertex_Uv;

    // TODO transpose-inverse on CPU
    mat4 NormalMatrix = transpose(inverse(ModelMatrix));
#ifndef STANDARDMATERIAL_NORMAL_MAP
    v_WorldNormal = normalize(vec3(NormalMatrix * vec4(Vertex_Normal, 0.0)));
#else
    vec3 WorldNormal = normalize(vec3(NormalMatrix * vec4(Vertex_Normal.xyz, 0.0)));
    vec3 WorldTangent = normalize(vec3(NormalMatrix * vec4(Vertex_Tangent.xyz, 0.0)));
    vec3 WorldBiTangent = cross(WorldNormal, WorldTangent) * Vertex_Tangent.w;
    v_TBN = mat3(WorldTangent, WorldBiTangent, WorldNormal);
#endif

    gl_Position = ViewProj * world_position;
}
