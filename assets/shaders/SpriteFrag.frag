#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 uv;
layout(location = 0) out vec4 target;
layout(set = 1, binding = 0) uniform sampler2D sprite;

void main()
{
	target = texture(sprite, uv.xy);
}
