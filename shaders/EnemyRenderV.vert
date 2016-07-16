#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

const uint MAX_ENEMY_COUNT = 128;
layout(std140, set = 1, binding = 0) uniform CharacterLocation
{
	vec4 rotq[(MAX_ENEMY_COUNT + 1) * 2];
	vec4 center_tf[MAX_ENEMY_COUNT + 1];
};

layout(location = 0) in vec4 pos;
layout(location = 1) in uint character_index_mult;
layout(constant_id = 10) const float r = 0.0f;
layout(constant_id = 11) const float g = 0.0f;
layout(constant_id = 12) const float b = 0.0f;
layout(constant_id = 13) const float a = 0.0f;

layout(std140, set = 0, binding = 0) uniform ProjectionMatrix
{
	mat4 ortho_projection_matrix, pixel_projection_matrix, persp_projection_matrix;
};
layout(push_constant) uniform PushedConstants
{
	uint transform_pass;
} pushed_constants;

layout(location = 0) out vec4 color_out;
out gl_PerVertex { vec4 gl_Position; };

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

void main()
{
	uint index = (gl_InstanceIndex + 1) * character_index_mult;
	vec4 rq = rotq[index + (MAX_ENEMY_COUNT + 1) * pushed_constants.transform_pass];
	vec4 ctf = center_tf[index];
	gl_Position = (vec4(qRot(pos.xyz, rq).xyz, 1.0f) + ctf) * ortho_projection_matrix;
	color_out = vec4(r, g, b, a);
}
