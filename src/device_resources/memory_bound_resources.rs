// Memory bound Resources

use std;
use vkffi::*;
use render_vk::wrap as vk;
use render_vk::traits::*;
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
	#[allow(dead_code)] memory: vk::DeviceMemory<'d>, pub staging_memory: vk::DeviceMemory<'d>,
	pub buffer: vk::Buffer<'d>, pub stage_buffer: vk::Buffer<'d>
}
impl <'d> MemoryBoundResources<'d>
{
	pub fn new(device: &'d vk::Device, preallocator: &MemoryPreallocator) -> Self
	{
		let device_local_mem_index = device.parent().get_memory_type_index(VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT).expect("Unable to find device local memory");
		let staging_mem_index = device.parent().get_memory_type_index(VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT).expect("Unable to find memory for staging buffers");

		let buffer = device.create_buffer(
			VK_BUFFER_USAGE_VERTEX_BUFFER_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT | VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT,
			preallocator.total_size).unwrap();
		let stage_buffer = device.create_buffer(VK_BUFFER_USAGE_TRANSFER_SRC_BIT, preallocator.total_size).unwrap();
		let alloc_info = VkMemoryAllocateInfo
		{
			sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
			allocationSize: buffer.get_memory_requirements().size, memoryTypeIndex: device_local_mem_index as u32
		};
		let stage_alloc_info = VkMemoryAllocateInfo
		{
			sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
			allocationSize: stage_buffer.get_memory_requirements().size, memoryTypeIndex: staging_mem_index as u32
		};
		let memory = device.allocate_memory(&alloc_info).unwrap();
		let stage_memory = device.allocate_memory(&stage_alloc_info).unwrap();
		memory.bind_buffer(&buffer, 0).unwrap();
		stage_memory.bind_buffer(&stage_buffer, 0).unwrap();

		MemoryBoundResources
		{
			memory: memory, staging_memory: stage_memory,
			buffer: buffer, stage_buffer: stage_buffer
		}
	}
}