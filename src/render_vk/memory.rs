// Memory Management Shortcuts //

use std;
use vkffi::*;
use render_vk::wrap as vk;

pub struct DeviceBuffer<'d>(vk::Buffer<'d>, vk::DeviceMemory<'d>);
pub struct StagingBuffer<'d>(vk::Buffer<'d>, vk::DeviceMemory<'d>);
impl <'d> DeviceBuffer<'d>
{
	pub fn new(device_ref: &'d vk::Device, size: VkDeviceSize, usage_bits: VkBufferUsageFlags) -> Self
	{
		let b = device_ref.create_buffer(usage_bits | VK_BUFFER_USAGE_TRANSFER_DST_BIT, size).unwrap();
		let m = device_ref.allocate_memory_for_buffer(&b, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
		m.bind_buffer(&b, 0).unwrap();

		DeviceBuffer(b, m)
	}
}
impl <'d> StagingBuffer<'d>
{
	pub fn new(device_ref: &'d vk::Device, size: VkDeviceSize) -> Self
	{
		let b = device_ref.create_buffer(VK_BUFFER_USAGE_TRANSFER_SRC_BIT, size).unwrap();
		let m = device_ref.allocate_memory_for_buffer(&b, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT).unwrap();
		m.bind_buffer(&b, 0).unwrap();

		StagingBuffer(b, m)
	}
	pub fn map(&self, range: std::ops::Range<VkDeviceSize>) -> Result<vk::MemoryMappedRange, VkResult>
	{
		self.1.map(range)
	}
}

// --- Dereferencer --- //
macro_rules! Dereferencer
{
	(BufferDeref for $name: ident) =>
	{
		impl <'d> std::ops::Deref for $name<'d>
		{
			type Target = vk::Buffer<'d>;
			fn deref(&self) -> &Self::Target { &self.0 }
		}
	}
}
Dereferencer!(BufferDeref for DeviceBuffer);
Dereferencer!(BufferDeref for StagingBuffer);
