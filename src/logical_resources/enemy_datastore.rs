
use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use device_resources;
use constants::*;
use nalgebra::*;
use time;
use utils::*;

#[repr(C)]
struct CharacterLocation { qrot: [[f32; 4]; 2], center_tf: [f32; 4] }
#[repr(C)]
struct UniformBufferData
{
	locations: [CharacterLocation; MAX_ENEMY_COUNT]
}

pub struct EnemyDatastore
{
	#[allow(dead_code)] descriptor_set_index: usize,
	pub uniform_offset: VkDeviceSize,
	pub character_indices_offset: VkDeviceSize,
	descriptor_buffer_info: VkDescriptorBufferInfo,
	memory_block_manager: MemoryBlockManager
}
impl EnemyDatastore
{
	pub fn new<'d>(buffer: &vk::Buffer<'d>, offset: VkDeviceSize, descriptor_set_index: usize) -> Self
	{
		EnemyDatastore
		{
			descriptor_set_index: descriptor_set_index,
			uniform_offset: offset,
			character_indices_offset: offset + Self::required_sizes()[0],
			descriptor_buffer_info: VkDescriptorBufferInfo(buffer.get(), offset, Self::required_sizes()[0]),
			memory_block_manager: MemoryBlockManager::new(MAX_ENEMY_COUNT as u32)
		}
	}
	pub fn update_instance_data(&self, mapped_range: &vk::MemoryMappedRange, index: u32, qrot1: &Quaternion<f32>, qrot2: &Quaternion<f32>, center: &Vector4<f32>)
	{
		let bufferdata_range = mapped_range.map_mut::<UniformBufferData>(self.uniform_offset);

		bufferdata_range.locations[index as usize].qrot[0] = [qrot1.i, qrot1.j, qrot1.k, qrot1.w];
		bufferdata_range.locations[index as usize].qrot[1] = [qrot2.i, qrot2.j, qrot2.k, qrot2.w];
		bufferdata_range.locations[index as usize].center_tf = [center.x, center.y, center.z, center.w];
	}
	pub fn allocate_block(&mut self, mapped_range: &vk::MemoryMappedRange) -> Option<u32>
	{
		let index = self.memory_block_manager.allocate();
		if let Some(i) = index { self.enable_instance(mapped_range, i); }
		index
	}
	pub fn free_block(&mut self, index: u32, mapped_range: &vk::MemoryMappedRange)
	{
		self.memory_block_manager.free(index);
		self.disable_instance(mapped_range, index);
	}
	fn enable_instance(&self, mapped_range: &vk::MemoryMappedRange, index: u32)
	{
		mapped_range.range_mut::<u32>(self.character_indices_offset, MAX_ENEMY_COUNT)[index as usize] = 1;
	}
	fn disable_instance(&self, mapped_range: &vk::MemoryMappedRange, index: u32)
	{
		mapped_range.range_mut::<u32>(self.character_indices_offset, MAX_ENEMY_COUNT)[index as usize] = 0;
	}
}
impl DeviceStore for EnemyDatastore
{
	fn required_sizes() -> Vec<VkDeviceSize>
	{
		vec![std::mem::size_of::<UniformBufferData>() as VkDeviceSize, std::mem::size_of::<u32>() as VkDeviceSize * MAX_ENEMY_COUNT as VkDeviceSize]
	}
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
	{
		let instance_switches_ref = mapped_range.range_mut::<u32>(self.character_indices_offset, MAX_ENEMY_COUNT);
		instance_switches_ref.copy_from_slice(&[0u32; MAX_ENEMY_COUNT]);
	}
}
impl HasDescriptor for EnemyDatastore
{
	fn write_descriptor_info<'d>(&self, sets: &device_resources::DescriptorSets<'d>) -> Vec<VkWriteDescriptorSet>
	{
		vec![VkWriteDescriptorSet
		{
			sType: VkStructureType::WriteDescriptorSet, pNext: std::ptr::null(),
			dstSet: sets.sets[self.descriptor_set_index], dstBinding: 0, dstArrayElement: 0,
			descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			pBufferInfo: &self.descriptor_buffer_info, pImageInfo: std::ptr::null(), pTexelBufferView: std::ptr::null()
		}]
	}
}
