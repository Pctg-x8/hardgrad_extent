// Memory bound Resources

use std;
use vkffi::*;
use render_vk::wrap as vk;
use render_vk::memory::*;
use traits::*;

use logical_resources::*;

pub type MemoryRange = std::ops::Range<VkDeviceSize>;		// begin .. end
pub struct MemoryPreallocator
{
	pub meshstore_range: MemoryRange,
	pub projection_matrixes_range: MemoryRange,
	pub enemy_datastore_range: MemoryRange,
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

		// Preallocations
		let meshstore_range: MemoryRange = 0 .. Meshstore::device_size();
		// required alignment
		let projection_matrixes_range: MemoryRange = align_for_uniform_buffer(meshstore_range.end) .. align_for_uniform_buffer(meshstore_range.end) + ProjectionMatrixes::device_size();
		let enemy_datastore_range: MemoryRange = align_for_uniform_buffer(projection_matrixes_range.end) .. align_for_uniform_buffer(projection_matrixes_range.end) + EnemyDatastore::device_size();
	
		MemoryPreallocator
		{
			total_size: enemy_datastore_range.end,
			meshstore_range: meshstore_range,
			projection_matrixes_range: projection_matrixes_range,
			enemy_datastore_range: enemy_datastore_range
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