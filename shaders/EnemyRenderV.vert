#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos;
layout(location = 1) in uint character_index_mult;

layout(location = 0) out uint instance_index;


void main()
{
	gl_Position = pos; instance_index = character_index_mult * (gl_InstanceIndex + 1);
}
