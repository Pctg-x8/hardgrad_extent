#version 450

layout(points, invocations = 8) in;
layout(line_strip, max_vertices=2) out;

layout(location = 0) out vec4 color;
in gl_PerVertex { vec4 gl_Position; } gl_in[];
out gl_PerVertex { vec4 gl_Position; };

#include "UniformMemory.glsl"
layout(set = 1, binding = 0) uniform sampler1D color_gradient;

vec4 vertex_transform(vec4 inv)
{
	return inv * projection_matrixes.ortho;
}

void main()
{
	if(gl_InvocationID < uint(gl_in[0].gl_Position.x))
	{
		uint index = uint(gl_in[0].gl_Position.y) * MAX_LBPARTICLES_PER_GROUP + gl_InvocationID;
		float lifetime = (gametime.x - lb_particle_info[index].length_colrel_lifetime_lifemult.z) * lb_particle_info[index].length_colrel_lifetime_lifemult.w;
		float lifetime_inv = 1.0f - lifetime;
		float dist = (1.0f - exp(-lifetime)) * 6.0f;
		vec4 dir = vec4(lb_particle_info[index].sincos_xx.xy, 0.0f, 0.0f);
		vec4 base_pos = fma(dir, lb_particle_info[index].length_colrel_lifetime_lifemult.xxxx * dist, vec4(gl_in[0].gl_Position.zw, 0.0f, 1.0f));

		if(0.0f <= lifetime_inv && lifetime_inv <= 1.0f)
		{
			color = texture(color_gradient, lifetime);
			gl_Position = vertex_transform(base_pos); EmitVertex();
			gl_Position = vertex_transform(fma(dir, lb_particle_info[index].length_colrel_lifetime_lifemult.xxxx, base_pos)); EmitVertex();
			EndPrimitive();
		}
	}
}
