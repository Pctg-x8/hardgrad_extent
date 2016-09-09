// Device Memory Structures

use vertex_formats::*;
use constants::*;
use std;

pub type CVector4 = [f32; 4];
pub type CMatrix4 = [CVector4; 4];

#[repr(C)] pub struct VertexMemoryForWireRender
{
	pub unit_plane_source_vts: [Position; 4],
	pub player_cube_vts: [Position; 8],
	pub enemy_rezonator_vts: [Position; 3]
}
impl VertexMemoryForWireRender
{
	pub fn enemy_rezonator_offs() -> usize { unsafe { std::mem::transmute(&std::mem::transmute::<_, &VertexMemoryForWireRender>(0usize).enemy_rezonator_vts) } }
}
#[repr(C)] pub struct IndexMemory
{
	pub player_cube_ids: [u16; 24]
}
#[repr(C)] pub struct InstanceMemory
{
	pub enemy_instance_mult: [u32; MAX_ENEMY_COUNT],
	pub background_instance_mult: [u32; MAX_BK_COUNT],
	pub player_rotq: [CVector4; 2],
	pub enemy_rez_instance_data: [CVector4; MAX_ENEMY_COUNT]
}
impl InstanceMemory
{
	pub fn partial_borrow(&mut self) -> (&mut [u32; MAX_ENEMY_COUNT], &mut [u32; MAX_BK_COUNT], &mut [CVector4; 2], &mut [CVector4; MAX_ENEMY_COUNT])
	{
		(&mut self.enemy_instance_mult, &mut self.background_instance_mult, &mut self.player_rotq, &mut self.enemy_rez_instance_data)
	}
	pub fn background_offs() -> usize { unsafe { std::mem::transmute(&std::mem::transmute::<_, &InstanceMemory>(0usize).background_instance_mult) } }
	pub fn player_rot_offs() -> usize { unsafe { std::mem::transmute(&std::mem::transmute::<_, &InstanceMemory>(0usize).player_rotq) } }
	pub fn enemy_rez_offs() -> usize { unsafe { std::mem::transmute(&std::mem::transmute::<_, &InstanceMemory>(0usize).enemy_rez_instance_data) } }
}
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
	pub background_instance_data: [BackgroundInstance; MAX_BK_COUNT],
	pub player_center_tf: CVector4,
	pub render_target_desc: CVector4
}
impl UniformMemory
{
	pub fn partial_borrow(&mut self) -> (&mut Matrixes, &mut [CharacterLocation; MAX_ENEMY_COUNT], &mut [BackgroundInstance; MAX_BK_COUNT], &mut CVector4)
	{
		(&mut self.projection_matrixes, &mut self.enemy_instance_data, &mut self.background_instance_data, &mut self.player_center_tf)
	}
}
