use std;
use vkffi::*;

pub trait ResultValueToObject where Self: std::marker::Sized
{
	fn to_result(self) -> Result<(), Self>;
	fn and_then<F, T>(self, f: F) -> Result<T, Self> where F: FnOnce() -> Result<T, Self>;
	fn map<F, T>(self, f: F) -> Result<T, Self> where F: FnOnce() -> T;
}

pub trait MemoryAllocationRequired
{
	fn get_memory_requirements(&self) -> VkMemoryRequirements;
}
pub trait OnDeviceMemory
{
	type RangeType: std::marker::Sized;
	type StructureType: std::marker::Sized;
	fn memory_barrier(&self, range: Self::RangeType, src_access_mask: VkAccessFlags, dst_access_mask: VkAccessFlags) -> Self::StructureType;
}
