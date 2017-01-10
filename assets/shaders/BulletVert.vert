#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos;
layout(location = 1) in vec2 lifetime;
layout(location = 0) out vec4 uv;
layout(location = 1) out vec4 color;
out gl_PerVertex { vec4 gl_Position; };

#include "UniformMemory.glsl"
layout(set = 2, binding = 0) uniform sampler1D color_ramp;
layout(constant_id = 0) const float SpriteScaling = 1.0f;

void main()
{
	gl_Position = fma(pos, vec4(SpriteScaling, SpriteScaling, 1.0f, 1.0f), bullet_translations[gl_InstanceIndex]) * projection_matrixes.ortho * step(1.0f, lifetime.y);
	uv = fma(pos, vec4(0.5f), vec4(0.5f));
	color = texture(color_ramp, clamp(lifetime.x, 0.0f, 1.0f));
}
