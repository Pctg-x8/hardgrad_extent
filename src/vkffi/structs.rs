#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Vulkan C to Rust FFI Structs and Handles

#[cfg(not(windows))]
use std::os::raw::*;
use vkffi::enums::*;
use vkffi::types::*;
use vkffi::functions::*;
use vkffi::objects::*;

#[cfg(windows)]
use winapi::*;

// Basic Types //
#[repr(C)] pub struct VkOffset2D(i32, i32);
#[repr(C)] pub struct VkExtent2D(u32, u32);
#[repr(C)] pub struct VkRect2D(VkOffset2D, VkExtent2D);

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
#[repr(C)]
pub struct VkDeviceQueueCreateInfo
{
	pub sType: VkStructureType,
	pub pNext: *const c_void,
	pub flags: VkDeviceQueueCreateFlags,
	pub queueFamilyIndex: u32,
	pub queueCount: u32,pub pQueuePriorities: *const f32
}
#[repr(C)]
pub struct VkDeviceCreateInfo
{
	pub sType: VkStructureType,
	pub pNext: *const c_void,
	pub flags: VkDeviceCreateFlags,
	pub queueCreateInfoCount: u32,
	pub pQueueCreateInfos: *const VkDeviceQueueCreateInfo,
	pub enabledLayerCount: u32,
	pub ppEnabledLayerNames: *const *const c_char,
	pub enabledExtensionCount: u32,
	pub ppEnabledExtensionNames: *const *const c_char,
	pub pEnabledFeatures: *const VkPhysicalDeviceFeatures
}

#[repr(C)] #[cfg(windows)]
pub struct VkWin32SurfaceCreateInfoKHR
{
	pub sType: VkStructureType, pub pNext: *const c_void,
	pub flags: VkWin32SurfaceCreateFlagsKHR, pub hinstance: HINSTANCE,
	pub hwnd: HWND
}
#[repr(C)]
pub struct VkSurfaceCapabilitiesKHR
{
	pub minImageCount: u32,
	pub maxImageCount: u32,
	pub currentExtent: VkExtent2D,
	pub minImageExtent: VkExtent2D,
	pub maxImageExtent: VkExtent2D,
	pub maxImageArrayLayers: u32,
	pub supportedTransforms: VkSurfaceTransformFlagsKHR,
	pub currentTransform: VkSurfaceTransformFlagBitsKHR,
	pub supportedCompositeAlpha: VkCompositeAlphaFlagsKHR,
	pub supportedUsageFlags: VkImageUsageFlags
}
#[repr(C)]
pub struct VkSurfaceFormatKHR
{
	pub format: VkFormat,
	pub colorSpace: VkColorSpaceKHR
}

#[repr(C)]
pub struct VkSwapchainCreateInfoKHR
{
	pub sType: VkStructureType,
	pub pNext: *const c_void,
	pub flags: VkSwapchainCreateFlagsKHR,
	pub surface: VkSurfaceKHR,
	pub minImageCount: u32,
	pub imageFormat: VkFormat,
	pub imageColorSpace: VkColorSpaceKHR,
	pub imageExtent: VkExtent2D,
	pub imageArrayLayers: u32,
	pub imageUsag: VkImageUsageFlags,
	pub imageSharingMode: VkSharingMode,
	pub queueFamilyIndexCount: u32,
	pub pQueueFamilyIndices: *const u32,
	pub preTransform: VkSurfaceTransformFlagBitsKHR,
	pub compositeAlpha: VkCompositeAlphaFlagBitsKHR,
	pub presentMode: VkPresentModeKHR,
	pub clipped: VkBool32,
	pub oldSwapchain: VkSwapchainKHR
}
#[repr(C)]
pub struct VkPresentInfoKHR
{
	pub sType: VkStructureType,
	pub pNext: *const c_void,
	pub waitSemaphoreCount: u32,
	pub pWaitSemaphores: *const VkSemaphore,
	pub swapchainCount: u32,
	pub pSwapchains: *const VkSwapchainKHR,
	pub pImageIndices: *const u32,
	pub pResults: *mut VkResult
}
