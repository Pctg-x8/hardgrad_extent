#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(push_constant) uniform PushConstant
{
	vec4 color;
} pushed_values;

layout(location = 0) in vec4 pos;
layout(location = 1) in vec4 in_instance_data;
out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec4 color;
layout(location = 1) out vec4 instance_data;

void main()
{
	gl_Position = pos;
	color = pushed_values.color;
	instance_data = vec4(in_instance_data.xyz, gl_InstanceIndex);
}
