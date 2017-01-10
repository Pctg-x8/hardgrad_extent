// Constants

// Limitations
pub const MAX_ENEMY_COUNT: usize = 128;
pub const MAX_BK_COUNT: usize = 64;
pub const MAX_PLAYER_BULLET_COUNT: usize = 64;
pub const MAX_LBPARTICLE_GROUPS: usize = 48;
pub const MAX_LBPARTICLES_PER_GROUP: usize = 8;
pub const MAX_LBPARTICLES: usize = MAX_LBPARTICLE_GROUPS * MAX_LBPARTICLES_PER_GROUP;
pub const MAX_BULLETS: usize = 128 * MAX_ENEMY_COUNT;

// Metrics
pub const PLAYER_SIZE_H: f32 = 1.5;
pub const SCREEN_HSIZE: f32 = 36.0;
pub const SCREEN_VSIZE: f32 = 4.0 * 36.0 / 3.0;
pub const PLAYER_HLIMIT: f32 = SCREEN_HSIZE - PLAYER_SIZE_H;
pub const PLAYER_VLIMIT: f32 = SCREEN_VSIZE - PLAYER_SIZE_H;
