
use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use device_resources;

const MAX_ENEMY_COUNTS: usize = 128;
pub struct EnemyDatastore
{
	#[allow(dead_code)] descriptor_set_index: usize,
	pub uniform_offset: VkDeviceSize, pub character_indices_offset: VkDeviceSize,
	descriptor_buffer_info: VkDescriptorBufferInfo
}
impl EnemyDatastore
{
	pub fn new<'d>(buffer: &vk::Buffer<'d>, offset: VkDeviceSize, descriptor_set_index: usize) -> Self
	{
		EnemyDatastore
		{
			descriptor_set_index: descriptor_set_index,
			uniform_offset: offset, character_indices_offset: offset + (std::mem::size_of::<[f32; 4]>() * 2 * MAX_ENEMY_COUNTS) as VkDeviceSize,
			descriptor_buffer_info: VkDescriptorBufferInfo(buffer.get(), offset, Self::device_size())
		}
	}
}
impl DeviceStore for EnemyDatastore
{
	fn device_size() -> VkDeviceSize
	{
		(std::mem::size_of::<[f32; 4]>() * MAX_ENEMY_COUNTS * 2 + std::mem::size_of::<u32>() * MAX_ENEMY_COUNTS) as VkDeviceSize
	}
	fn initial_stage_data(&self, _: &vk::MemoryMappedRange)
	{
		// Nothing to do
	}
}
impl HasDescriptor for EnemyDatastore
{
	fn write_descriptor_info<'d>(&self, sets: &device_resources::DescriptorSets<'d>) -> VkWriteDescriptorSet
	{
		VkWriteDescriptorSet
		{
			sType: VkStructureType::WriteDescriptorSet, pNext: std::ptr::null(),
			dstSet: sets.sets[0], dstBinding: 0, dstArrayElement: 0,
			descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			pBufferInfo: &self.descriptor_buffer_info, pImageInfo: std::ptr::null(), pTexelBufferView: std::ptr::null()
		}
	}
}