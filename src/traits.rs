// Traits

use vkffi::*;
use render_vk::wrap as vk;
// use device_resources;
// use structures;

/// Indicates the object can produce window
pub trait WindowProvider<WindowHandle>
{
	fn create_unresizable_window(&self, size: VkExtent2D, title: &str) -> WindowHandle;
	fn show_window(&self, handle: WindowHandle);
}
/// Indicates the object can process message from system
pub trait MessageHandler
{
	fn process_messages(&self) -> bool;
}

/// Indicates the object is a placeholder of FFI objects
pub trait NativeOwner<InternalType>
{
	/// Gets native pointer for FFI objects
	fn get(&self) -> InternalType;
}

// Provides Reference to Parent object
pub trait HasParent
{
	type ParentRefType;
	fn parent(&self) -> &Self::ParentRefType;
}

pub trait DeviceStore
{
	fn required_sizes() -> Vec<VkDeviceSize>;
	// fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange);
}
/*
pub trait UniformStore
{
	fn initial_stage_data(&self, uniform_memory_ref: &mut structures::UniformMemory);
}
pub trait HasDescriptor
{
	fn write_descriptor_info<'d>(&self, sets: &device_resources::DescriptorSets<'d>) -> Vec<VkWriteDescriptorSet>;
}
*/
