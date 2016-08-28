
mod error;
mod engine;
mod device;
mod command;
mod resource;
mod framebuffer;
mod synchronize;
mod shading;
mod window;
mod descriptor;
mod internal_traits;

mod debug_info;

// Exported APIs //
pub use self::error::*;
pub use self::engine::DeviceFeatures;
pub use self::framebuffer::{AttachmentDesc, PassDesc, PassDependency, AttachmentClearValue};
pub use self::command::{MemoryBarrier, BufferMemoryBarrier, ImageMemoryBarrier, IndirectCallParameter, BufferCopyRegion, ImageCopyRegion};
pub use self::resource::{
	ImageSubresourceRange, ImageSubresourceLayers, BufferDataType, ImageUsagePresets,
	ImageDescriptor1, ImageDescriptor2, ImageDescriptor3, ImagePreallocator,
	SamplerState, ComponentSwizzle, ComponentMapping
};
pub use self::shading::{
	VertexBinding, VertexAttribute, PushConstantDesc,
	PrimitiveTopology, ViewportWithScissorRect, RasterizerState, AttachmentBlendState,
	GraphicsPipelineBuilder
};
pub use self::descriptor::{ShaderStage, Descriptor, BufferInfo, ImageInfo, DescriptorSetWriteInfo};
pub use self::debug_info::DebugLine;
pub mod traits
{
	pub use super::command::{PrimaryCommandBuffers, SecondaryCommandBuffers};
	pub use super::resource::{ImageDescriptor};
}
// exported objects
pub use self::engine::Engine;
pub use self::synchronize::{QueueFence, Fence};
pub use self::framebuffer::{RenderPass, Framebuffer};
pub use self::command::{GraphicsCommandBuffers, BundledCommandBuffers, TransferCommandBuffers, TransientTransferCommandBuffers};
pub use self::resource::{Buffer, Image1D, Image2D, Image3D, LinearImage2D, DeviceBuffer, StagingBuffer, DeviceImage, StagingImage};
pub use self::shading::{PipelineLayout, GraphicsPipeline};
pub use self::descriptor::{DescriptorSetLayout, DescriptorSets};
pub use self::debug_info::DebugInfo;

// For internal exports //
mod internals
{
	pub use super::internal_traits::*;
	pub use super::engine::*;
	pub use super::window::*;
	pub use super::error::*;
	pub use super::device::*;
	pub use super::command::*;
	pub use super::resource::*;
	pub use super::framebuffer::*;
	pub use super::synchronize::*;
	pub use super::shading::*;
	pub use super::descriptor::*;
	pub use super::debug_info::*;
}
