
use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use device_resources;
use constants::*;
use nalgebra::*;

fn size_vec4() -> VkDeviceSize { std::mem::size_of::<[f32; 4]>() as VkDeviceSize }
const ENEMY_DATA_COUNT: VkDeviceSize = MAX_ENEMY_COUNT as VkDeviceSize + 1;

type BlockIndexRange = std::ops::Range<u32>;
pub struct EnemyDatastore
{
	#[allow(dead_code)] descriptor_set_index: usize,
	pub uniform_offset: VkDeviceSize, uniform_center_tf_offset: VkDeviceSize,
	pub character_indices_offset: VkDeviceSize,
	descriptor_buffer_info: VkDescriptorBufferInfo,
	freelist: std::collections::LinkedList<BlockIndexRange>
}
impl EnemyDatastore
{
	pub fn new<'d>(buffer: &vk::Buffer<'d>, offset: VkDeviceSize, descriptor_set_index: usize) -> Self
	{
		let mut fl = std::collections::LinkedList::<BlockIndexRange>::new();
		fl.push_back(1 .. MAX_ENEMY_COUNT as u32);

		EnemyDatastore
		{
			descriptor_set_index: descriptor_set_index,
			uniform_offset: offset, uniform_center_tf_offset: offset + size_vec4() * 2 * ENEMY_DATA_COUNT,
			character_indices_offset: offset + size_vec4() * 3 * ENEMY_DATA_COUNT,
			descriptor_buffer_info: VkDescriptorBufferInfo(buffer.get(), offset, Self::device_size()),
			freelist: fl
		}
	}
	pub fn update_instance_data(&self, mapped_range: &vk::MemoryMappedRange, index: u32, qrot1: &Quaternion<f32>, qrot2: &Quaternion<f32>, center: &Vector4<f32>)
	{
		assert!(index != 0, "Index 0 is reserved for unused");

		let q1_range = mapped_range.range_mut::<[f32; 4]>(self.uniform_offset + size_vec4() * index as VkDeviceSize, 1);
		let q2_range = mapped_range.range_mut::<[f32; 4]>(self.uniform_offset + size_vec4() * (ENEMY_DATA_COUNT + index as VkDeviceSize), 1);
		let cv_range = mapped_range.range_mut::<[f32; 4]>(self.uniform_center_tf_offset + size_vec4() * index as VkDeviceSize, 1);

		q1_range[0] = [qrot1.i, qrot1.j, qrot1.k, qrot1.w];
		q2_range[0] = [qrot2.i, qrot2.j, qrot2.k, qrot2.w];
		cv_range[0] = [center.x, center.y, center.z, center.w];
	}
	pub fn allocate_block(&mut self, mapped_range: &vk::MemoryMappedRange) -> Option<u32>
	{
		if self.freelist.is_empty() { panic!("Unable to allocate block"); }
		let front_elem = self.freelist.pop_front();
		match front_elem
		{
			Some(v) =>
			{
				let head = v.start;
				if v.start + 1 < v.end { self.freelist.push_front(v.start + 1 .. v.end) };
				self.enable_instance(mapped_range, head);
				Some(head)
			}, None => None
		}
	}
	pub fn free_block(&mut self, index: u32, mapped_range: &vk::MemoryMappedRange)
	{
		let mut cloned = self.freelist.clone();
		let mut iter = cloned.iter_mut().enumerate();
		while let Some((i, mut b)) = iter.next()
		{
			if index == b.end + 1
			{
				if let Some((_, b_after)) = iter.next()
				{
					if index == b_after.start - 1
					{
						// concat
						b.end = b_after.end;
						let mut frontlist = self.freelist.split_off(i + 1);
						self.freelist.pop_front().unwrap();
						frontlist.append(&mut self.freelist);
						self.freelist = frontlist;
						self.disable_instance(mapped_range, index);
						return;
					}
					else
					{
						// append to back
						b.end = b.end + 1;
						self.disable_instance(mapped_range, index);
						return;
					}
				}
				else
				{
					// append to back
					b.end = b.end + 1;
					self.disable_instance(mapped_range, index);
					return;
				}
			}
			else if index == b.start - 1
			{
				// append to front
				b.start = b.start - 1;
				self.disable_instance(mapped_range, index);
				return;
			}
			else if index < b.start
			{
				// new block
				let mut frontlist = self.freelist.split_off(i + 1);
				frontlist.push_back(index .. index);
				frontlist.append(&mut self.freelist);
				self.freelist = frontlist;
				self.disable_instance(mapped_range, index);
				return;
			}
		}
		// append to last
		self.freelist.push_back(index .. index);
		self.disable_instance(mapped_range, index);
	}
	fn enable_instance(&self, mapped_range: &vk::MemoryMappedRange, index: u32)
	{
		mapped_range.range_mut::<u32>(self.character_indices_offset, MAX_ENEMY_COUNT)[(index - 1) as usize] = 1;
	}
	fn disable_instance(&self, mapped_range: &vk::MemoryMappedRange, index: u32)
	{
		mapped_range.range_mut::<u32>(self.character_indices_offset, MAX_ENEMY_COUNT)[(index - 1) as usize] = 0;
	}
}
impl DeviceStore for EnemyDatastore
{
	fn device_size() -> VkDeviceSize
	{
		(std::mem::size_of::<[f32; 4]>() * (MAX_ENEMY_COUNT + 1) * 3 + std::mem::size_of::<u32>() * MAX_ENEMY_COUNT) as VkDeviceSize
	}
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
	{
		// Setup Invalid Data
		let first_q_ref = mapped_range.range_mut::<[f32; 4]>(self.uniform_offset, 1);
		let second_q_ref = mapped_range.range_mut::<[f32; 4]>(self.uniform_offset + size_vec4() * ENEMY_DATA_COUNT, 1);
		let center_tf_ref = mapped_range.range_mut::<[f32; 4]>(self.uniform_center_tf_offset, 1);
		let instance_switches_ref = mapped_range.range_mut::<u32>(self.character_indices_offset, MAX_ENEMY_COUNT);

		first_q_ref[0] = [0.0f32; 4];
		second_q_ref[0] = [0.0f32; 4];
		center_tf_ref[0] = [0.0f32; 4];
		for i in 0 .. MAX_ENEMY_COUNT { instance_switches_ref[i] = 0u32; }
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
