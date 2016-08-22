#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(push_constant) uniform PushConstant
{
	vec4 color;
} pushed_values;

layout(location = 0) in vec4 pos;
layout(location = 1) in uint index_mult;
out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec4 color;
layout(location = 1) out uint instance_id;

void main()
{
	gl_Position = pos;
	color = pushed_values.color;
	instance_id = (gl_InstanceIndex + 1) * index_mult;
}
