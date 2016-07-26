# Makefile for GLSL Shader Sources

CC_GLSL = glslangValidator -V -l
.SUFFIXES: .vert .frag .geom .spv
VERT_OUTS = $(patsubst %.vert,%.spv,$(wildcard *.vert))
FRAG_OUTS = $(patsubst %.frag,%.spv,$(wildcard *.frag))
GEOM_OUTS = $(patsubst %.geom,%.spv,$(wildcard *.geom))
ALL_OUTS = $(VERT_OUTS) $(FRAG_OUTS) $(GEOM_OUTS)

all: $(ALL_OUTS)


.vert.spv:
	$(CC_GLSL) $< -o $@

.frag.spv:
	$(CC_GLSL) $< -o $@

.geom.spv:
	$(CC_GLSL) $< -o $@
