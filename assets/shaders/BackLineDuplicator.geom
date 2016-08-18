#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(lines_adjacency, invocations = 10) in;
layout(line_strip, max_vertices = 5) out;
layout(push_constant) uniform PushConstant
{
	vec4 color;
} pushed_values;

layout(location = 0) in uint instance_id[];
layout(location = 0) out vec4 color;
in gl_PerVertex { vec4 gl_Position; } gl_in[];
out gl_PerVertex { vec4 gl_Position; };

#include "UniformMemory.glsl"

vec4 vertex_transform(vec4 base, vec4 displacement, vec4 scale)
{
	return (base * scale + displacement) * projection_matrixes.persp;
}

void main()
{
	if(instance_id[0] > 0)
	{
		vec4 instance_offset = background_instance_data[instance_id[0] - 1].offset;
		vec4 instance_scale = background_instance_data[instance_id[0] - 1].scale;
		if(instance_offset.w > gl_InvocationID)
		{
			vec4 offsetter = vec4(0.0f, 0.0f, gl_InvocationID * 1.25f, 0.0f) + vec4(instance_offset.xyz, 0.0f);
			color = pushed_values.color;
			gl_Position = vertex_transform(gl_in[0].gl_Position, offsetter, instance_scale); EmitVertex();
			gl_Position = vertex_transform(gl_in[1].gl_Position, offsetter, instance_scale); EmitVertex();
			gl_Position = vertex_transform(gl_in[2].gl_Position, offsetter, instance_scale); EmitVertex();
			gl_Position = vertex_transform(gl_in[3].gl_Position, offsetter, instance_scale); EmitVertex();
			gl_Position = vertex_transform(gl_in[0].gl_Position, offsetter, instance_scale); EmitVertex();
			EndPrimitive();
		}
	}
}
