#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(location = 0) in vec4 uv_pixcoord;
layout(location = 1) in vec4 offsets[3];
layout(location = 0) out vec4 target;

#include "common_setup.glsl"
layout(set = 0, binding = 0) uniform sampler2D intex[3];	// edges, area, search
#define edge_tex intex[0]
#define area_tex intex[1]
#define search_tex intex[2]

// conditional move //
void movc(bvec2 cond, inout vec2 variable, vec2 value)
{
	if(cond.x) variable.x = value.x;
	if(cond.y) variable.y = value.y;
}
void movc(bvec4 cond, inout vec4 variable, vec4 value)
{
	movc(cond.xy, variable.xy, value.xy);
	movc(cond.zw, variable.zw, value.zw);
}

// Allows to decode two binary values from a bilinear-filtered access
vec2 decode_diag_bilinear_access(vec2 edge)
{
	edge.r = edge.r * abs(5.0f * edge.r - 5.0f * 0.75f);
	return round(edge);
}
// decodeDiagBilinearAccess2 Edge = round $ applyVec2 morph id Edge
//	where morph x = x * abs $ 5.0f * x - 5.0f * 0.75f
vec4 decode_diag_bilinear_access(vec4 edge)
{
	edge.rb = edge.rb * abs(5.0f * edge.rb - 5.0f * 0.75f);
	return round(edge);
}
// decodeDiagBilinearAccess4 Edge = round $ applyVec4 morph id morph id Edge
//	where morph x = x * abs $ 5.0f * x - 5.0f * 0.75f
vec2 search_diag1(vec2 uv, vec2 dir, out vec2 edge)
{
	vec4 coord = vec4(uv, -1.0f, 1.0f);
	vec3 t = vec3(rt_metrics.xy, 1.0f);
	while(coord.z < float(SMAA_MAX_SEARCH_STEPS_DIAG - 1) && coord.w > 0.9f)
	{
		coord.xyz = fma(t, vec3(dir, 1.0f), coord.xyz);
		edge = textureLod(edge_tex, coord.xy, 0.0f).rg;
		coord.w = dot(edge, vec2(0.5f));
	}
	return coord.zw;
}
vec2 search_diag2(vec2 uv, vec2 dir, out vec2 edge)
{
	vec4 coord = vec4(uv, -1.0f, 1.0f);
	coord.x += 0.25f * rt_metrics.x;
	vec3 t = vec3(rt_metrics.xy, 1.0f);
	while(coord.z < float(SMAA_MAX_SEARCH_STEPS_DIAG - 1) && coord.w > 0.9f)
	{
		coord.xyz = fma(t, vec3(dir, 1.0f), coord.xyz);
		// optimized version
		edge = textureLod(edge_tex, coord.xy, 0.0f).rg;
		edge = decode_diag_bilinear_access(edge);
		coord.w = dot(edge, vec2(0.5f));
	}
	return coord.zw;
}
vec2 area_diag(vec2 dist, vec2 e, float offset)
{
	vec2 texcoord = fma(vec2(SMAA_AREATEX_MAX_DISTANCE_DIAG), e, dist);
	texcoord = fma(SMAA_AREATEX_PIXEL_SIZE, texcoord, 0.5f * SMAA_AREATEX_PIXEL_SIZE);
	texcoord.x += 0.5f;
	texcoord.y += SMAA_AREATEX_SUBTEX_SIZE * offset;
	return areatex_select(textureLod(area_tex, texcoord, 0.0f));
}
vec2 calculate_diag_weights(vec2 texcoord, vec2 edge)
{
	vec2 weights = vec2(0.0f);

	// Search for the line ends
	vec4 d; vec2 end;
	if(edge.r > 0.0f)
	{
		d.xz = search_diag1(texcoord, vec2(-1.0f, 1.0f), end);
		d.x += float(end.y > 0.9f);
	}
	else d.xz = vec2(0.0f);
	d.yw = search_diag1(texcoord, vec2(1.0f, -1.0f), end);

	if(d.x + d.y > 2.0f)
	{
		// Fetch the crossing edges
		vec4 coords = fma(vec4(-d.x + 0.25f, d.x, d.y, -d.y - 0.25f), rt_metrics.xyxy, texcoord.xyxy);
		vec4 c = vec4(
			textureLodOffset(edge_tex, coords.xy, 0.0f, ivec2(-1, 0)).rg,
			textureLodOffset(edge_tex, coords.zw, 0.0f, ivec2( 1, 0)).rg
		);
		c.yxwz = decode_diag_bilinear_access(c.xyzw);

		// Merge crossing edges at each side into a single value
		vec2 cc = fma(vec2(2.0f), c.xz, c.yw);
		// Remove the crossing edge if we didn't found the end of the line
		cc *= 1.0f - step(0.9f, d.zw);
		// Fetch the areas for this line
		weights += area_diag(d.xy, cc, 0.0f);
	}

	// Search for the line ends
	d.xz = search_diag2(texcoord, vec2(-1.0f), end);
	if(textureLodOffset(edge_tex, texcoord, 0.0f, ivec2(1, 0)).r > 0.0f)
	{
		d.yw = search_diag2(texcoord, vec2(1.0f), end);
		d.y += float(end.y > 0.9f);
	}
	else d.yw = vec2(0.0f);

	if(d.x + d.y > 2.0f)
	{
		// Fetch the crossing edges
		vec4 coords = fma(vec4(-d.x, -d.x, d.y, d.y), rt_metrics.xyxy, texcoord.xyxy);
		vec4 c = vec4(
			textureLodOffset(edge_tex, coords.xy, 0.0f, ivec2(-1,  0)).g,
			textureLodOffset(edge_tex, coords.xy, 0.0f, ivec2( 0, -1)).r,
			textureLodOffset(edge_tex, coords.zw, 0.0f, ivec2( 1,  0)).gr
		);
		vec2 cc = fma(vec2(2.0f), c.xz, c.yw);

		// Remove the crossing edge if we didn't found the end of the line
		cc *= 1.0f - step(0.9f, d.zw);
		weights += area_diag(d.xy, cc, 0.0f).gr;
	}

	return weights;
}

