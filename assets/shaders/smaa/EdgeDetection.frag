#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 uv;
layout(location = 1) in vec4 offsets[3];
layout(location = 0) out vec2 target;

#include "common_setup.glsl"
layout(set = 0, binding = 0) uniform sampler2D input_tex;

void main()
{
	// Calculate the threshold
	vec2 threshold = vec2(SMAA_THRES, SMAA_THRES);

	// Calculate lumas
	vec3 weights = vec3(0.2126, 0.7152, 0.0722);
	float l = dot(texture(input_tex, uv.xy).rgb, weights);
	float l_left = dot(texture(input_tex, offsets[0].xy).rgb, weights);
	float l_top = dot(texture(input_tex, offsets[0].zw).rgb, weights);

	// We do the usual threshold
	vec4 delta;
	delta.xy = abs(l - vec2(l_left, l_top));
	vec2 edges = step(threshold, delta.xy);
	// Discard pixel if there is no edge
	if(dot(edges, vec2(1.0f, 1.0f)) == 0.0f) discard;

	// Calculate right and bottom deltas
	float l_right = dot(texture(input_tex, offsets[1].xy).rgb, weights);
	float l_bottom = dot(texture(input_tex, offsets[1].zw).rgb, weights);
	delta.zw = abs(l - vec2(l_right, l_bottom));

	// Calculate the maximum delta in the direct neighborhood
	vec2 max_delta = max(delta.xy, delta.zw);

	// Calculate left-left and top-top deltas
	float l_left_left = dot(texture(input_tex, offsets[2].xy).rgb, weights);
	float l_top_top = dot(texture(input_tex, offsets[2].zw).rgb, weights);
	delta.zw = abs(vec2(l_left, l_top) - vec2(l_left_left, l_top_top));
	
	// Calculate the final maximum delta
	max_delta = max(max_delta.xy, delta.zw);
	float final_delta = max(max_delta.x, max_delta.y);

	// Local contrast adaption
	edges.xy *= step(final_delta, SMAA_LOCAL_CONTRAST_ADAPTION_FACTOR * delta.xy);

	target = edges;
}
