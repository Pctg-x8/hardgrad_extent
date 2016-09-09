
use nalgebra::*;
use utils::*;
use structures;
use constants::*;

pub struct EnemyDatastore<'a>
{
	instance_memory_ref: &'a mut [u32; MAX_ENEMY_COUNT],
	memory_block_manager: MemoryBlockManager
}
impl <'a> EnemyDatastore<'a>
{
	pub fn new(im_ref: &'a mut [u32; MAX_ENEMY_COUNT]) -> Self
	{
		EnemyDatastore
		{
			instance_memory_ref: im_ref, memory_block_manager: MemoryBlockManager::new(MAX_ENEMY_COUNT as u32)
		}
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

