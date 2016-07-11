use std;
use vkffi::*;

pub trait ResultValueToObject where Self: std::marker::Sized { fn to_result(self) -> Result<(), Self>; }

pub trait CreationObject<StructureT> where Self: std::marker::Sized
{
	fn create(info: &StructureT) -> Result<Self, VkResult>;
}
pub trait MemoryAllocationRequired
{
	fn get_memory_requirements(&self) -> VkMemoryRequirements;
}
