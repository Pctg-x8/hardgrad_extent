
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
pub use self::engine::Engine;
pub use self::synchronize::{QueueFence, Fence};
pub use self::framebuffer::{AttachmentDesc, PassDesc, PassDependency, AttachmentClearValue};
pub use self::command::{MemoryBarrier, BufferMemoryBarrier, ImageMemoryBarrier, BufferCopyRegion};
pub use self::resource::{
	ImageSubresourceRange, BufferDataType, ImageUsagePresets,
	ImageDescriptor1, ImageDescriptor2, ImageDescriptor3, ResourcePreallocator
};
pub use self::shading::{
	VertexBinding, VertexAttribute, PushConstantDesc,
	PrimitiveTopology, ViewportWithScissorRect, RasterizerState, AttachmentBlendState,
	GraphicsPipelineBuilder
};
pub use self::descriptor::{ShaderStage, Descriptor, BufferInfo, DescriptorSetWriteInfo};
pub use self::debug_info::DebugInfo;
pub mod traits
{
	pub use super::command::{PrimaryCommandBuffers};
	pub use super::resource::{ImageDescriptor};
}

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
