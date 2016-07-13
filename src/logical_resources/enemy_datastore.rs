
use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use device_resources;
use constants::*;
use nalgebra::*;

pub struct EnemyDatastore
{
	#[allow(dead_code)] descriptor_set_index: usize,
	pub uniform_offset: VkDeviceSize, uniform_center_tf_offset: VkDeviceSize,
	pub character_indices_offset: VkDeviceSize,
	descriptor_buffer_info: VkDescriptorBufferInfo
}
impl EnemyDatastore
{
	pub fn new<'d>(buffer: &vk::Buffer<'d>, offset: VkDeviceSize, descriptor_set_index: usize) -> Self
	{
		EnemyDatastore
		{
			descriptor_set_index: descriptor_set_index,
			uniform_offset: offset, uniform_center_tf_offset: offset + (std::mem::size_of::<[f32; 4]>() * 2 * MAX_ENEMY_COUNT) as VkDeviceSize,
			character_indices_offset: offset + (std::mem::size_of::<[f32; 4]>() * 3 * MAX_ENEMY_COUNT) as VkDeviceSize,
			descriptor_buffer_info: VkDescriptorBufferInfo(buffer.get(), offset, Self::device_size())
		}
	}
	pub fn update_instance_data(&self, mapped_range: &vk::MemoryMappedRange, index: usize, qrot1: &Quaternion<f32>, qrot2: &Quaternion<f32>, center: &Vector4<f32>)
	{
		assert!(index != 0, "Index 0 is reserved for unused");

		let q1_range = mapped_range.range_mut::<f32>(self.uniform_offset + (std::mem::size_of::<[f32; 4]>() * index) as VkDeviceSize, 4);
		let q2_range = mapped_range.range_mut::<f32>(self.uniform_offset + (std::mem::size_of::<[f32; 4]>() * (MAX_ENEMY_COUNT + index)) as VkDeviceSize, 4);
		let cv_range = mapped_range.range_mut::<f32>(self.uniform_center_tf_offset + (std::mem::size_of::<[f32; 4]>() * index) as VkDeviceSize, 4);

		q1_range[0] = qrot1.i; q1_range[1] = qrot1.j; q1_range[2] = qrot1.k; q1_range[3] = qrot1.w;
		q2_range[0] = qrot2.i; q2_range[1] = qrot2.j; q2_range[2] = qrot2.k; q2_range[3] = qrot2.w;
		cv_range[0] = center.x; cv_range[1] = center.y; cv_range[2] = center.z; cv_range[3] = center.w;
	}
}
impl DeviceStore for EnemyDatastore
{
	fn device_size() -> VkDeviceSize
	{
		(std::mem::size_of::<[f32; 4]>() * MAX_ENEMY_COUNT * 3 + std::mem::size_of::<u32>() * MAX_ENEMY_COUNT) as VkDeviceSize
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
			dstSet: sets.sets[self.descriptor_set_index], dstBinding: 0, dstArrayElement: 0,
			descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			pBufferInfo: &self.descriptor_buffer_info, pImageInfo: std::ptr::null(), pTexelBufferView: std::ptr::null()
		}
	}
}