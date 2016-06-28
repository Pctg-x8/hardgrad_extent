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

#[cfg(feature = "use_win32")]
use winapi::*;

pub struct Surface<'a>
{
	instance_ref: &'a Instance,
	obj: VkSurfaceKHR
}
impl <'a> Surface<'a>
{
	#[cfg(feature = "use_win32")]
	fn create(instance: &'a Instance, target: HWND) -> Result<Self, VkResult>
	{
		let mut obj: VkSurfaceKHR = std::ptr::null_mut();
		let info = VkWin32SurfaceCreateInfoKHR
		{
			sType: VkStructureType::Win32SurfaceCreateInfoKHR,
			pNext: std::ptr::null(), flags: 0,
			hinstance: unsafe { GetModuleHandleW(std::ptr::null_mut()) },
			hwnd: target
		};
		let res = unsafe { vkCreateWin32SurfaceKHR(instance.obj, &info, std::ptr::null(), &obj) };
		if res != VkResult::Success { Err(res) } else { Ok(Surface { instance_ref: instance, obj: obj }) }
	}
}
impl <'a> std::ops::Drop for Surface<'a>
{
	fn drop(&mut self) { unsafe { vkDestorySurfaceKHR(self.instance_ref, self.obj, std::ptr::null()) }; }
}
