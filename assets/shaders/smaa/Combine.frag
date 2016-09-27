#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 uv;
layout(location = 1) in vec4 offset;
layout(location = 0) out vec4 target;

#include "common_setup.glsl"
layout(set = 0, binding = 0) uniform sampler2D intex[2];
#define input_tex intex[0]
#define blend_tex intex[1]

void main()
{
	// Fetch the blending weights for current pixel
	vec4 a;
	a.x = texture(blend_tex, offset.xy).a;
	a.y = texture(blend_tex, offset.zw).g;
	a.wz = texture(blend_tex, uv.xy).xz;

	// Is there any blending weight with a value greates than 0.0?
	if(dot(a, vec4(1.0f)) < 1e-5)
	{
		target = textureLod(input_tex, uv.xy, 0.0f);
	}
	else
	{
		bool h = max(a.x, a.z) > max(a.y, a.w);

		// Calculate the blending offsets
		vec4 blendingOffset = h ? vec4(a.x, 0.0f, a.z, 0.0f) : vec4(0.0f, a.y, 0.0f, a.w);
		vec2 blendingWeight = h ? a.xz : a.yw;
		blendingWeight /= dot(blendingWeight, vec2(1.0f, 1.0f));

		// Calculate the texture coordinates
		vec4 blendingCoord = mad(blendingOffset, vec4(rt_metrics.xy, -rt_metrics.xy), uv.xyxy);
		target =
			blendingWeight.x * textureLod(input_tex, blendingCoord.xy, 0.0f) +
			blendingWeight.y * textureLod(input_tex, blendingCoord.zw, 0.0f);
	}
}
