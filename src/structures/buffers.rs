// Device Memory Structures

use vertex_formats::*;
use std;

pub const MAX_ENEMY_COUNT: usize = 128;
pub const MAX_BACKGROUND_COUNT: usize = 64;

pub type CVector4 = [f32; 4];
pub type CMatrix4 = [CVector4; 4];

#[repr(C)] pub struct VertexMemoryForWireRender
{
    pub unit_plane_source_vts: [Position; 4],
    pub player_cube_vts: [Position; 8]
}
pub fn player_cube_vertex_offs() -> usize { unsafe { std::mem::transmute(&std::mem::transmute::<_, &VertexMemoryForWireRender>(0usize).player_cube_vts) } }
#[repr(C)] pub struct IndexMemory
{
    pub player_cube_ids: [u16; 24]
}
#[repr(C)] pub struct InstanceMemory
{
    pub enemy_instance_mult: [u32; MAX_ENEMY_COUNT],
    pub background_instance_mult: [u32; MAX_BACKGROUND_COUNT],
    pub player_rotq: [CVector4; 2]
}
pub fn background_instance_offs() -> usize { unsafe { std::mem::transmute(&std::mem::transmute::<_, &InstanceMemory>(0usize).background_instance_mult) } }
pub fn player_instance_offs() -> usize { unsafe { std::mem::transmute(&std::mem::transmute::<_, &InstanceMemory>(0usize).player_rotq) } }
#[repr(C)] pub struct Matrixes
{
    pub ortho: CMatrix4, pub pixel: CMatrix4, pub persp: CMatrix4
}
#[repr(C)] pub struct CharacterLocation
{
    pub rotq: [CVector4; 2], pub center_tf: CVector4
}
#[repr(C)] pub struct BackgroundInstance
{
    pub offset: CVector4, pub scale: CVector4
}
#[repr(C)] pub struct UniformMemory
{
    pub projection_matrixes: Matrixes,
    pub enemy_instance_data: [CharacterLocation; MAX_ENEMY_COUNT],
    pub background_instance_data: [BackgroundInstance; MAX_BACKGROUND_COUNT],
    pub player_center_tf: CVector4
}
