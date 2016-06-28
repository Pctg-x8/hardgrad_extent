#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Vulkan C to Rust FFI Structs and Handles

#[cfg(not(windows))]
use std::os::raw::*;
use vkffi::enums::*;
use vkffi::types::*;
use vkffi::functions::*;

#[cfg(windows)]
use winapi::*;

#[repr(C)]
pub struct VkInstanceCreateInfo
{
	pub sType: VkStructureType, pub pNext: *const c_void,
	pub flags: VkInstanceCreateFlags, pub pApplicationInfo: *const VkApplicationInfo,
	pub enabledLayerCount: u32, pub ppEnabledLayerNames: *const *const c_char,
	pub enabledExtensionCount: u32, pub ppEnabledExtensionNames: *const *const c_char
}
#[repr(C)]
pub struct VkApplicationInfo
{
	pub sType: VkStructureType, pub pNext: *const c_void,
	pub pApplicationName: *const c_char, pub applicationVersion: u32,
	pub pEngineName: *const c_char, pub engineVersion: u32,
	pub apiVersion: u32
}
#[repr(C)]
pub struct VkAllocationCallbacks
{
	pub pUserData: *mut c_void,
	pub pfnAllocation: PFN_vkAllocationFunction,
	pub pfnReallocation: PFN_vkReallocationFunction,
	pub fnFree: PFN_vkFreeFunction,
	pub pfnInternalAllocation: PFN_vkInternalAllocationNotification,
	pub pfnInternalFree: PFN_vkInternalFreeNotification
}

#[repr(C)] #[cfg(windows)]
pub struct VkWin32SurfaceCreateInfoKHR
{
	pub sType: VkStructureType, pub pNext: *const c_void,
	pub flags: VkWin32SurfaceCreateFlagsKHR, pub hinstance: HINSTANCE,
	pub hwnd: HWND
}
