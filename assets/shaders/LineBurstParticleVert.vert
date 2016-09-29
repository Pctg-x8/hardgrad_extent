#version 450

layout(location = 0) in uint count;
layout(location = 1) in vec2 center_offs;
out gl_PerVertex { vec4 gl_Position; };

void main()
{
	gl_Position = vec4(float(count), float(gl_VertexIndex), center_offs);
}
