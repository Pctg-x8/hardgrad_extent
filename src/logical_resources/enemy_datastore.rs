
use traits::*;
use nalgebra::*;
use utils::*;
use structures;
use constants::*;

pub struct EnemyDatastore<'a>
{
	uniform_memory_ref: &'a mut [structures::CharacterLocation; MAX_ENEMY_COUNT], instance_memory_ref: &'a mut [u32; MAX_ENEMY_COUNT],
	memory_block_manager: MemoryBlockManager
}
impl <'a> EnemyDatastore<'a>
{
	pub fn new(um_ref: &'a mut [structures::CharacterLocation; MAX_ENEMY_COUNT], im_ref: &'a mut [u32; MAX_ENEMY_COUNT]) -> Self
	{
		EnemyDatastore
		{
			uniform_memory_ref: um_ref, instance_memory_ref: im_ref,
			memory_block_manager: MemoryBlockManager::new(MAX_ENEMY_COUNT as u32)
		}
	}
	pub fn update_instance_data(&mut self, index: u32, qrot1: &Quaternion<f32>, qrot2: &Quaternion<f32>, center: &Vector4<f32>)
	{
		self.uniform_memory_ref[index as usize].rotq[0] = [qrot1.i, qrot1.j, qrot1.k, qrot1.w];
		self.uniform_memory_ref[index as usize].rotq[1] = [qrot2.i, qrot2.j, qrot2.k, qrot2.w];
		self.uniform_memory_ref[index as usize].center_tf = [center.x, center.y, center.z, center.w];
	}
	pub fn allocate_block(&mut self) -> Option<u32>
	{
		let index = self.memory_block_manager.allocate();
		if let Some(i) = index { self.enable_instance(i); }
		index
	}
	pub fn free_block(&mut self, index: u32)
	{
		self.memory_block_manager.free(index);
		self.disable_instance(index);
	}
	fn enable_instance(&mut self, index: u32)
	{
		self.instance_memory_ref[index as usize] = 1;
	}
	fn disable_instance(&mut self, index: u32)
	{
		self.instance_memory_ref[index as usize] = 0;
	}
}

