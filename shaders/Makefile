# Makefile for GLSL Shader Sources

CC_GLSL = glslangValidator -V -l
.SUFFIXES: .vert .frag .spv
VERT_OUTS = $(patsubst %.vert,%.spv,$(wildcard *.vert))
FRAG_OUTS = $(patsubst %.frag,%.spv,$(wildcard *.frag))
ALL_OUTS = $(VERT_OUTS) $(FRAG_OUTS)

all: $(ALL_OUTS)
	

.vert.spv:
	$(CC_GLSL) $< -o $@

.frag.spv:
	$(CC_GLSL) $< -o $@

