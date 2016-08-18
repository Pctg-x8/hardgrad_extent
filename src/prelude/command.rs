// Prelude: Command Pool and Buffers

use prelude::internals::*;
use std;
use std::rc::Rc;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*; use render_vk::traits::*;

pub struct CommandPool
{
	graphics: Rc<vk::CommandPool>, transfer: Rc<vk::CommandPool>, transient: vk::CommandPool
}
impl CommandPool
{
	pub fn new(device: &Device) -> Result<Self, EngineError>
	{
		vk::CommandPool::new(device, device.get_graphics_queue(), false).and_then(|g|
		vk::CommandPool::new(device, device.get_transfer_queue(), false).and_then(move |t|
		vk::CommandPool::new(device, device.get_transfer_queue(), true).map(move |tt| CommandPool
		{
			graphics: Rc::new(g), transfer: Rc::new(t), transient: tt
		}))).map_err(EngineError::from)
	}
}
pub trait CommandPoolInternals
{
	fn for_graphics(&self) -> &Rc<vk::CommandPool>;
	fn for_transfer(&self) -> &Rc<vk::CommandPool>;
	fn for_transient(&self) -> &vk::CommandPool;
}
impl CommandPoolInternals for CommandPool
{
	fn for_graphics(&self) -> &Rc<vk::CommandPool> { &self.graphics }
	fn for_transfer(&self) -> &Rc<vk::CommandPool> { &self.transfer }
	fn for_transient(&self) -> &vk::CommandPool { &self.transient }
}

pub struct MemoryBarrier { pub src_access_mask: VkAccessFlags, pub dst_access_mask: VkAccessFlags }
impl std::default::Default for MemoryBarrier
{
	fn default() -> Self
	{
		MemoryBarrier { src_access_mask: VK_ACCESS_MEMORY_READ_BIT, dst_access_mask: VK_ACCESS_MEMORY_READ_BIT }
	}
}
impl <'a> std::convert::Into<VkMemoryBarrier> for &'a MemoryBarrier
{
	fn into(self) -> VkMemoryBarrier
	{
		VkMemoryBarrier
		{
			sType: VkStructureType::MemoryBarrier, pNext: std::ptr::null(),
			srcAccessMask: self.src_access_mask, dstAccessMask: self.dst_access_mask
		}
	}
}
pub struct BufferMemoryBarrier<'a>
{
	pub src_access_mask: VkAccessFlags, pub dst_access_mask: VkAccessFlags,
	pub src_queue_family_index: u32, pub dst_queue_family_index: u32,
	pub buffer: &'a Buffer, pub range: std::ops::Range<VkDeviceSize>
}
impl <'a> BufferMemoryBarrier<'a>
{
	pub fn hold_ownership(buf: &'a Buffer, range: std::ops::Range<VkDeviceSize>,
		src_access_mask: VkAccessFlags, dst_access_mask: VkAccessFlags) -> Self
	{
		BufferMemoryBarrier
		{
			src_access_mask: src_access_mask, dst_access_mask: dst_access_mask,
			src_queue_family_index: VK_QUEUE_FAMILY_IGNORED, dst_queue_family_index: VK_QUEUE_FAMILY_IGNORED,
			buffer: buf, range: range
		}
	}
}
impl <'a> std::convert::Into<VkBufferMemoryBarrier> for &'a BufferMemoryBarrier<'a>
{
	fn into(self) -> VkBufferMemoryBarrier
	{
		VkBufferMemoryBarrier
		{
			sType: VkStructureType::BufferMemoryBarrier, pNext: std::ptr::null(),
			srcAccessMask: self.src_access_mask, dstAccessMask: self.dst_access_mask,
			srcQueueFamilyIndex: self.src_queue_family_index, dstQueueFamilyIndex: self.dst_queue_family_index,
			buffer: self.buffer.internal.get(), offset: self.range.start, size: self.range.end - self.range.start
		}
	}
}
pub struct ImageMemoryBarrier<'a>
{
	pub src_access_mask: VkAccessFlags, pub dst_access_mask: VkAccessFlags,
	pub src_layout: VkImageLayout, pub dst_layout: VkImageLayout,
	pub src_queue_family_index: u32, pub dst_queue_family_index: u32,
	pub image: &'a ImageResource, pub subresource_range: ImageSubresourceRange
}
impl <'a> ImageMemoryBarrier<'a>
{
	pub fn hold_ownership(img: &'a ImageResource, subresource_range: ImageSubresourceRange,
		src_access_mask: VkAccessFlags, dst_access_mask: VkAccessFlags,
		src_layout: VkImageLayout, dst_layout: VkImageLayout) -> Self
	{
		ImageMemoryBarrier
		{
			src_access_mask: src_access_mask, dst_access_mask: dst_access_mask,
			src_layout: src_layout, dst_layout: dst_layout,
			src_queue_family_index: VK_QUEUE_FAMILY_IGNORED, dst_queue_family_index: VK_QUEUE_FAMILY_IGNORED,
			image: img, subresource_range: subresource_range
		}
	}
}
impl <'a> std::convert::Into<VkImageMemoryBarrier> for &'a ImageMemoryBarrier<'a>
{
	fn into(self) -> VkImageMemoryBarrier
	{
		VkImageMemoryBarrier
		{
			sType: VkStructureType::ImageMemoryBarrier, pNext: std::ptr::null(),
			srcAccessMask: self.src_access_mask, dstAccessMask: self.dst_access_mask,
			oldLayout: self.src_layout, newLayout: self.dst_layout,
			srcQueueFamilyIndex: self.src_queue_family_index, dstQueueFamilyIndex: self.dst_queue_family_index,
			image: self.image.get_resource(), subresourceRange: (&self.subresource_range).into()
		}
	}
}