// searching //
float search_length(vec2 edge, float offset)
{
	vec2 scale = SMAA_SEARCHTEX_SIZE * vec2(0.5f, -1.0f);
	vec2 bias = SMAA_SEARCHTEX_SIZE * vec2(offset, 1.0f);

	// Scale and bias to access texel centers
	scale += vec2(-1.0f, 1.0f);
	bias += vec2(0.5f, -0.5f);

	// Convert from pixel coordinates to texcoords
	scale *= 1.0f / SMAA_SEARCHTEX_PACKED_SIZE;
	bias *= 1.0f / SMAA_SEARCHTEX_PACKED_SIZE;

	// Lookup the search texture
	return searchtex_select(textureLod(search_tex, fma(scale, edge, bias), 0.0f));
}
float search_x_left(vec2 texcoord, float end)
{
	vec2 e = vec2(0.0f, 1.0f);
	while(texcoord.x > end && e.g > 0.8281f && e.r == 0.0f)
	{
		e = textureLod(edge_tex, texcoord, 0.0f).rg;
		texcoord = mad(-vec2(2.0f, 0.0f), rt_metrics.xy, texcoord);
	}

	float offset = mad(-(255.0f / 127.0f), search_length(e, 0.0f), 3.25f);
	return mad(rt_metrics.x, offset, texcoord.x);
}
float search_x_right(vec2 texcoord, float end)
{
	vec2 e = vec2(0.0f, 1.0f);
	while(texcoord.x < end && e.g > 0.8281f && e.r == 0.0f)
	{
		e = textureLod(edge_tex, texcoord, 0.0f).rg;
		texcoord = mad(vec2(2.0f, 0.0f), rt_metrics.xy, texcoord);
	}
	float offset = mad(-(255.0 / 127.0f), search_length(e, 0.5f), 3.25f);
	return mad(-rt_metrics.x, offset, texcoord.x);
}
float search_y_up(vec2 texcoord, float end)
{
	vec2 e = vec2(1.0f, 0.0f);
	while(texcoord.y > end && e.r > 0.8281f && e.g == 0.0f)
	{
		e = textureLod(edge_tex, texcoord, 0.0f).rg;
		texcoord = mad(-vec2(0.0f, 2.0f), rt_metrics.xy, texcoord);
	}
	float offset = mad(-(255.0f / 127.0f), search_length(e.gr, 0.0f), 3.25f);
	return mad(rt_metrics.y, offset, texcoord.y);
}
float search_y_down(vec2 texcoord, float end)
{
	vec2 e = vec2(1.0f, 0.0f);
	while(texcoord.y < end && e.r > 0.8281f && e.g == 0.0f)
	{
		e = textureLod(edge_tex, texcoord, 0.0f).rg;
		texcoord = mad(vec2(0.0f, 2.0f), rt_metrics.xy, texcoord);
	}
	float offset = mad(-(255.0f / 127.0f), search_length(e.gr, 0.5f), 3.25f);
	return mad(-rt_metrics.y, offset, texcoord.y);
}
vec2 area(vec2 dist, float e1, float e2, float offset)
{
	vec2 texcoord = mad(vec2(SMAA_AREATEX_MAX_DISTANCE, SMAA_AREATEX_MAX_DISTANCE), round(4.0f * vec2(e1, e2)), dist);
	texcoord = mad(SMAA_AREATEX_PIXEL_SIZE, texcoord, 0.5f * SMAA_AREATEX_PIXEL_SIZE);
	texcoord.y = mad(SMAA_AREATEX_SUBTEX_SIZE, offset, texcoord.y);
	return areatex_select(textureLod(area_tex, texcoord, 0.0f));
}

