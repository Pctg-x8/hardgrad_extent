#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(constant_id = 10) const float r = 0.0f;
layout(constant_id = 11) const float g = 0.0f;
layout(constant_id = 12) const float b = 0.0f;
layout(constant_id = 13) const float a = 0.0f;

layout(location = 0) in vec4 pos;
layout(location = 1) in vec4 qrot;
layout(location = 0) out vec4 color;
out gl_PerVertex { vec4 gl_Position; };

#include "UniformMemory.glsl"
#include "Quaternion.glsl"

void main()
{
	color = vec4(r, g, b, a);
	gl_Position = (vec4(qRot(pos.xyz, qrot).xyz, 1.0f) + player_center_tf) * projection_matrixes.ortho;
}