pub struct GraphicsCommandBuffers { parent: Rc<vk::CommandPool>, internal: Vec<VkCommandBuffer> }
impl std::ops::Drop for GraphicsCommandBuffers
{
	fn drop(&mut self)
	{
		unsafe { vkFreeCommandBuffers(self.parent.parent().get(), self.parent.get(), self.internal.len() as u32, self.internal.as_ptr()) };
	}
}
pub trait GraphicsCommandBuffersInternals { fn new(parent: &Rc<vk::CommandPool>, cbs: Vec<VkCommandBuffer>) -> Self; }
impl GraphicsCommandBuffersInternals for GraphicsCommandBuffers
{
	fn new(parent: &Rc<vk::CommandPool>, cbs: Vec<VkCommandBuffer>) -> Self
	{
		GraphicsCommandBuffers { parent: parent.clone(), internal: cbs }
	}
}
pub struct TransferCommandBuffers { parent: Rc<vk::CommandPool>, internal: Vec<VkCommandBuffer> }
impl std::ops::Drop for TransferCommandBuffers
{
	fn drop(&mut self)
	{
		unsafe { vkFreeCommandBuffers(self.parent.parent().get(), self.parent.get(), self.internal.len() as u32, self.internal.as_ptr()) };
	}
}
pub trait TransferCommandBuffersInternals { fn new(parent: &Rc<vk::CommandPool>, cbs: Vec<VkCommandBuffer>) -> Self; }
impl TransferCommandBuffersInternals for TransferCommandBuffers
{
	fn new(parent: &Rc<vk::CommandPool>, cbs: Vec<VkCommandBuffer>) -> Self
	{
		TransferCommandBuffers { parent: parent.clone(), internal: cbs }
	}
}
pub struct TransientTransferCommandBuffers<'a> { parent: &'a vk::CommandPool, queue: &'a vk::Queue, internal: Vec<VkCommandBuffer> }
impl <'a> std::ops::Drop for TransientTransferCommandBuffers<'a>
{
	fn drop(&mut self)
	{
		unsafe { vkFreeCommandBuffers(self.parent.parent().get(), self.parent.get(), self.internal.len() as u32, self.internal.as_ptr()) };
	}
}
pub trait TransientTransferCommandBuffersInternals<'a> { fn new(parent: &'a vk::CommandPool, queue: &'a vk::Queue, cbs: Vec<VkCommandBuffer>) -> Self; }
impl <'a> TransientTransferCommandBuffersInternals<'a> for TransientTransferCommandBuffers<'a>
{
	fn new(parent: &'a vk::CommandPool, queue: &'a vk::Queue, cbs: Vec<VkCommandBuffer>) -> Self
	{
		TransientTransferCommandBuffers { parent: parent, queue: queue, internal: cbs }
	}
}

