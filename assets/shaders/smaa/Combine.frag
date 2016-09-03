#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 uv;
layout(location = 1) in vec4 offset;
layout(set = 0, binding = 0) uniform sampler2D intex[2];	// color, blend
layout(location = 0) out vec4 target;

layout(push_constant) uniform RenderTargetDesc
{
	vec4 metrics;
} rtdesc;

#include "common_setup.glsl"

void main()
{
	// Fetch the blending weights for current pixel
	vec4 a;
	a.x = texture(intex[1], offset.xy).a;
	a.y = texture(intex[1], offset.zw).g;
	a.wz = texture(intex[1], uv.xy).xz;

	// Is there any blending weight with a value greates than 0.0?
	if(dot(a, vec4(1.0f, 1.0f, 1.0f, 1.0f)) < 1e-5)
	{
		vec4 color = textureLod(intex[0], uv.xy, 0.0f);

		target = color;
	}
	else
	{
		bool h = max(a.x, a.z) > max(a.y, a.w);

		// Calculate the blending offsets
		vec4 blendingOffset = h ? vec4(a.x, 0.0f, a.z, 0.0f) : vec4(0.0f, a.y, 0.0f, a.w);
		vec2 blendingWeight = h ? a.xz : a.yw;
		blendingWeight /= dot(blendingWeight, vec2(1.0f, 1.0f));

		// Calculate the texture coordinates
		vec4 blendingCoord = mad(blendingOffset, vec4(rtdesc.metrics.xy, -rtdesc.metrics.xy), uv.xyxy);
		vec4 color = blendingWeight.x * textureLod(intex[0], blendingCoord.xy, 0.0f);
		color += blendingWeight.y * textureLod(intex[0], blendingCoord.zw, 0.0f);
		target = color;
	}
}
