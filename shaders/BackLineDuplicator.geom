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
in gl_PerVertex { vec4 gl_Position; } gl_in[];
out gl_PerVertex { vec4 gl_Position; };

// CommonHeader
const int MAX_ENEMY_COUNT = 128;
const int MAX_BK_COUNT = 64;
struct Matrixes { mat4 ortho, pixel, persp; };
struct CharacterLocation { vec4 rotq[2], center_tf; };
struct BackgroundInstance { vec4 offset, scale; };
layout(std140, set = 0, binding = 0) uniform UniformMemory
{
	Matrixes projection_matrixes;
	CharacterLocation enemy_instance_data[MAX_ENEMY_COUNT];
	BackgroundInstance background_instance_data[MAX_BK_COUNT];
};

vec4 vertex_transform(vec4 base, vec4 displacement, vec4 scale)
{
	return (base * scale + displacement) * projection_matrixes.persp;
}

void main()
{
	if(instance_id[0] > 0)
	{
		BackgroundInstance instance_data = background_instance_data[instance_id[0] - 1];
		if(instance_data.offset.w > gl_InvocationID)
		{
			vec4 offsetter = vec4(0.0f, 0.0f, gl_InvocationID * 1.25f, 0.0f) + vec4(instance_data.offset.xyz, 0.0f);
			color = vec4(r, g, b, a);
			gl_Position = vertex_transform(gl_in[0].gl_Position, offsetter, instance_data.scale); EmitVertex();
			gl_Position = vertex_transform(gl_in[1].gl_Position, offsetter, instance_data.scale); EmitVertex();
			gl_Position = vertex_transform(gl_in[2].gl_Position, offsetter, instance_data.scale); EmitVertex();
			gl_Position = vertex_transform(gl_in[3].gl_Position, offsetter, instance_data.scale); EmitVertex();
			gl_Position = vertex_transform(gl_in[0].gl_Position, offsetter, instance_data.scale); EmitVertex();
			EndPrimitive();
		}
	}
}
