#version 450

layout(location = 0) in vec4 source_pos;
out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec4 color;
layout(push_constant) uniform PConst { float direction; } pushed_values;
#include "UniformMemory.glsl"

void main()
{
	color = vec4(1.0f, 1.0f, 1.0f, 0.25f);
	switch(uint(pushed_values.direction))
	{
		case 0: gl_Position = fma(vec4(gl_InstanceIndex), vec4(1.0f, 0.0f, 0.0f, 0.0f), source_pos) * projection_matrixes.ortho; break;
		case 1: gl_Position = fma(vec4(gl_InstanceIndex + 1), vec4(-1.0f, 0.0f, 0.0f, 0.0f), source_pos) * projection_matrixes.ortho; break;
		case 2:
			gl_Position = fma(vec4(gl_InstanceIndex), vec4(0.0f, 1.0f, 0.0f, 0.0f), source_pos.yxzw) * projection_matrixes.ortho - vec4(1.0f, 0.0f, 0.0f, 0.0f);
			break;
	}
}
