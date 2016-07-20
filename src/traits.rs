// Traits

use vkffi::*;
use render_vk::wrap as vk;
use device_resources;

/// Indicates the object can process message from system
pub trait MessageHandler
{
	fn process_messages(&self) -> bool;
}

// Provides Internal Pointer type(for wrapper objects)
pub trait InternalProvider<InternalType>
{
	fn get(&self) -> InternalType;
}
// Provides Reference to Parent object
pub trait HasParent
{
	type ParentRefType;
	fn parent(&self) -> Self::ParentRefType;
}

pub trait DeviceStore
{
	fn device_size() -> VkDeviceSize;
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange);
}
pub trait HasDescriptor
{
	fn write_descriptor_info<'d>(&self, sets: &device_resources::DescriptorSets<'d>) -> Vec<VkWriteDescriptorSet>;
}
