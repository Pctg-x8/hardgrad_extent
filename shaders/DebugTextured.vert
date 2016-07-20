#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos;
layout(location = 1) in vec4 uv;
layout(location = 2) in vec4 xy_uv_scale;
layout(location = 3) in vec4 xy_uv_offset;

layout(constant_id = 10) const float r = 0.0f;
layout(constant_id = 11) const float g = 0.0f;
layout(constant_id = 12) const float b = 0.0f;

layout(std140, set = 0, binding = 0) uniform ProjectionMatrix
{
	mat4 ortho_projection_matrix, pixel_projection_matrix, persp_projection_matrix;
};

layout(location = 0) out vec4 color_out;
layout(location = 1) out vec2 uv_out;
out gl_PerVertex { vec4 gl_Position; };

void main()
{
	vec4 xy_uv = vec4(pos.xy, uv.xy) * xy_uv_scale + xy_uv_offset;
	gl_Position = vec4(xy_uv.xy, pos.zw) * pixel_projection_matrix;
	color_out = vec4(r, g, b, 1.0f);
	uv_out = xy_uv.zw;
}
