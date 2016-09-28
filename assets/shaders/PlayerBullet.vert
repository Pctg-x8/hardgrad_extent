#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos;
layout(location = 1) in vec4 offs_sincos;
layout(location = 0) out vec4 uv;
out gl_PerVertex { vec4 gl_Position; };

#include "UniformMemory.glsl"
layout(constant_id = 0) const float SpriteScaling = 1.0f;

void main()
{
	const mat4 local_transform = mat4(
		vec4(offs_sincos.w, -offs_sincos.z, 0.0f, offs_sincos.x),
		vec4(offs_sincos.z,  offs_sincos.w, 0.0f, offs_sincos.y),
		vec4(0.0f, 0.0f, 1.0f, 0.0f),
		vec4(0.0f, 0.0f, 0.0f, 1.0f)
	);
	gl_Position = pos * vec4(SpriteScaling, SpriteScaling, 1.0f, 1.0f)
		* local_transform * projection_matrixes.ortho;
	uv = fma(pos, vec4(0.5f), vec4(0.5f));
}
