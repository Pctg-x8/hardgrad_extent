#![allow(non_camel_case_types)]

// Vulkan C to Rust FFI functions

use vkffi::enums::*;
use vkffi::structs::*;
use vkffi::objects::*;
use std::os::raw::*;
use libc::size_t;

#[link(name = "vulkan-1")]
extern "system"
{
	pub fn vkCreateInstance(pCreateInfo: *const VkInstanceCreateInfo, pAllocator: *const VkAllocationCallbacks, pInstance: *mut VkInstance) -> VkResult;
	pub fn vkDestroyInstance(instance: VkInstance, pAllocator: *const VkAllocationCallbacks);
}

// Function Pointers
pub type PFN_vkAllocationFunction = unsafe extern "system" fn(pUserData: *mut c_void, size: size_t, alignment: size_t, allocationScope: VkSystemAllocationScope) -> *mut c_void;
pub type PFN_vkReallocationFunction = unsafe extern "system" fn(pUserData: *mut c_void, pOriginal: *mut c_void, size: size_t, alignment: size_t, allocationScope: VkSystemAllocationScope) -> *mut c_void;
pub type PFN_vkFreeFunction = unsafe extern "system" fn(pUserData: *mut c_void, pMemory: *mut c_void);
pub type PFN_vkInternalAllocationNotification = unsafe extern "system" fn(pUserData: *mut c_void, size: size_t, allocationType: VkInternalAllocationType, allocationScope: VkSystemAllocationScope);
pub type PFN_vkInternalFreeNotification = unsafe extern "system" fn(pUserData: *mut c_void, size: size_t, allocationType: VkInternalAllocationType, allocationScope: VkSystemAllocationScope);
