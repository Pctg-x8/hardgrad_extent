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
	pub buffer: &'a BufferResource, pub range: std::ops::Range<VkDeviceSize>
}
impl <'a> BufferMemoryBarrier<'a>
{
	pub fn hold_ownership(buf: &'a BufferResource, range: std::ops::Range<VkDeviceSize>,
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
			buffer: self.buffer.get_resource(), offset: self.range.start, size: self.range.end - self.range.start
		}
	}
}
pub struct ImageMemoryBarrierTemplate<'a>
{
	image: &'a ImageResource, subresource_range: ImageSubresourceRange
}
impl <'a> ImageMemoryBarrierTemplate<'a>
{
	pub fn hold_ownership(&self, src_access_mask: VkAccessFlags, dst_access_mask: VkAccessFlags,
		src_layout: VkImageLayout, dst_layout: VkImageLayout) -> ImageMemoryBarrier<'a>
	{
		ImageMemoryBarrier::hold_ownership(self.image, self.subresource_range.clone(), src_access_mask, dst_access_mask, src_layout, dst_layout)
	}
}
pub struct ImageMemoryBarrier<'a>
{
	src_access_mask: VkAccessFlags, dst_access_mask: VkAccessFlags,
	src_layout: VkImageLayout, dst_layout: VkImageLayout,
	src_queue_family_index: u32, dst_queue_family_index: u32,
	image: &'a ImageResource, subresource_range: ImageSubresourceRange
}
impl <'a> ImageMemoryBarrier<'a>
{
	pub fn template(img: &'a ImageResource, subresource_range: ImageSubresourceRange) -> ImageMemoryBarrierTemplate<'a>
	{
		ImageMemoryBarrierTemplate { image: img, subresource_range: subresource_range }
	}
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

pub struct IndirectCallParameter(pub u32, pub u32, pub u32, pub u32);		// vertex_count, instance_count, first_vertex, first_instance

pub type GraphicsCommandBuffer = VkCommandBuffer;
pub type GraphicsCommandBuffersView = [GraphicsCommandBuffer];
pub type BundledCommandBuffersView = [VkCommandBuffer];

pub struct GraphicsCommandBuffers { parent: Rc<vk::CommandPool>, internal: Vec<VkCommandBuffer> }
impl std::ops::Drop for GraphicsCommandBuffers
{
	fn drop(&mut self)
	{
		unsafe { vkFreeCommandBuffers(self.parent.parent().get(), self.parent.get(), self.internal.len() as u32, self.internal.as_ptr()) };
	}
}
impl std::ops::Deref for GraphicsCommandBuffers
{
	type Target = GraphicsCommandBuffersView;
	fn deref(&self) -> &Self::Target { &self.internal }
}
impl InternalExports<Vec<VkCommandBuffer>> for GraphicsCommandBuffers { fn get_internal(&self) -> &Vec<VkCommandBuffer> { &self.internal } }
pub trait GraphicsCommandBuffersInternals { fn new(parent: &Rc<vk::CommandPool>, cbs: Vec<VkCommandBuffer>) -> Self; }
impl GraphicsCommandBuffersInternals for GraphicsCommandBuffers
{
	fn new(parent: &Rc<vk::CommandPool>, cbs: Vec<VkCommandBuffer>) -> Self
	{
		GraphicsCommandBuffers { parent: parent.clone(), internal: cbs }
	}
}
pub struct BundledCommandBuffers { parent: Rc<vk::CommandPool>, internal: Vec<VkCommandBuffer> }
impl std::ops::Drop for BundledCommandBuffers
{
	fn drop(&mut self)
	{
		unsafe { vkFreeCommandBuffers(self.parent.parent().get(), self.parent.get(), self.internal.len() as u32, self.internal.as_ptr()) };
	}
}
impl std::ops::Deref for BundledCommandBuffers
{
	type Target = BundledCommandBuffersView;
	fn deref(&self) -> &Self::Target { &self.internal }
}
impl InternalExports<Vec<VkCommandBuffer>> for BundledCommandBuffers { fn get_internal(&self) -> &Vec<VkCommandBuffer> { &self.internal } }
pub trait BundledCommandBuffersInternals { fn new(parent: &Rc<vk::CommandPool>, cbs: Vec<VkCommandBuffer>) -> Self; }
impl BundledCommandBuffersInternals for BundledCommandBuffers
{
	fn new(parent: &Rc<vk::CommandPool>, cbs: Vec<VkCommandBuffer>) -> Self
	{
		BundledCommandBuffers { parent: parent.clone(), internal: cbs }
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
impl InternalExports<Vec<VkCommandBuffer>> for TransferCommandBuffers { fn get_internal(&self) -> &Vec<VkCommandBuffer> { &self.internal } }
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
pub trait SecondaryCommandBuffers<'a, Recorder: 'a>
{
	fn begin(&'a self, index: usize, cont_rp: &RenderPass, subindex: u32) -> Result<Recorder, EngineError>;
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
impl <'a> SecondaryCommandBuffers<'a, BundleCommandRecorder<'a>> for BundledCommandBuffers
{
	fn begin(&'a self, index: usize, cont_rp: &RenderPass, subindex: u32) -> Result<BundleCommandRecorder, EngineError>
	{
		let inheritance_info = VkCommandBufferInheritanceInfo
		{
			sType: VkStructureType::CommandBufferInheritanceInfo, pNext: std::ptr::null(),
			renderPass: cont_rp.get_internal().get(), subpass: subindex, framebuffer: std::ptr::null_mut(),
			occlusionQueryEnable: false as VkBool32, queryFlags: 0, pipelineStatistics: 0
		};
		unsafe
		{
			vkBeginCommandBuffer(self.internal[index], &VkCommandBufferBeginInfo
			{
				sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(),
				flags: VK_COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT, pInheritanceInfo: &inheritance_info
			}).map(|| BundleCommandRecorder { buffer_ref: Some(&self.internal[index]) }).map_err(EngineError::from)
		}
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
				flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT, pInheritanceInfo: std::ptr::null()
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
pub struct BundleCommandRecorder<'a> { buffer_ref: Option<&'a VkCommandBuffer> }
impl <'a> std::ops::Drop for BundleCommandRecorder<'a>
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

	pub fn begin_render_pass(self, framebuffer: &Framebuffer, clear_values: &[AttachmentClearValue], use_bundles: bool) -> Self
	{
		let clear_values_native = clear_values.into_iter().map(|x| x.into()).collect::<Vec<_>>();
		let begin_info = VkRenderPassBeginInfo
		{
			sType: VkStructureType::RenderPassBeginInfo, pNext: std::ptr::null(),
			renderPass: framebuffer.get_mold().get(), framebuffer: framebuffer.get_internal().get(),
			renderArea: VkRect2D(VkOffset2D(0, 0), framebuffer.get_area()),
			clearValueCount: clear_values_native.len() as u32, pClearValues: clear_values_native.as_ptr()
		};
		unsafe { vkCmdBeginRenderPass(*self.buffer_ref.unwrap(), &begin_info,
			if use_bundles { VkSubpassContents::SecondaryCommandBuffers } else { VkSubpassContents::Inline }) };
		self
	}
	pub fn end_render_pass(self) -> Self
	{
		unsafe { vkCmdEndRenderPass(*self.buffer_ref.unwrap()) };
		self
	}

	pub fn bind_pipeline(self, pipeline: &GraphicsPipeline) -> Self
	{
		unsafe { vkCmdBindPipeline(*self.buffer_ref.unwrap(), VkPipelineBindPoint::Graphics, pipeline.get_internal().get()) };
		self
	}
	pub fn bind_descriptor_sets(self, layout: &PipelineLayout, sets: &DescriptorSetArrayView) -> Self
	{
		unsafe { vkCmdBindDescriptorSets(*self.buffer_ref.unwrap(), VkPipelineBindPoint::Graphics, layout.get_internal().get(), 0,
			sets.len() as u32, sets.as_ptr(), 0, std::ptr::null()) };
		self
	}
	pub fn bind_vertex_buffers(self, buffer_offsets: &[(&BufferResource, usize)]) -> Self
	{
		// :Specialized:
		self.bind_vertex_buffers_partial(0, buffer_offsets)
	}
	pub fn bind_vertex_buffers_partial(self, start_binding: u32, buffer_offsets: &[(&BufferResource, usize)]) -> Self
	{
		let (buffer_native, offsets_native): (Vec<_>, Vec<_>) = buffer_offsets.into_iter()
			.map(|&(b, v)| (b.get_resource(), v as VkDeviceSize)).unzip();
		unsafe { vkCmdBindVertexBuffers(*self.buffer_ref.unwrap(), start_binding, buffer_native.len() as u32, buffer_native.as_ptr(), offsets_native.as_ptr()) };
		self
	}
	pub fn bind_index_buffer(self, buffer: &BufferResource, offset: usize) -> Self
	{
		unsafe { vkCmdBindIndexBuffer(*self.buffer_ref.unwrap(), buffer.get_resource(), offset as VkDeviceSize, VkIndexType::U16) };
		self
	}
	pub fn push_constants(self, layout: &PipelineLayout, shader_stage: &[ShaderStage], range: std::ops::Range<u32>, data: &[f32]) -> Self
	{
		let stages = shader_stage.into_iter().fold(0, |acc, x| acc | Into::<VkShaderStageFlags>::into(*x));
		unsafe { vkCmdPushConstants(*self.buffer_ref.unwrap(), layout.get_internal().get(), stages,
			range.start, range.len() as u32, data.as_ptr() as *const std::os::raw::c_void) };
		self
	}

	pub fn draw(self, vertex_count: u32, instance_count: u32) -> Self
	{
		unsafe { vkCmdDraw(*self.buffer_ref.unwrap(), vertex_count, instance_count, 0, 0) };
		self
	}
	pub fn draw_indexed(self, index_count: u32, instance_count: u32, index_offset: u32) -> Self
	{
		unsafe { vkCmdDrawIndexed(*self.buffer_ref.unwrap(), index_count, instance_count, 0, index_offset, 0) };
		self
	}
	pub fn draw_indirect(self, param_buffer: &BufferResource, param_offs: usize) -> Self
	{
		unsafe { vkCmdDrawIndirect(*self.buffer_ref.unwrap(), param_buffer.get_resource(), param_offs as VkDeviceSize, 1, 0) };
		self
	}

	pub fn execute_commands(self, buffers: &BundledCommandBuffersView) -> Self
	{
		unsafe { vkCmdExecuteCommands(*self.buffer_ref.unwrap(), buffers.len() as u32, buffers.as_ptr()) };
		self
	}
	pub fn inject_commands<F>(self, f: F) -> Self where F: FnOnce(Self) -> Self
	{
		f(self)
	}
}
impl <'a> BundleCommandRecorder<'a>
{
	pub fn end(mut self) -> Result<(), EngineError>
	{
		unsafe { vkEndCommandBuffer(*self.buffer_ref.unwrap()) }.and_then(|| { self.buffer_ref = None; Ok(()) }).map_err(EngineError::from)
	}

	pub fn bind_pipeline(self, pipeline: &GraphicsPipeline) -> Self
	{
		unsafe { vkCmdBindPipeline(*self.buffer_ref.unwrap(), VkPipelineBindPoint::Graphics, pipeline.get_internal().get()) };
		self
	}
	pub fn bind_descriptor_sets(self, layout: &PipelineLayout, sets: &DescriptorSetArrayView) -> Self
	{
		unsafe { vkCmdBindDescriptorSets(*self.buffer_ref.unwrap(), VkPipelineBindPoint::Graphics, layout.get_internal().get(), 0,
			sets.len() as u32, sets.as_ptr(), 0, std::ptr::null()) };
		self
	}
	pub fn bind_vertex_buffers(self, buffer_offsets: &[(&BufferResource, usize)]) -> Self
	{
		// :Specialized:
		self.bind_vertex_buffers_partial(0, buffer_offsets)
	}
	pub fn bind_vertex_buffers_partial(self, start_binding: u32, buffer_offsets: &[(&BufferResource, usize)]) -> Self
	{
		let (buffer_native, offsets_native): (Vec<_>, Vec<_>) = buffer_offsets.into_iter()
			.map(|&(b, v)| (b.get_resource(), v as VkDeviceSize)).unzip();
		unsafe { vkCmdBindVertexBuffers(*self.buffer_ref.unwrap(), start_binding, buffer_native.len() as u32, buffer_native.as_ptr(), offsets_native.as_ptr()) };
		self
	}
	pub fn bind_index_buffer(self, buffer: &BufferResource, offset: usize) -> Self
	{
		unsafe { vkCmdBindIndexBuffer(*self.buffer_ref.unwrap(), buffer.get_resource(), offset as VkDeviceSize, VkIndexType::U16) };
		self
	}
	pub fn push_constants(self, layout: &PipelineLayout, shader_stage: &[ShaderStage], range: std::ops::Range<u32>, data: &[f32]) -> Self
	{
		let stages = shader_stage.into_iter().fold(0, |acc, x| acc | Into::<VkShaderStageFlags>::into(*x));
		unsafe { vkCmdPushConstants(*self.buffer_ref.unwrap(), layout.get_internal().get(), stages,
			range.start, range.len() as u32, data.as_ptr() as *const std::os::raw::c_void) };
		self
	}

	pub fn draw(self, vertex_count: u32, instance_count: u32) -> Self
	{
		unsafe { vkCmdDraw(*self.buffer_ref.unwrap(), vertex_count, instance_count, 0, 0) };
		self
	}
	pub fn draw_indexed(self, index_count: u32, instance_count: u32, index_offset: u32) -> Self
	{
		unsafe { vkCmdDrawIndexed(*self.buffer_ref.unwrap(), index_count, instance_count, 0, index_offset, 0) };
		self
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
		unsafe { vkEndCommandBuffer(*self.buffer_ref.unwrap()) }.and_then(||
		{
			self.buffer_ref = None;
			Ok(())
		}).map_err(EngineError::from)
	}

	pub fn copy_buffer(self, src: &BufferResource, dst: &BufferResource, regions: &[BufferCopyRegion]) -> Self
	{
		let regions_native = regions.into_iter().map(|&x| x.into()).collect::<Vec<_>>();
		unsafe { vkCmdCopyBuffer(*self.buffer_ref.unwrap(), src.get_resource(), dst.get_resource(),
			regions_native.len() as u32, regions_native.as_ptr()) };
		self
	}
	pub fn copy_image(self, src: &ImageResource, dst: &ImageResource, src_layout: VkImageLayout, dst_layout: VkImageLayout, regions: &[ImageCopyRegion]) -> Self
	{
		let regions_native = regions.into_iter().map(|&x| x.into()).collect::<Vec<_>>();
		unsafe { vkCmdCopyImage(*self.buffer_ref.unwrap(), src.get_resource(), src_layout, dst.get_resource(), dst_layout,
			regions_native.len() as u32, regions_native.as_ptr()) };
		self
	}
}

#[derive(Clone, Copy)]
pub struct BufferCopyRegion(pub usize, pub usize, pub usize);		// src, dst, size
impl std::convert::Into<VkBufferCopy> for BufferCopyRegion
{
	fn into(self) -> VkBufferCopy
	{
		let BufferCopyRegion(src, dst, size) = self;
		VkBufferCopy(src as VkDeviceSize, dst as VkDeviceSize, size as VkDeviceSize)
	}
}
#[derive(Clone, Copy)]
// src_layers, src_offset, dst_layers, dst_offset, extent
pub struct ImageCopyRegion(pub ImageSubresourceLayers, pub VkOffset3D, pub ImageSubresourceLayers, pub VkOffset3D, pub VkExtent3D);
impl std::convert::Into<VkImageCopy> for ImageCopyRegion
{
	fn into(self) -> VkImageCopy
	{
		let ImageCopyRegion(sl, so, dl, _do, ex) = self;
		VkImageCopy(sl.into(), so, dl.into(), _do, ex)
	}
}
