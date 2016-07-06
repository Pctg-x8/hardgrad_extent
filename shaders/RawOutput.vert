#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos;
layout(constant_id = 20) const float r = 0.0f;
layout(constant_id = 21) const float g = 0.0f;
layout(constant_id = 22) const float b = 0.0f;
layout(constant_id = 23) const float a = 0.0f;

layout(location = 0) out vec4 color_out;
out gl_PerVertex { vec4 gl_Position; };

void main()
{
	gl_Position = vec4(pos.xyz * 0.5f, 1.0f);
	color_out = /*vec4(0.25f, 0.9875f, 1.5f, 1.0f)*/vec4(r, g, b, 1.0f);
}

