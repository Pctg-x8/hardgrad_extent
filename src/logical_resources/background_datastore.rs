use vkffi::*;
use render_vk::wrap as vk;
use std;
use traits::*;
use device_resources;
use rand; use time;
use rand::distributions::*;

const MAX_BK_COUNT: VkDeviceSize = 64;

#[repr(C)]
struct BackgroundInstance
{
	offset: [f32; 4], scale: [f32; 4]
}
#[repr(C)]
struct UniformBufferData
{
	instances: [BackgroundInstance; MAX_BK_COUNT as usize]
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
	pub fn update(&self, mapped_range: &vk::MemoryMappedRange, mut randomizer: &mut rand::Rng, delta_time: time::Duration)
	{
		let delta_sec = delta_time.num_microseconds().unwrap_or(0) as f32 / (1000.0f32 * 1000.0f32);
		let uniform_range = mapped_range.map_mut::<UniformBufferData>(self.uniform_offset);
		let index_multipliers_range = mapped_range.map_mut::<[u32; MAX_BK_COUNT as usize]>(self.index_multipliers_offset);
		let mut rrange = rand::distributions::Range::new(0, 64 * 4);
		let mut left_range = rand::distributions::Range::new(-10.0f32, 10.0f32);
		let mut count_range = rand::distributions::Range::new(2, 10);
		let mut scale_range = rand::distributions::Range::new(1.0f32, 3.0f32);
		for i in 0 .. MAX_BK_COUNT as usize
		{
			if index_multipliers_range[i] == 0
			{
				// instantiate randomly
				if rrange.sample(&mut randomizer) == 0
				{
					let scale = scale_range.sample(&mut randomizer);
					index_multipliers_range[i] = 1;
					uniform_range.instances[i].offset = [left_range.sample(&mut randomizer), -20.0f32, -20.0f32, count_range.sample(&mut randomizer) as f32];
					uniform_range.instances[i].scale = [scale, scale, 1.0f32, 1.0f32];
				}
			}
			else
			{
				uniform_range.instances[i].offset[1] += delta_sec * 16.0f32;
				if uniform_range.instances[i].offset[1] >= 20.0f32
				{
					index_multipliers_range[i] = 0;
				}
			}
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
		let index_multipliers_range = mapped_range.map_mut::<[u32; MAX_BK_COUNT as usize]>(self.index_multipliers_offset);

		*index_multipliers_range = [0u32; MAX_BK_COUNT as usize];
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
