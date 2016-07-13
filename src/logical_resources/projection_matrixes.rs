use nalgebra::*;

use traits::*;
use render_vk::wrap as vk;
use vkffi::*;
use std;
use device_resources;

pub struct ProjectionMatrixes
{
	pub descriptor_set_index: usize,
	pub ortho_offset: VkDeviceSize, pub persp_offset: VkDeviceSize,
	pub screen_size: VkExtent2D,
	descriptor_buffer_info: VkDescriptorBufferInfo
}
impl ProjectionMatrixes
{
	pub fn new<'d>(buffer: &vk::Buffer<'d>, offset: VkDeviceSize, descriptor_set_index: usize, screen_size: VkExtent2D) -> Self
	{
		ProjectionMatrixes
		{
			descriptor_set_index: descriptor_set_index,
			ortho_offset: offset, persp_offset: offset + std::mem::size_of::<[[f32; 4]; 4]>() as VkDeviceSize,
			screen_size: screen_size,
			descriptor_buffer_info: VkDescriptorBufferInfo(buffer.get(), offset, Self::device_size())
		}
	}
}
impl DeviceStore for ProjectionMatrixes
{
	fn device_size() -> VkDeviceSize { (std::mem::size_of::<[[f32; 4]; 4]>() * 2) as VkDeviceSize }
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
	{
		let VkExtent2D(width, height) = self.screen_size;
		let (aspect, scaling) = (height as f32 / width as f32, 35.0f32);
		let ortho_matrix = OrthographicMatrix3::new(-scaling * aspect, scaling * aspect, 0.0f32, scaling, -100.0f32, 100.0f32);
		let persp_matrix = PerspectiveMatrix3::new(aspect, 70.0f32, -100.0f32, 100.0f32);

		{
			let r = mapped_range.range_mut::<f32>(self.ortho_offset, 16);
			let matr = ortho_matrix.as_matrix();
			for x in 0 .. 4 { for y in 0 .. 4 { r[x + y * 4] = matr.as_ref()[x][y]; } }
		}
		{
			let r = mapped_range.range_mut::<f32>(self.persp_offset, 16);
			let matr = persp_matrix.as_matrix();
			for x in 0 .. 4 { for y in 0 .. 4 { r[x + y * 4] = matr.as_ref()[x][y]; } }
		}
	}
}
impl HasDescriptor for ProjectionMatrixes
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
