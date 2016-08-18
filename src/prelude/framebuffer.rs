// Prelude: RenderPass and Framebuffer

use prelude::internals::*;
use std;
use vkffi::*;
use render_vk::wrap as vk;
use std::rc::Rc;

pub struct AttachmentDesc
{
	pub format: VkFormat, pub samples: VkSampleCountFlagBits,
	pub clear_on_load: Option<bool>, pub preserve_stored_value: bool,
	pub stencil_clear_on_load: Option<bool>, pub preserve_stored_stencil_value: bool,
	pub initial_layout: VkImageLayout, pub final_layout: VkImageLayout
}
impl std::default::Default for AttachmentDesc
{
	fn default() -> Self
	{
		AttachmentDesc
		{
			format: VkFormat::UNDEFINED, samples: VK_SAMPLE_COUNT_1_BIT,
			clear_on_load: None, preserve_stored_value: false,
			stencil_clear_on_load: None, preserve_stored_stencil_value: false,
			initial_layout: VkImageLayout::Undefined, final_layout: VkImageLayout::Undefined
		}
	}
}
impl <'a> std::convert::Into<VkAttachmentDescription> for &'a AttachmentDesc
{
	fn into(self) -> VkAttachmentDescription
	{
		VkAttachmentDescription
		{
			flags: 0, format: self.format, samples: self.samples,
			loadOp: self.clear_on_load.map(|b| if b { VkAttachmentLoadOp::Clear } else { VkAttachmentLoadOp::Load })
				.unwrap_or(VkAttachmentLoadOp::DontCare),
			stencilLoadOp: self.stencil_clear_on_load.map(|b| if b { VkAttachmentLoadOp::Clear } else { VkAttachmentLoadOp::Load })
				.unwrap_or(VkAttachmentLoadOp::DontCare),
			storeOp: if self.preserve_stored_value { VkAttachmentStoreOp::Store } else { VkAttachmentStoreOp::DontCare },
			stencilStoreOp: if self.preserve_stored_stencil_value { VkAttachmentStoreOp::Store } else { VkAttachmentStoreOp::DontCare },
			initialLayout: self.initial_layout, finalLayout: self.final_layout
		}
	}
}
pub type AttachmentRef = VkAttachmentReference;
impl AttachmentRef
{
	pub fn color(index: u32) -> Self { VkAttachmentReference(index, VkImageLayout::ColorAttachmentOptimal) }
}
pub struct PassDesc
{
	pub input_attachment_indices: Vec<AttachmentRef>,
	pub color_attachment_indices: Vec<AttachmentRef>,
	pub resolved_attachment_indices: Option<Vec<AttachmentRef>>,
	pub depth_stencil_attachment_index: Option<AttachmentRef>,
	pub preserved_attachment_indices: Vec<u32>
}
impl std::default::Default for PassDesc
{
	fn default() -> Self
	{
		PassDesc
		{
			input_attachment_indices: Vec::new(),
			color_attachment_indices: Vec::new(),
			resolved_attachment_indices: None,
			depth_stencil_attachment_index: None,
			preserved_attachment_indices: Vec::new()
		}
	}
}
impl PassDesc
{
	pub fn single_fragment_output(index: u32) -> PassDesc
	{
		PassDesc { color_attachment_indices: vec![AttachmentRef::color(index)], .. Default::default() }
	}
}
impl <'a> std::convert::Into<VkSubpassDescription> for &'a PassDesc
{
	fn into(self) -> VkSubpassDescription
	{
		VkSubpassDescription
		{
			flags: 0, pipelineBindPoint: VkPipelineBindPoint::Graphics,
			inputAttachmentCount: self.input_attachment_indices.len() as u32,
			pInputAttachments: self.input_attachment_indices.as_ptr(),
			colorAttachmentCount: self.color_attachment_indices.len() as u32,
			pColorAttachments: self.color_attachment_indices.as_ptr(),
			pResolveAttachments: self.resolved_attachment_indices.as_ref().map(|x| x.as_ptr()).unwrap_or(std::ptr::null()),
			pDepthStencilAttachment: self.depth_stencil_attachment_index.as_ref().map(|x| x as *const AttachmentRef).unwrap_or(std::ptr::null_mut()),
			preserveAttachmentCount: self.preserved_attachment_indices.len() as u32,
			pPreserveAttachments: self.preserved_attachment_indices.as_ptr()
		}
	}
}
pub struct PassDependency
{
	pub src: u32, pub dst: u32,
	pub src_stage_mask: VkPipelineStageFlags, pub dst_stage_mask: VkPipelineStageFlags,
	pub src_access_mask: VkAccessFlags, pub dst_access_mask: VkAccessFlags,
	pub depend_by_region: bool
}
impl std::default::Default for PassDependency
{
	fn default() -> Self
	{
		PassDependency
		{
			src: 0, dst: 0,
			src_stage_mask: VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
			dst_stage_mask: VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
			src_access_mask: VK_ACCESS_MEMORY_READ_BIT,
			dst_access_mask: VK_ACCESS_MEMORY_READ_BIT,
			depend_by_region: false
		}
	}
}
impl <'a> std::convert::Into<VkSubpassDependency> for &'a PassDependency
{
	fn into(self) -> VkSubpassDependency
	{
		VkSubpassDependency
		{
			srcSubpass: self.src, dstSubpass: self.dst,
			srcStageMask: self.src_stage_mask, dstStageMask: self.dst_stage_mask,
			srcAccessMask: self.src_access_mask, dstAccessMask: self.dst_access_mask,
			dependencyFlags: if self.depend_by_region { VK_DEPENDENCY_BY_REGION_BIT } else { 0 }
		}
	}
}
pub struct RenderPass { internal: Rc<vk::RenderPass> }
pub struct Framebuffer { #[allow(dead_code)] mold: Rc<vk::RenderPass>, internal: vk::Framebuffer }
impl InternalExports<Rc<vk::RenderPass>> for RenderPass { fn get_internal(&self) -> &Rc<vk::RenderPass> { &self.internal } }
impl InternalExports<vk::Framebuffer> for Framebuffer { fn get_internal(&self) -> &vk::Framebuffer { &self.internal } }
pub trait RenderPassInternals
{
	fn new(rp: vk::RenderPass) -> Self;
}
impl RenderPassInternals for RenderPass
{
	fn new(rp: vk::RenderPass) -> Self { RenderPass { internal: Rc::new(rp) } }
}
pub trait FramebufferInternals
{
	fn new(fb: vk::Framebuffer, mold: &Rc<vk::RenderPass>) -> Self;
}
impl FramebufferInternals for Framebuffer
{
	fn new(fb: vk::Framebuffer, mold: &Rc<vk::RenderPass>) -> Self
	{
		Framebuffer { internal: fb, mold: mold.clone() }
	}
}
