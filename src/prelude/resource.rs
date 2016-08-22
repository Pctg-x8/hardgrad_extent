// Prelude: Resources(Buffer and Image)

use prelude::internals::*;
use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use render_vk::traits::*;
use std::os::raw::c_void;

pub trait BufferResource { fn get_resource(&self) -> VkBuffer; }
pub trait ImageResource { fn get_resource(&self) -> VkImage; }

pub struct Image2D { pub internal: vk::Image }
impl ImageResource for Image2D { fn get_resource(&self) -> VkImage { self.internal.get() } }
pub struct ImageSubresourceRange(VkImageAspectFlags, u32, u32, u32, u32);
impl ImageSubresourceRange
{
	pub fn base_color() -> Self
	{
		ImageSubresourceRange(VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1)
	}
}
impl std::convert::Into<VkImageSubresourceRange> for ImageSubresourceRange
{
	fn into(self) -> VkImageSubresourceRange { (&self).into() }
}
impl <'a> std::convert::Into<VkImageSubresourceRange> for &'a ImageSubresourceRange
{
	fn into(self) -> VkImageSubresourceRange
	{
		let ImageSubresourceRange(aspect, base_mip, level_count, base_array, layer_count) = *self;
		VkImageSubresourceRange
		{
			aspectMask: aspect,
			baseMipLevel: base_mip, levelCount: level_count,
			baseArrayLayer: base_array, layerCount: layer_count
		}
	}
}

#[derive(Clone, Copy)]
pub enum BufferDataType
{
	Vertex, Index, Uniform
}
pub struct MemoryPreallocator
{
	usage_flags: VkBufferUsageFlags, offsets: Vec<usize>
}
pub trait MemoryPreallocatorInternals
{
	fn new(usage: VkBufferUsageFlags, offsets: Vec<usize>) -> Self;
	fn get_usage(&self) -> VkBufferUsageFlags;
}
impl MemoryPreallocatorInternals for MemoryPreallocator
{
	fn new(usage: VkBufferUsageFlags, offsets: Vec<usize>) -> Self { MemoryPreallocator { usage_flags: usage, offsets: offsets } }
	fn get_usage(&self) -> VkBufferUsageFlags { self.usage_flags }
}
impl MemoryPreallocator
{
	pub fn offset(&self, index: usize) -> usize { self.offsets[index] }
	pub fn total_size(&self) -> VkDeviceSize { self.offsets.last().map(|&x| x).unwrap_or(0) as VkDeviceSize }
}

pub struct DeviceBuffer
{
	buffer: vk::Buffer, memory: vk::DeviceMemory, size: VkDeviceSize
}
pub trait DeviceBufferInternals where Self: std::marker::Sized
{
	fn new(engine: &Engine, size: VkDeviceSize, usage: VkBufferUsageFlags) -> Result<Self, EngineError>;
}
impl DeviceBufferInternals for DeviceBuffer
{
	fn new(engine: &Engine, size: VkDeviceSize, usage: VkBufferUsageFlags) -> Result<Self, EngineError>
	{
		info!(target: "Prelude", "Creating Device Buffer with usage {:b}", usage);
		vk::Buffer::new(engine.get_device().get_internal(), &VkBufferCreateInfo
		{
			sType: VkStructureType::BufferCreateInfo, pNext: std::ptr::null(), flags: 0,
			size: size, usage: usage | VK_BUFFER_USAGE_TRANSFER_DST_BIT, sharingMode: VkSharingMode::Exclusive,
			queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null()
		}).and_then(|buffer|
		{
			let alloc_size = buffer.get_memory_requirements().size;
			vk::DeviceMemory::alloc(engine.get_device().get_internal(), &VkMemoryAllocateInfo
			{
				sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
				allocationSize: alloc_size, memoryTypeIndex: engine.get_memory_type_index_for_device_local()
			}).and_then(|memory| memory.bind_buffer(&buffer, 0).map(|()| DeviceBuffer { buffer: buffer, memory: memory, size: alloc_size }))
		}).map_err(EngineError::from)
	}
}
pub struct StagingBuffer
{
	buffer: vk::Buffer, memory: vk::DeviceMemory, size: VkDeviceSize
}
pub trait StagingBufferInternals where Self: std::marker::Sized
{
	fn new(engine: &Engine, size: VkDeviceSize) -> Result<Self, EngineError>;
}
impl StagingBufferInternals for StagingBuffer
{
	fn new(engine: &Engine, size: VkDeviceSize) -> Result<Self, EngineError>
	{
		vk::Buffer::new(engine.get_device().get_internal(), &VkBufferCreateInfo
		{
			sType: VkStructureType::BufferCreateInfo, pNext: std::ptr::null(), flags: 0,
			size: size, usage: VK_BUFFER_USAGE_TRANSFER_SRC_BIT, sharingMode: VkSharingMode::Exclusive,
			queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null()
		}).and_then(|buffer|
		{
			let alloc_size = buffer.get_memory_requirements().size;
			vk::DeviceMemory::alloc(engine.get_device().get_internal(), &VkMemoryAllocateInfo
			{
				sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
				allocationSize: alloc_size, memoryTypeIndex: engine.get_memory_type_index_for_host_visible()
			}).and_then(|memory| memory.bind_buffer(&buffer, 0).map(|()| StagingBuffer { buffer: buffer, memory: memory, size: alloc_size }))
		}).map_err(EngineError::from)
	}
}
impl StagingBuffer
{
	pub fn map(&self) -> Result<MemoryMappedRange, EngineError>
	{
		self.memory.map(0 .. self.size).map(|ptr| MemoryMappedRange { parent: self, ptr: ptr }).map_err(EngineError::from)
	}
}
impl BufferResource for StagingBuffer
{
	fn get_resource(&self) -> VkBuffer { self.buffer.get() }
}
impl BufferResource for DeviceBuffer
{
	fn get_resource(&self) -> VkBuffer { self.buffer.get() }
}

pub struct MemoryMappedRange<'a>
{
	parent: &'a StagingBuffer, ptr: *mut c_void
}
impl <'a> MemoryMappedRange<'a>
{
	pub fn map_mut<MappedStructureT>(&self, offset: usize) -> &mut MappedStructureT
	{
		let t: &mut MappedStructureT = unsafe { std::mem::transmute(std::mem::transmute::<_, usize>(self.ptr) + offset) };
		t
	}
}
impl <'a> std::ops::Drop for MemoryMappedRange<'a>
{
	fn drop(&mut self) { self.parent.memory.unmap(); }
}
