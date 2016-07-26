
use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use device_resources;
use constants::*;
use nalgebra::*;
use rand;
use rand::Rng;
use time;

fn size_vec4() -> VkDeviceSize { std::mem::size_of::<[f32; 4]>() as VkDeviceSize }
const ENEMY_DATA_COUNT: VkDeviceSize = MAX_ENEMY_COUNT as VkDeviceSize + 1;

type BlockIndexRange = std::ops::Range<u32>;
enum FreeOperation
{
	ConcatenateBlock(usize),
	AppendBack(usize), AppendFront(usize),
	InsertNew(usize), AppendNew
}

pub struct EnemyMemoryBlockManager
{
	freelist: std::collections::LinkedList<BlockIndexRange>
}
impl EnemyMemoryBlockManager
{
	fn new() -> Self
	{
		let mut fl = std::collections::LinkedList::<BlockIndexRange>::new();
		fl.push_back(1 .. MAX_ENEMY_COUNT as u32);

		EnemyMemoryBlockManager { freelist: fl }
	}
	fn free_all(&mut self) { self.freelist.clear(); self.freelist.push_back(1 .. MAX_ENEMY_COUNT as u32); }
	fn allocate(&mut self) -> Option<u32>
	{
		match self.freelist.pop_front()
		{
			Some(v) =>
			{
				let head = v.start;
				if v.start + 1 <= v.end { self.freelist.push_front(v.start + 1 .. v.end) };
				Some(head)
			}, None => None
		}
	}
	fn free(&mut self, index: u32)
	{
		let search = { self.free_search(index) };
		match search
		{
			FreeOperation::ConcatenateBlock(i) =>
			{
				let mut backlist = self.freelist.split_off(i + 1);
				self.freelist.back_mut().unwrap().end = backlist.pop_front().unwrap().end;
				self.freelist.append(&mut backlist);
			},
			FreeOperation::AppendBack(i) =>
			{
				let mut iter = self.freelist.iter_mut();
				iter.nth(i).unwrap().end += 1;
			}
			FreeOperation::AppendFront(i) =>
			{
				let mut iter = self.freelist.iter_mut();
				iter.nth(i).unwrap().start -= 1;
			}
			FreeOperation::InsertNew(i) =>
			{
				let mut backlist = self.freelist.split_off(i);
				self.freelist.push_back(index .. index);
				self.freelist.append(&mut backlist);
			},
			FreeOperation::AppendNew => self.freelist.push_back(index .. index)
		}
	}

	fn free_search(&self, index: u32) -> FreeOperation
	{
		fn recursive<'a, IterT>(mut iter: IterT, target: u32) -> FreeOperation
			where IterT: std::iter::Iterator<Item = (usize, &'a BlockIndexRange)>
		{
			if let Some((i, b)) = iter.next()
			{
				if target == b.end + 1
				{
					if let Some((_, b2)) = iter.next()
					{
						if target == b2.start - 1 { FreeOperation::ConcatenateBlock(i) }
						else { FreeOperation::AppendBack(i) }
					}
					else { FreeOperation::AppendBack(i) }
				}
				else if target == b.start - 1 { FreeOperation::AppendFront(i) }
				else if target < b.start { FreeOperation::InsertNew(i) }
				else { recursive(iter, target) }
			}
			else { FreeOperation::AppendNew }
		}

		recursive(self.freelist.iter().enumerate(), index)
	}
	fn dump_freelist(&self)
	{
		println!("== Freelist ==");
		for r in self.freelist.iter()
		{
			println!("-- {} .. {}", r.start, r.end);
		}
	}
}
pub fn memory_management_test()
{
	let mut mb = EnemyMemoryBlockManager::new();
	mb.dump_freelist();
	let mut list = [0; 16];
	for i in 0 .. 16
	{
		let b1 = mb.allocate().unwrap();
		println!("Allocated Memory Block: {}", b1);
		list[i] = b1;
	};
	mb.dump_freelist();
	let mut rng = rand::thread_rng();
	rng.shuffle(&mut list);
	for index in &list
	{
		println!("Freeing Index {}...", index);
		mb.free(*index);
		mb.dump_freelist();
	}

	println!("== Sequential Deallocation Performance ==");
	let seq_time =
	{
		let mut list = [0; 100];
		for i in 0 .. 100
		{
			list[i] = mb.allocate().unwrap();
		}
		let start_time = time::PreciseTime::now();
		for i in 0 .. 100
		{
			mb.free(list[i]);
		}
		start_time.to(time::PreciseTime::now()).num_nanoseconds().unwrap()
	};
	println!("x100 {}(avg. {}) ns", seq_time, seq_time / 100);
	mb.free_all();
	println!("== Random Deallocation Performance ==");
	let s_rand_time =
	{
		let mut rng = rand::thread_rng();
		let mut list = [0; 100];
		let mut dur_total = time::Duration::zero();
		for _ in 0 .. 10
		{
			for i in 0 .. 100
			{
				list[i] = mb.allocate().unwrap();
			}
			rng.shuffle(&mut list);
			let start_time = time::PreciseTime::now();
			for i in 0 .. 100 { mb.free(list[i]); }
			dur_total = dur_total + start_time.to(time::PreciseTime::now());
		}
		dur_total.num_nanoseconds().unwrap()
	};
	println!("x1000 {}(avg. {}) ns", s_rand_time, s_rand_time / 1000);
}

pub struct EnemyDatastore
{
	#[allow(dead_code)] descriptor_set_index: usize,
	pub uniform_offset: VkDeviceSize, uniform_center_tf_offset: VkDeviceSize,
	pub character_indices_offset: VkDeviceSize,
	descriptor_buffer_info: VkDescriptorBufferInfo,
	memory_block_manager: EnemyMemoryBlockManager
}
impl EnemyDatastore
{
	pub fn new<'d>(buffer: &vk::Buffer<'d>, offset: VkDeviceSize, descriptor_set_index: usize) -> Self
	{
		EnemyDatastore
		{
			descriptor_set_index: descriptor_set_index,
			uniform_offset: offset, uniform_center_tf_offset: offset + size_vec4() * 2 * ENEMY_DATA_COUNT,
			character_indices_offset: offset + size_vec4() * 3 * ENEMY_DATA_COUNT,
			descriptor_buffer_info: VkDescriptorBufferInfo(buffer.get(), offset, Self::required_sizes()[0]),
			memory_block_manager: EnemyMemoryBlockManager::new()
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
		mapped_range.range_mut::<u32>(self.character_indices_offset, MAX_ENEMY_COUNT)[(index - 1) as usize] = 1;
	}
	fn disable_instance(&self, mapped_range: &vk::MemoryMappedRange, index: u32)
	{
		mapped_range.range_mut::<u32>(self.character_indices_offset, MAX_ENEMY_COUNT)[(index - 1) as usize] = 0;
	}
}
impl DeviceStore for EnemyDatastore
{
	fn required_sizes() -> Vec<VkDeviceSize>
	{
		vec![std::mem::size_of::<[f32; 4]>() as VkDeviceSize * ENEMY_DATA_COUNT * 3, std::mem::size_of::<u32>() as VkDeviceSize * MAX_ENEMY_COUNT as VkDeviceSize]
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
