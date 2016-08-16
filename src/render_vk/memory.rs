// Memory Management Shortcuts //

use std;
use vkffi::*;
use render_vk::wrap as vk;

pub struct DeviceBuffer(vk::Buffer, vk::DeviceMemory);
pub struct StagingBuffer(vk::Buffer, vk::DeviceMemory);
impl DeviceBuffer
{
	pub fn new(device_ref: &vk::Device, size: VkDeviceSize, usage_bits: VkBufferUsageFlags) -> Self
	{
		let b = device_ref.create_buffer(usage_bits | VK_BUFFER_USAGE_TRANSFER_DST_BIT, size).unwrap();
		let m = device_ref.allocate_memory_for_buffer(&b, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
		m.bind_buffer(&b, 0).unwrap();

		DeviceBuffer(b, m)
	}
}
impl StagingBuffer
{
	pub fn new(device_ref: &vk::Device, size: VkDeviceSize) -> Self
	{
		let buffer_info = VkBufferCreateInfo
		{
			sType: VkStructureType::BufferCreateInfo, pNext: std::ptr::null(), flags: 0,
			usage: VK_BUFFER_USAGE_TRANSFER_SRC_BIT, size: size, sharingMode: VkSharingMode::Exclusive,
			queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null()
		};
		let b = device_ref.create_buffer(&buffer_info).unwrap();
		let m = device_ref.allocate_memory_for_buffer(&b, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT).unwrap();
		m.bind_buffer(&b, 0).unwrap();

		StagingBuffer(b, m)
	}
	/*pub fn map(&self, range: std::ops::Range<VkDeviceSize>) -> Result<vk::MemoryMappedRange, VkResult>
	{
		self.1.map(range)
	}*/
}

// --- Dereferencer --- //
macro_rules! Dereferencer
{
	(BufferDeref for $name: ident) =>
	{
		impl std::ops::Deref for $name
		{
			type Target = vk::Buffer;
			fn deref(&self) -> &Self::Target { &self.0 }
		}
	}
}
Dereferencer!(BufferDeref for DeviceBuffer);
Dereferencer!(BufferDeref for StagingBuffer);
