#version 400
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 color;
layout(location = 0) out vec4 target;

void main() { target = vec4(color.xyz * color.a, color.a); }

