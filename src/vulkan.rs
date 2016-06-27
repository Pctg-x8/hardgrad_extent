// Safety Vulkan Modules

use vkffi::*;
use ::std;

pub trait CreationObject<StructureT> where Self: std::marker::Sized
{
	fn create(info: &StructureT) -> Result<Self, VkResult>;
}

pub struct Instance
{
	obj: VkInstance
}
impl CreationObject<VkInstanceCreateInfo> for Instance
{
	fn create(info: &VkInstanceCreateInfo) -> Result<Self, VkResult>
	{
		let mut i: VkInstance = std::ptr::null_mut();
		let res = unsafe { vkCreateInstance(info, std::ptr::null_mut(), &mut i) };
		if res != VkResult::Success { Err(res) } else { Ok(Instance { obj: i }) }
	}
}
impl std::ops::Drop for Instance
{
	fn drop(&mut self) { unsafe { vkDestroyInstance(self.obj, std::ptr::null_mut()) }; }
}
