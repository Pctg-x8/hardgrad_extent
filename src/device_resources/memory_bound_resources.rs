// Memory bound Resources

use vkffi::*;
use render_vk::wrap as vk;
use render_vk::memory::*;
use traits::*;
use structures;
use std;

use logical_resources::*;

pub struct MemoryPreallocator
{
	pub meshstore_base: VkDeviceSize,
	pub instance_base: VkDeviceSize,
	pub uniform_memory_base: VkDeviceSize,
	pub uniform_memory_size: VkDeviceSize,
	pub total_size: VkDeviceSize
}
impl MemoryPreallocator
{
	pub fn new(adapter: &vk::PhysicalDevice) -> Self
	{
		fn align(x: VkDeviceSize, a: VkDeviceSize) -> VkDeviceSize { (x as f64 / a as f64).ceil() as VkDeviceSize * a }
		let adapter_limits = adapter.get_properties().limits;
		let uniform_buffer_alignment = adapter_limits.minUniformBufferOffsetAlignment;
		let align_for_uniform_buffer = |x: VkDeviceSize| align(x, uniform_buffer_alignment);

		// Request all memory block sizes
		let meshstore_size = Meshstore::required_sizes().iter().fold(0, |a, x| a + x);
		let vi_buffer_size = meshstore_size + std::mem::size_of::<structures::InstanceMemory>() as VkDeviceSize;
		let vi_buffer_aligned_size = align_for_uniform_buffer(vi_buffer_size);
		let uniform_memory_size = std::mem::size_of::<structures::UniformMemory>() as VkDeviceSize;
		// Preallocations
		let meshstore_base_offs = 0;
		let instance_base_offs = meshstore_base_offs + meshstore_size;
		let uniform_base_offs = meshstore_base_offs + vi_buffer_aligned_size;

		println!("-- Memory Preallocation: {} bytes", uniform_base_offs + uniform_memory_size);

		MemoryPreallocator
		{
			total_size: uniform_base_offs + uniform_memory_size,
			meshstore_base: meshstore_base_offs,
			instance_base: instance_base_offs,
			uniform_memory_base: uniform_base_offs,
			uniform_memory_size: uniform_memory_size
		}
	}
}

pub struct MemoryBoundResources<'d>
{
	pub buffer: DeviceBuffer<'d>, pub stage_buffer: StagingBuffer<'d>
}
impl <'d> MemoryBoundResources<'d>
{
	pub fn new(device: &'d vk::Device, preallocator: &MemoryPreallocator) -> Self
	{
		MemoryBoundResources
		{
			buffer: DeviceBuffer::new(device, preallocator.total_size, VK_BUFFER_USAGE_VERTEX_BUFFER_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT | VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT),
			stage_buffer: StagingBuffer::new(device, preallocator.total_size)
		}
	}
}
