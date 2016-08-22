#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(lines_adjacency, invocations = 2) in;
layout(line_strip, max_vertices = 16) out;

layout(location = 0) in vec4 color_in[];
layout(location = 1) in uint instance_id[];
layout(location = 0) out vec4 color;
in gl_PerVertex { vec4 gl_Position; } gl_in[];
out gl_PerVertex { vec4 gl_Position; };

#include "UniformMemory.glsl"
#include "Quaternion.glsl"

vec4 vertex_transform(vec4 base, uint instance_index)
{
	vec4 q = enemy_instance_data[instance_index].rotq[gl_InvocationID];
	return (vec4(qRot(base.xyz, q).xyz, 1.0f) + enemy_instance_data[instance_index].center_tf) * projection_matrixes.ortho;
}

void main()
{
	if(instance_id[0] > 0)
	{
		color = color_in[0];
		gl_Position = vertex_transform(gl_in[0].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[1].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[2].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[3].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[0].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[0].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[1].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[2].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[3].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[0].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[1].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[1].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[2].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[2].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[3].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		gl_Position = vertex_transform(gl_in[3].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f), instance_id[0] - 1); EmitVertex();
		EndPrimitive();
	}
}
