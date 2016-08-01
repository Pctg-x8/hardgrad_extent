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

vec4 qConjugate(vec4 iq) { return vec4(-iq.xyz, iq.w); }
vec4 qMult(vec4 q1, vec4 q2) { return vec4(cross(q1.xyz, q2.xyz) + q2.w * q1.xyz + q1.w * q2.xyz, q1.w * q2.w - dot(q1.xyz, q2.xyz)); }
vec4 qRot(vec3 in_vec, vec4 rq)
{
    vec4 q1 = rq;
    vec4 qp = vec4(in_vec, 0.0f);
    vec4 q2 = qConjugate(rq);
    vec4 qt = qMult(q1, qp);
    return qMult(qt, q2);
}
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
