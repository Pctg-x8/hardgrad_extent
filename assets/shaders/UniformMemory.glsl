// Uniform Memory and Mapped Structures

const int MAX_ENEMY_COUNT = 128;
const int MAX_BK_COUNT = 64;

struct Matrixes { mat4 ortho, pixel, persp; };
struct CharacterLocation { vec4 rotq[2], center_tf; };
struct BackgroundInstance { vec4 offset, scale; };

layout(std140, set = 0, binding = 0) uniform UniformMemory
{
	Matrixes projection_matrixes;
	CharacterLocation enemy_instance_data[MAX_ENEMY_COUNT];
	BackgroundInstance background_instance_data[MAX_BK_COUNT];
	vec4 player_center_tf;
	vec4 rt_metrics;
};
