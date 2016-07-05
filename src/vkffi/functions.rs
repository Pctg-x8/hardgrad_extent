#![allow(non_camel_case_types)]
#![allow(dead_code)]

// Vulkan C to Rust FFI functions

use vkffi::enums::*;
use vkffi::structs::*;
use vkffi::objects::*;
use vkffi::types::*;
use std::os::raw::*;
use libc::size_t;
use xcb;
use xcb::ffi::*;

pub type PFN_vkVoidFunction = unsafe extern "system" fn();

#[link(name = "vulkan")]
extern "system"
{
	pub fn vkCreateInstance(pCreateInfo: *const VkInstanceCreateInfo, pAllocator: *const VkAllocationCallbacks, pInstance: *mut VkInstance) -> VkResult;
	pub fn vkDestroyInstance(instance: VkInstance, pAllocator: *const VkAllocationCallbacks);
	pub fn vkGetInstanceProcAddr(instance: VkInstance, pName: *const c_char) -> PFN_vkVoidFunction;
	pub fn vkEnumeratePhysicalDevices(instance: VkInstance, pPhysicalDeviceCount: *mut u32, pPhysicalDevices: *mut VkPhysicalDevice) -> VkResult;
	pub fn vkGetPhysicalDeviceFeatures(physicalDevice: VkPhysicalDevice, pFeatures: *mut VkPhysicalDeviceFeatures);
	pub fn vkGetPhysicalDeviceProperties(physicalDevice: VkPhysicalDevice, pProperties: *mut VkPhysicalDeviceProperties);
	pub fn vkGetPhysicalDeviceQueueFamilyProperties(physicalDevice: VkPhysicalDevice, pQueueFamilyPropertyCount: *mut u32, pQueueFamilyProperties: *mut VkQueueFamilyProperties);
	pub fn vkGetPhysicalDeviceMemoryProperties(physicalDevice: VkPhysicalDevice, pMemoryProperties: *mut VkPhysicalDeviceMemoryProperties);
	pub fn vkCreateDevice(physicalDevice: VkPhysicalDevice, pCreateInfo: *const VkDeviceCreateInfo, pAllocator: *const VkAllocationCallbacks, pDevice: *mut VkDevice) -> VkResult;
	pub fn vkDestroyDevice(device: VkDevice, pAllocator: *const VkAllocationCallbacks);
	pub fn vkCreateRenderPass(device: VkDevice, pCreateInfo: *const VkRenderPassCreateInfo, pAllocator: *const VkAllocationCallbacks, pRenderPass: *mut VkRenderPass) -> VkResult;
	pub fn vkDestroyRenderPass(device: VkDevice, renderPass: VkRenderPass, pAllocator: *const VkAllocationCallbacks);
	pub fn vkCreateFramebuffer(device: VkDevice, pCreateInfo: *const VkFramebufferCreateInfo, pAllocator: *const VkAllocationCallbacks, pFramebuffer: *mut VkFramebuffer) -> VkResult;
	pub fn vkDestroyFramebuffer(device: VkDevice, framebuffer: VkFramebuffer, pAllocator: *const VkAllocationCallbacks);

	// Surface Extension //
	pub fn vkCreateXcbSurfaceKHR(instance: VkInstance, pCreateInfo: *const VkXcbSurfaceCreateInfoKHR, pAllocator: *const VkAllocationCallbacks, pSurface: *mut VkSurfaceKHR) -> VkResult;
	pub fn vkGetPhysicalDeviceXcbPresentationSupportKHR(physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32, connection: *mut xcb_connection_t, visual_id: xcb_visualid_t) -> VkBool32;
	pub fn vkDestroySurfaceKHR(instance: VkInstance, ssurface: VkSurfaceKHR, pAllocator: *const VkAllocationCallbacks);
	pub fn vkGetPhysicalDeviceSurfaceSupportKHR(physicalDevice: VkPhysicalDevice, queueFamilyIndex: u32, surface: VkSurfaceKHR, pSupported: *mut VkBool32) -> VkResult;
	pub fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaceCapabilities: *mut VkSurfaceCapabilitiesKHR) -> VkResult;
	pub fn vkGetPhysicalDeviceSurfaceFormatsKHR(physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pSurfaecFormatCount: *mut u32, pSurfaceFormats: *mut VkSurfaceFormatKHR) -> VkResult;
	pub fn vkGetPhysicalDeviceSurfacePresentModesKHR(physicalDevice: VkPhysicalDevice, surface: VkSurfaceKHR, pPresentModeCount: *mut u32, pPresentModes: *mut VkPresentModeKHR) -> VkResult;
	pub fn vkCreateImageView(device: VkDevice, pCreateInfo: *const VkImageViewCreateInfo, pAllocator: *const VkAllocationCallbacks, pView: *mut VkImageView) -> VkResult;
	pub fn vkDestroyImageView(device: VkDevice, imageView: VkImageView, pAllocator: *const VkAllocationCallbacks);

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

pub type PFN_vkCreateDebugReportCallbackEXT = unsafe extern "system" fn(instance: VkInstance, pCreateInfo: *const VkDebugReportCallbackCreateInfoEXT, pAllocator: *const VkAllocationCallbacks, pCallback: *mut VkDebugReportCallbackEXT) -> VkResult;
pub type PFN_vkDestroyDebugReportCallbackEXT = unsafe extern "system" fn(instance: VkInstance, callback: VkDebugReportCallbackEXT, pAllocator: *const VkAllocationCallbacks);
pub type PFN_vkDebugReportMessageEXT = unsafe extern "system" fn(instance: VkInstance, flags: VkDebugReportFlagsEXT, objectType: VkDebugReportObjectTypeEXT, object: u64, location: size_t, messageCode: i32, pLayerPrefix: *const c_char, pMessage: *const c_char);
pub type PFN_vkDebugReportCallbackEXT = unsafe extern "system" fn(flags: VkDebugReportFlagsEXT, objectType: VkDebugReportObjectTypeEXT, object: u64, location: size_t, messageCode: i32, pLayerPrefix: *const c_char, pMessage: *const c_char, pUserData: *mut c_void) -> VkBool32;
