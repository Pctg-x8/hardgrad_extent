#![allow(non_camel_case_types)]

// Vulkan C to Rust FFI functions

use vkffi::enums::*;
use vkffi::structs::*;
use vkffi::objects::*;
use vkffi::types::*;
use std::os::raw::*;
use libc::size_t;

#[link(name = "vulkan-1")]
extern "system"
{
	pub fn vkCreateInstance(pCreateInfo: *const VkInstanceCreateInfo, pAllocator: *const VkAllocationCallbacks, pInstance: *mut VkInstance) -> VkResult;
	pub fn vkDestroyInstance(instance: VkInstance, pAllocator: *const VkAllocationCallbacks);

	// Surface Extension //
	#[cfg(windows)]
	pub fn vkCreateWin32SurfaceKHR(instance: VkInstance, pCreateInfo: *const VkWin32SurfaceCreateInfoKHR, pAllocator: *const VkAllocationCallbacks, pSurface: *mut VkSurfaceKHR) -> VkResult;
	pub fn vkDestroySurfaceKHR(instance: VkInstance, ssurface: VkSurfaceKHR, pAllocator: *const VkAllocationCallbacks);
	pub fn vkGetPhysicalDeviceSurfaceSupportKHR(physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32, surface: VkSurfaceKHR, pSupported: *mut VkBool32) -> VkResult;
	pub fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceCapabilities: *mut VkSurfaceCapabilitiesKHR) -> VkResult;
	pub fn vkGetPhysicalDeviceSurfaceFormatsKHR(physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaecFormatCount: *mut u32, pSurfaceFormats: *mut VkSurfaceFormatKHR) -> VkResult;
	pub fn vkGetPhysicalDeviceSurfacePresentModesKHR(physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pPresentModeCount: *mut u32, pPresentModes: *mut VkPresentModeKHR) -> VkResult;

	// Swapchain Extension //
	pub fn vkCreateSwapchainKHR(device: VkDevice, pCreateInfo: *const VkSwapchainCreateInfoKHR, pAllocator: *const VkAllocationCallbacks, pSwapchain: *mut VkSwapchainKHR) -> VkResult;
	pub fn vkDestroySwapchainKHR(device: VkDevice, swapchain: VkSwapchainKHR, pAllocator: *const VkAllocationCallbacks);
	pub fn vkGetSwapchainImagesKHR(device: VkDevice, swapchain: VkSwapchainKHR, pSwapchainImageCount: *mut u32, pSwapchainImages: *mut VkImage) -> VkResult;
	pub fn vkAcquireNextImageKHR(device: VkDevice, swapchain: VkSwapchainKHR, timeout: u64, semaphore: VkSemaphore, fence: VkFence, pImageIndex: *mut u32) -> VkResult;
	pub fn vkQueuePresentKHR(queue: VkQueue, pPresentInfo: *const VkPresentInfoKHR) -> VkResult;
}

// Function Pointers
pub type PFN_vkAllocationFunction = unsafe extern "system" fn(pUserData: *mut c_void, size: size_t, alignment: size_t, allocationScope: VkSystemAllocationScope) -> *mut c_void;
pub type PFN_vkReallocationFunction = unsafe extern "system" fn(pUserData: *mut c_void, pOriginal: *mut c_void, size: size_t, alignment: size_t, allocationScope: VkSystemAllocationScope) -> *mut c_void;
pub type PFN_vkFreeFunction = unsafe extern "system" fn(pUserData: *mut c_void, pMemory: *mut c_void);
pub type PFN_vkInternalAllocationNotification = unsafe extern "system" fn(pUserData: *mut c_void, size: size_t, allocationType: VkInternalAllocationType, allocationScope: VkSystemAllocationScope);
pub type PFN_vkInternalFreeNotification = unsafe extern "system" fn(pUserData: *mut c_void, size: size_t, allocationType: VkInternalAllocationType, allocationScope: VkSystemAllocationScope);
