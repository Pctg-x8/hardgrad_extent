#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(lines_adjacency, invocations = 6) in;
layout(line_strip, max_vertices = 16) out;

layout(location = 0) in vec4 color_in[];
layout(location = 1) in vec4 instance_data[];
#define MAX_REZONATORS uint(instance_data[0].x)
#define FIRST_ROT instance_data[0].y
#define FIRST_REZ_ROT instance_data[0].z
#define INSTANCE_ID uint(instance_data[0].w)

const float RezonatorScaling = 0.875f;
const float OffsetScaling = 3.0f;

layout(location = 0) out vec4 color;
in gl_PerVertex { vec4 gl_Position; } gl_in[];
out gl_PerVertex { vec4 gl_Position; };

#include "UniformMemory.glsl"
#include "Quaternion.glsl"

const float PI = atan(1.0f) * 4.0f;

vec4 vertex_transform(vec4 base)
{
	float ay = fma(float(gl_InvocationID), 10.0f * PI / 180.0f, FIRST_REZ_ROT);
	float az = fma(float(gl_InvocationID), (2.0f * PI) / MAX_REZONATORS, FIRST_ROT);
	mat4 yrotmatr = mat4(vec4(cos(ay), 0.0f, sin(ay), 0.0f), vec4(0.0f, 1.0f, 0.0f, 0.0f), vec4(-sin(ay), 0.0f, cos(ay), 0.0f), vec4(0.0f, 0.0f, 0.0f, 1.0f));
	vec4 raw_pos = (RezonatorScaling * base * yrotmatr + vec4(-sin(az), cos(az), 0.0f, 1.0f) * OffsetScaling) + enemy_instance_data[INSTANCE_ID].center_tf;
	raw_pos.w = 1.0f;
	return raw_pos * projection_matrixes.ortho;
}

void emit_rezonator()
{
	color = color_in[0];
	gl_Position = vertex_transform(gl_in[1].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f)); EmitVertex();
	gl_Position = vertex_transform(gl_in[2].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f)); EmitVertex();
	gl_Position = vertex_transform(gl_in[2].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f)); EmitVertex();
	gl_Position = vertex_transform(gl_in[1].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f)); EmitVertex();
	gl_Position = vertex_transform(gl_in[1].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f)); EmitVertex();
	gl_Position = vertex_transform(gl_in[0].gl_Position); EmitVertex();
	gl_Position = vertex_transform(gl_in[2].gl_Position - vec4(0.0f, 0.0f, 1.0f, 0.0f)); EmitVertex();
	gl_Position = vertex_transform(gl_in[2].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f)); EmitVertex();
	gl_Position = vertex_transform(gl_in[0].gl_Position); EmitVertex();
	gl_Position = vertex_transform(gl_in[1].gl_Position + vec4(0.0f, 0.0f, 1.0f, 0.0f)); EmitVertex();
	EndPrimitive();
}

void main()
{
	if(MAX_REZONATORS > gl_InvocationID) emit_rezonator();
}

/*
struct OutVertex
{
	[out Position] pos :: Vec4,
	[out 0] color :: Vec4
}

vertex_transform base = raw_pos * uniform_memory.projection_matrixes.ortho where
	raw_pos = Vec4From3 (rotZ rotY $ Vec4 0 1 0 1) 1 + uniform_memory.enemy_instance_data[instance_id].center_tf,
	rotY v4 = v4 * yrotmatr ay, rotZ v4 = v4 * zrotmatr az,
	yrotmatr a = Mat4 (Vec4 (cos a) 0 (sin a) 0) (Vec4 0 1 0 0) (Vec4 (neg sin a) 0 (cos a) 0) (Vec4 0 0 0 1),
	zrotmatr a = Mat4 (Vec4 (cos a) (neg sin a) 0 0) (Vec4 (sin a) (cos a) 0 0) (Vec4 0 0 1 0) (Vec4 0 0 0 1),
	ay = fma invocation_count 10 first_rot_rezonator, az = fma invocation_count (360 / max_rezonators) first_rot

final_primitive = [
	OutVertex { pos: vertex_transform $ pos_in[1] - Vec4 0 0 1 0, color: color_in[0] },
	OutVertex { pos: vertex_transform $ pos_in[2] - Vec4 0 0 1 0, color: color_in[0] },
	OutVertex { pos: vertex_transform $ pos_in[2] + Vec4 0 0 1 0, color: color_in[0] },
	OutVertex { pos: vertex_transform $ pos_in[1] + Vec4 0 0 1 0, color: color_in[0] },
	OutVertex { pos: vertex_transform $ pos_in[1] - Vec4 0 0 1 0, color: color_in[0] },
	OutVertex { pos: vertex_transform pos_in[0], color: color_in[0] },
	OutVertex { pos: vertex_transform $ pos_in[2] - Vec4 0 0 1 0, color: color_in[0] },
	OutVertex { pos: vertex_transform $ pos_in[2] + Vec4 0 0 1 0, color: color_in[0] },
	OutVertex { pos: vertex_transform pos_in[0], color: color_in[0] },
	OutVertex { pos: vertex_transform $ pos_in[1] + Vec4 0 0 1 0, color: color_in[0] }
]
[out LineStrip] primitive = if max_rezonators > invocation_count then final_primitive else Discard
*/
