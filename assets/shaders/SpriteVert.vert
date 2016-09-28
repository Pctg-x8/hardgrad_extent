#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos;
layout(location = 0) out vec4 uv;
out gl_PerVertex { vec4 gl_Position; };

void main()
{
	gl_Position = pos;
	uv = fma(pos, vec4(0.5f), vec4(0.5f));
}
