use vkffi::*;
use render_vk::wrap as vk;
use std;
use traits::*;
use device_resources;

const MAX_BK_COUNT: VkDeviceSize = 64;

#[repr(C)]
struct UniformBufferData
{
	offsets: [[f32; 4]; MAX_BK_COUNT as usize]
}

pub struct BackgroundDatastore
{
	#[allow(dead_code)] descriptor_set_index: usize,
	pub uniform_offset: VkDeviceSize, pub index_multipliers_offset: VkDeviceSize,
	descriptor_buffer_info: VkDescriptorBufferInfo
}
impl BackgroundDatastore
{
	pub fn new<'d>(buffer: &vk::Buffer<'d>, offset: VkDeviceSize, descriptor_set_index: usize) -> BackgroundDatastore
	{
		BackgroundDatastore
		{
			descriptor_set_index: descriptor_set_index,
			uniform_offset: offset,
			index_multipliers_offset: offset + Self::required_sizes()[0],
			descriptor_buffer_info: VkDescriptorBufferInfo(**buffer, offset, Self::required_sizes()[0])
		}
	}
}
impl DeviceStore for BackgroundDatastore
{
	fn required_sizes() -> Vec<VkDeviceSize>
	{
		vec![std::mem::size_of::<UniformBufferData>() as VkDeviceSize, std::mem::size_of::<u32>() as VkDeviceSize * MAX_BK_COUNT as VkDeviceSize]
	}
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
	{
		let uniform_range = mapped_range.map_mut::<UniformBufferData>(self.uniform_offset);
		let index_multipliers_range = mapped_range.map_mut::<[u32; MAX_BK_COUNT as usize]>(self.index_multipliers_offset);

		*index_multipliers_range = [0u32; MAX_BK_COUNT as usize];

		uniform_range.offsets[0] = [0.0f32, 0.0f32, -20.0f32, 10.0f32];
		index_multipliers_range[0] = 1;
	}
}
impl HasDescriptor for BackgroundDatastore
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