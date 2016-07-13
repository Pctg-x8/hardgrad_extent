// Memory bound Resources

use std;
use vkffi::*;
use render_vk::wrap as vk;
use render_vk::traits::*;
use traits::*;

use logical_resources::*;

pub struct MemoryBoundResources<'d>
{
	#[allow(dead_code)] memory: vk::DeviceMemory<'d>, pub staging_memory: vk::DeviceMemory<'d>,
	pub buffer: vk::Buffer<'d>, pub stage_buffer: vk::Buffer<'d>,
	pub enemy_datastore_offset: VkDeviceSize, pub projection_matrixes_offset: VkDeviceSize,
	pub meshstore_offset: VkDeviceSize, pub size: VkDeviceSize
}
impl <'d> MemoryBoundResources<'d>
{
	pub fn new(device: &'d vk::Device) -> Self
	{
		let device_local_mem_index = device.parent().get_memory_type_index(VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT).expect("Unable to find device local memory");
		let staging_mem_index = device.parent().get_memory_type_index(VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT).expect("Unable to find memory for staging buffers");

		let total_size = Meshstore::device_size() + EnemyDatastore::device_size() + ProjectionMatrixes::device_size();
		let buffer = device.create_buffer(
			VK_BUFFER_USAGE_VERTEX_BUFFER_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT | VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT,
			total_size).unwrap();
		let stage_buffer = device.create_buffer(VK_BUFFER_USAGE_TRANSFER_SRC_BIT, total_size).unwrap();
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
			buffer: buffer, stage_buffer: stage_buffer,
			enemy_datastore_offset: 0, projection_matrixes_offset: EnemyDatastore::device_size(),
			meshstore_offset: EnemyDatastore::device_size() + ProjectionMatrixes::device_size(),
			size: total_size
		}
	}
}