// corner detection //
void detect_horizontal_corner_pattern(inout vec2 weights, vec4 texcoord, vec2 d)
{
	vec2 left_right = step(d.xy, d.yx);
	vec2 rounding = (1.0f - SMAA_CORNER_ROUNDING_NORM) * left_right;
	rounding /= left_right.x + left_right.y;
	
	vec2 factor = vec2(1.0f, 1.0f);
	factor.x -= rounding.x * textureLodOffset(edge_tex, texcoord.xy, 0.0f, ivec2(0, 1)).r;
	factor.x -= rounding.y * textureLodOffset(edge_tex, texcoord.zw, 0.0f, ivec2(1, 1)).r;
	factor.y -= rounding.x * textureLodOffset(edge_tex, texcoord.xy, 0.0f, ivec2(0, -2)).r;
	factor.y -= rounding.y * textureLodOffset(edge_tex, texcoord.zw, 0.0f, ivec2(1, -2)).r;

	weights *= clamp(factor, 0.0f, 1.0f);
}
void detect_vertical_corner_pattern(inout vec2 weights, vec4 texcoord, vec2 d)
{
	vec2 left_right = step(d.xy, d.yx);
	vec2 rounding = (1.0f - SMAA_CORNER_ROUNDING_NORM) * left_right;
	rounding /= left_right.x + left_right.y;

	vec2 factor = vec2(1.0f, 1.0f);
	factor.x -= rounding.x * textureLodOffset(edge_tex, texcoord.xy, 0.0f, ivec2(1, 0)).g;
	factor.x -= rounding.y * textureLodOffset(edge_tex, texcoord.zw, 0.0f, ivec2(1, 1)).g;
	factor.y -= rounding.x * textureLodOffset(edge_tex, texcoord.xy, 0.0f, ivec2(-2, 0)).g;
	factor.y -= rounding.y * textureLodOffset(edge_tex, texcoord.zw, 0.0f, ivec2(-2, 1)).g;

	weights *= clamp(factor, 0.0f, 1.0f);
}

void main()
{
	vec4 weights = vec4(0.0f, 0.0f, 0.0f, 0.0f);
	vec2 e = texture(edge_tex, uv_pixcoord.xy).rg;

	if(e.g > 0.0f)
	{
		// edge at north
		weights.rg = calculate_diag_weights(uv_pixcoord.xy, e);
		if(weights.r == -weights.g)
		{
			vec2 d; vec3 coords;

			// Find the distance to the left
			coords.x = search_x_left(offsets[0].xy, offsets[2].x);
			coords.y = offsets[1].y;
			d.x = coords.x;

			float e1 = textureLod(edge_tex, coords.xy, 0.0f).r;

			coords.z = search_x_right(offsets[0].zw, offsets[2].y);
			d.y = coords.z;

			d = abs(round(mad(rt_metrics.zz, d, -uv_pixcoord.zz)));
			vec2 sqrt_d = sqrt(d);
			float e2 = textureLodOffset(edge_tex, coords.zy, 0.0f, ivec2(1, 0)).r;
			weights.rg = area(sqrt_d, e1, e2, 0.0f);
			coords.y = uv_pixcoord.y;
			detect_horizontal_corner_pattern(weights.rg, coords.xyzy, d);
		}
		else e.r = 0.0f;	// skip vertical processing
	}

	if(e.r > 0.0f)
	{
		// edge at west
		vec2 d; vec3 coords;

		// Find the distance to the top
		coords.y = search_y_up(offsets[1].xy, offsets[2].z);
		coords.x = offsets[0].x;
		d.x = coords.y;

		float e1 = textureLod(edge_tex, coords.xy, 0.0f).g;
		coords.z = search_y_down(offsets[1].zw, offsets[2].w);
		d.y = coords.z;

		d = abs(round(mad(rt_metrics.ww, d, -uv_pixcoord.ww)));
		vec2 sqrt_d = sqrt(d);

		float e2 = textureLodOffset(edge_tex, coords.xz, 0.0f, ivec2(0, 1)).g;
		weights.ba = area(sqrt_d, e1, e2, 0.0f);
		coords.x = uv_pixcoord.x;
		detect_vertical_corner_pattern(weights.ba, coords.xyxz, d);
	}
	target = weights;
}