pub trait PrimaryCommandBuffers<'a, Recorder: 'a>
{
	fn begin(&'a self, index: usize) -> Result<Recorder, EngineError>;
	fn begin_all(&'a self) -> Result<std::vec::IntoIter<(usize, Recorder)>, EngineError>;
}
impl <'a> PrimaryCommandBuffers<'a, GraphicsCommandRecorder<'a>> for GraphicsCommandBuffers
{
	fn begin(&'a self, index: usize) -> Result<GraphicsCommandRecorder, EngineError>
	{
		unsafe
		{
			vkBeginCommandBuffer(self.internal[index], &VkCommandBufferBeginInfo
			{
				sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(),
				flags: 0, pInheritanceInfo: std::ptr::null()
			}).map(|| GraphicsCommandRecorder { buffer_ref: Some(&self.internal[index]) }).map_err(EngineError::from)
		}
	}
	fn begin_all(&'a self) -> Result<std::vec::IntoIter<(usize, GraphicsCommandRecorder)>, EngineError>
	{
		self.internal.iter().enumerate().map(|(i, x)|
		unsafe {
			vkBeginCommandBuffer(*x, &VkCommandBufferBeginInfo
			{
				sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(),
				flags: 0, pInheritanceInfo: std::ptr::null()
			}).map(|| (i, GraphicsCommandRecorder { buffer_ref: Some(&x) }))
		}).collect::<Result<Vec<_>, _>>().map_err(EngineError::from).map(|x| x.into_iter())
	}
}
impl <'a> PrimaryCommandBuffers<'a, TransferCommandRecorder<'a>> for TransferCommandBuffers
{
	fn begin(&'a self, index: usize) -> Result<TransferCommandRecorder, EngineError>
	{
		unsafe
		{
			vkBeginCommandBuffer(self.internal[index], &VkCommandBufferBeginInfo
			{
				sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(),
				flags: 0, pInheritanceInfo: std::ptr::null()
			}).map(|| TransferCommandRecorder { buffer_ref: Some(&self.internal[index]) }).map_err(EngineError::from)
		}
	}
	fn begin_all(&'a self) -> Result<std::vec::IntoIter<(usize, TransferCommandRecorder)>, EngineError>
	{
		self.internal.iter().enumerate().map(|(i, x)|
		unsafe {
			vkBeginCommandBuffer(*x, &VkCommandBufferBeginInfo
			{
				sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(),
				flags: 0, pInheritanceInfo: std::ptr::null()
			}).map(|| (i, TransferCommandRecorder { buffer_ref: Some(&x) }))
		}).collect::<Result<Vec<_>, _>>().map_err(EngineError::from).map(|x| x.into_iter())
	}
}
impl <'a> PrimaryCommandBuffers<'a, TransferCommandRecorder<'a>> for TransientTransferCommandBuffers<'a>
{
	fn begin(&'a self, index: usize) -> Result<TransferCommandRecorder, EngineError>
	{
		unsafe
		{
			vkBeginCommandBuffer(self.internal[index], &VkCommandBufferBeginInfo
			{
				sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(),
				flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT, pInheritanceInfo: std::ptr::null()
			}).map(|| TransferCommandRecorder { buffer_ref: Some(&self.internal[index]) }).map_err(EngineError::from)
		}
	}
	fn begin_all(&'a self) -> Result<std::vec::IntoIter<(usize, TransferCommandRecorder)>, EngineError>
	{
		self.internal.iter().enumerate().map(|(i, x)|
		unsafe {
			vkBeginCommandBuffer(*x, &VkCommandBufferBeginInfo
			{
				sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(),
				flags: 0, pInheritanceInfo: std::ptr::null()
			}).map(|| (i, TransferCommandRecorder { buffer_ref: Some(&x) }))
		}).collect::<Result<Vec<_>, _>>().map_err(EngineError::from).map(|x| x.into_iter())
	}
}
impl <'a> TransientTransferCommandBuffers<'a>
{
	pub fn execute(self) -> Result<(), EngineError>
	{
		self.queue.submit_commands(&self.internal, &[], &[], &[], None).and_then(|()| self.queue.wait_for_idle()).map_err(EngineError::from)
	}
}
pub struct GraphicsCommandRecorder<'a> { buffer_ref: Option<&'a VkCommandBuffer> }
impl <'a> std::ops::Drop for GraphicsCommandRecorder<'a>
{
	fn drop(&mut self) { if let Some(&br) = self.buffer_ref { unsafe { vkEndCommandBuffer(br) }; } }
}
pub struct TransferCommandRecorder<'a> { buffer_ref: Option<&'a VkCommandBuffer> }
impl <'a> std::ops::Drop for TransferCommandRecorder<'a>
{
	fn drop(&mut self) { if let Some(&br) = self.buffer_ref { unsafe { vkEndCommandBuffer(br) }; } }
}
impl <'a> GraphicsCommandRecorder<'a>
{
	pub fn pipeline_barrier(self, src_stage_mask: VkPipelineStageFlags, dst_stage_mask: VkPipelineStageFlags,
		depend_by_region: bool, memory_barriers: &[MemoryBarrier], buffer_barriers: &[BufferMemoryBarrier], image_barriers: &[ImageMemoryBarrier]) -> Self
	{
		let (mbs_native, bbs_native, ibs_native) = (
			memory_barriers.into_iter().map(|x| x.into()).collect::<Vec<_>>(),
			buffer_barriers.into_iter().map(|x| x.into()).collect::<Vec<_>>(),
			image_barriers.into_iter().map(|x| x.into()).collect::<Vec<_>>()
		);
		unsafe { vkCmdPipelineBarrier(*self.buffer_ref.unwrap(), src_stage_mask, dst_stage_mask,
			if depend_by_region { VK_DEPENDENCY_BY_REGION_BIT } else { 0 },
			mbs_native.len() as u32, mbs_native.as_ptr(),
			bbs_native.len() as u32, bbs_native.as_ptr(),
			ibs_native.len() as u32, ibs_native.as_ptr()) };
		self
	}
	pub fn end(mut self) -> Result<(), EngineError>
	{
		unsafe { vkEndCommandBuffer(*self.buffer_ref.unwrap()) }.and_then(|| { self.buffer_ref = None; Ok(()) }).map_err(EngineError::from)
	}
}
impl <'a> TransferCommandRecorder<'a>
{
	pub fn pipeline_barrier(self, src_stage_mask: VkPipelineStageFlags, dst_stage_mask: VkPipelineStageFlags,
		depend_by_region: bool, memory_barriers: &[MemoryBarrier], buffer_barriers: &[BufferMemoryBarrier], image_barriers: &[ImageMemoryBarrier]) -> Self
	{
		let (mbs_native, bbs_native, ibs_native) = (
			memory_barriers.into_iter().map(|x| x.into()).collect::<Vec<_>>(),
			buffer_barriers.into_iter().map(|x| x.into()).collect::<Vec<_>>(),
			image_barriers.into_iter().map(|x| x.into()).collect::<Vec<_>>()
		);
		unsafe { vkCmdPipelineBarrier(*self.buffer_ref.unwrap(), src_stage_mask, dst_stage_mask,
			if depend_by_region { VK_DEPENDENCY_BY_REGION_BIT } else { 0 },
			mbs_native.len() as u32, mbs_native.as_ptr(),
			bbs_native.len() as u32, bbs_native.as_ptr(),
			ibs_native.len() as u32, ibs_native.as_ptr()) };
		self
	}
	pub fn end(mut self) -> Result<(), EngineError>
	{
		unsafe { vkEndCommandBuffer(*self.buffer_ref.unwrap()) }.and_then(|| { self.buffer_ref = None; Ok(()) }).map_err(EngineError::from)
	}
}
