
use traits::*;
use constants::*;
use nalgebra::*;
use utils::*;
use structures;

pub struct EnemyDatastore { memory_block_manager: MemoryBlockManager }
impl EnemyDatastore
{
	pub fn new() -> Self
	{
		EnemyDatastore { memory_block_manager: MemoryBlockManager::new(MAX_ENEMY_COUNT as u32) }
	}
	pub fn update_instance_data(&self, memory_ref: &mut structures::UniformMemory, index: u32, qrot1: &Quaternion<f32>, qrot2: &Quaternion<f32>, center: &Vector4<f32>)
	{
		memory_ref.enemy_instance_data[index as usize].rotq[0] = [qrot1.i, qrot1.j, qrot1.k, qrot1.w];
		memory_ref.enemy_instance_data[index as usize].rotq[1] = [qrot2.i, qrot2.j, qrot2.k, qrot2.w];
		memory_ref.enemy_instance_data[index as usize].center_tf = [center.x, center.y, center.z, center.w];
	}
	pub fn allocate_block(&mut self, memory_ref: &mut structures::InstanceMemory) -> Option<u32>
	{
		let index = self.memory_block_manager.allocate();
		if let Some(i) = index { self.enable_instance(memory_ref, i); }
		index
	}
	pub fn free_block(&mut self, index: u32, memory_ref: &mut structures::InstanceMemory)
	{
		self.memory_block_manager.free(index);
		self.disable_instance(memory_ref, index);
	}
	fn enable_instance(&self, memory_ref: &mut structures::InstanceMemory, index: u32)
	{
		memory_ref.enemy_instance_mult[index as usize] = 1;
	}
	fn disable_instance(&self, memory_ref: &mut structures::InstanceMemory, index: u32)
	{
		memory_ref.enemy_instance_mult[index as usize] = 0;
	}
}
impl UniformStore for EnemyDatastore
{
	fn initial_stage_data(&self, memory_ref: &mut structures::UniformMemory) {}
}
