#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos_uv;
out gl_PerVertex { vec4 gl_Position; };

void main()
{
	gl_Position = vec4(pos_uv.xy, 0.0f, 1.0f);
}
