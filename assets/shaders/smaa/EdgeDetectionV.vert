#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 pos_uv;
out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec4 uv;
layout(location = 1) out vec4 offsets[3];

#include "common_setup.glsl"

void main()
{
	gl_Position = vec4(pos_uv.xy, 0.0f, 1.0f);
	uv = vec4(pos_uv.zw, 0.0f, 0.0f);
	offsets[0] = mad(rt_metrics.xyxy, vec4(-1.0f, 0.0f, 0.0f, -1.0f), uv.xyxy);
	offsets[1] = mad(rt_metrics.xyxy, vec4(1.0f, 0.0f, 0.0f, 1.0f), uv.xyxy);
	offsets[2] = mad(rt_metrics.xyxy, vec4(-2.0f, 0.0f, 0.0f, -2.0f), uv.xyxy);
}
