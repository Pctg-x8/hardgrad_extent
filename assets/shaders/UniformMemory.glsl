// Uniform Memory and Mapped Structures

const int MAX_ENEMY_COUNT = 128;
const int MAX_BK_COUNT = 64;
const int MAX_LBPARTICLE_GROUPS = 48;
const int MAX_LBPARTICLES_PER_GROUP = 8;
const int MAX_LBPARTICLES = MAX_LBPARTICLE_GROUPS * MAX_LBPARTICLES_PER_GROUP;

struct Matrixes { mat4 ortho, pixel, persp; };
struct CharacterLocation { vec4 rotq[2], center_tf; };
struct BackgroundInstance { vec4 offset, scale; };
struct LineBurstParticle { vec4 length_colrel_lifetime_lifemult, sincos_xx; };

layout(std140, set = 0, binding = 0) uniform UniformMemory
{
	Matrixes projection_matrixes;
	CharacterLocation enemy_instance_data[MAX_ENEMY_COUNT];
	BackgroundInstance background_instance_data[MAX_BK_COUNT];
	vec4 player_center_tf; vec4 gametime;
	LineBurstParticle lb_particle_info[MAX_LBPARTICLES];
};
