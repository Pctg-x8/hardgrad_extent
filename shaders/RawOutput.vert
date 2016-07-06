#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos;
layout(constant_id = 10) const float r = 0.0f;
layout(constant_id = 11) const float g = 0.0f;
layout(constant_id = 12) const float b = 0.0f;
layout(constant_id = 13) const float a = 0.0f;

layout(set = 0, binding = 0) uniform ProjectionMatrix
{
	mat4 ortho_projection_matrix, persp_projection_matrix;
};

layout(location = 0) out vec4 color_out;
out gl_PerVertex { vec4 gl_Position; };

void main()
{
	gl_Position = pos * ortho_projection_matrix;
	color_out = vec4(r, g, b, a);
}
