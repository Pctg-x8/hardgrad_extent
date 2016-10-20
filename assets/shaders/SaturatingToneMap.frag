#version 450

layout(location = 0) out vec4 target;

layout(input_attachment_index = 0) uniform subpassInput prepixels;

void main()
{
	target = clamp(subpassLoad(prepixels), vec4(0.0f), vec4(1.0f));
}
