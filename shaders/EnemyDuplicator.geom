#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(lines_adjacency, invocations = 2) in;
layout(line_strip, max_vertices = 16) out;
layout(constant_id = 10) const float r = 0.0f;
layout(constant_id = 11) const float g = 0.0f;
layout(constant_id = 12) const float b = 0.0f;
layout(constant_id = 13) const float a = 0.0f;

layout(location = 0) in uint instance_id[];
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
		color = vec4(r, g, b, a);
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
