#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(lines_adjacency, invocations = 10) in;
layout(line_strip, max_vertices = 5) out;
layout(constant_id = 10) const float r = 0.0f;
layout(constant_id = 11) const float g = 0.0f;
layout(constant_id = 12) const float b = 0.0f;
layout(constant_id = 13) const float a = 0.0f;

layout(location = 0) in uint instance_id[];
layout(location = 0) out vec4 color;

const int MAX_BK_COUNT = 64;

layout(std140, set = 0, binding = 0) uniform ProjectionMatrix
{
	mat4 ortho_projection_matrix, pixel_projection_matrix, persp_projection_matrix;
};
layout(std140, binding = 0, set = 1) uniform BackgroundInststancingParams
{
	vec4 offset[MAX_BK_COUNT];
};

vec4 vertex_transform(vec4 base, vec4 displacement)
{
	return (base + displacement)/* * persp_projection_matrix*/;
}

void main()
{
	if(instance_id[0] > 0)
	{
		vec4 offset_layers = offset[instance_id[0] - 1];
		if(offset_layers.w >= gl_InvocationID)
		{
			vec4 offsetter = vec4(0.0f, 0.0f, gl_InvocationID, 0.0f) + vec4(offset_layers.xyz, 0.0f);
			color = vec4(1.0f, 1.0f, 1.0f, 1.0f);
			gl_Position = vertex_transform(gl_in[0].gl_Position, offsetter); EmitVertex();
			gl_Position = vertex_transform(gl_in[1].gl_Position, offsetter); EmitVertex();
			gl_Position = vertex_transform(gl_in[2].gl_Position, offsetter); EmitVertex();
			gl_Position = vertex_transform(gl_in[3].gl_Position, offsetter); EmitVertex();
			gl_Position = vertex_transform(gl_in[0].gl_Position, offsetter); EmitVertex();
			EndPrimitive();
		}
	}
}
