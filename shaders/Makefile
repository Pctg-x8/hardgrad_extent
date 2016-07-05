# Makefile for GLSL Shader Sources

CC_GLSL = glslangValidator -V -l
.SUFFIXES: .vert .frag .spv

.vert.spv:
	$(CC_GLSL) $< -o $@

.frag.spv:
	$(CC_GLSL) $< -o $@

