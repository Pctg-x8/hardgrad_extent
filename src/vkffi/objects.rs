#![allow(non_snake_case)]

// Vulkan C to Rust FFI (Dispatchable/Non-Dispatchable) Objects

// Defines Dispatchable Handles(by Opaque Structs representing in Rust)
macro_rules! DefHandle
{
	($name: ident: $bname: ident) =>
	{
		mod $bname { pub enum _T {} }
		pub type $name = *mut $bname::_T;
	}
}
#[cfg(target_pointer_width = "64")]
macro_rules! DefNonDispatchableHandle
{
	($name: ident: $bname: ident) =>
	{
		mod $bname { pub enum _T {} }
		pub type $name = *mut $bname::_T;
	}
}
#[cfg(target_pointer_width = "32")]
macro_rules! DefNonDispatchableHandle
{
	($name: ident: $bname: ident) =>
	{
		pub type $name = u64;
	}
}

DefHandle!(VkInstance: __VkInstance);
DefHandle!(VkPhysicalDevice: __VkPhysicalDevice);
DefHandle!(VkDevice: __VkDevice);
DefHandle!(VkQueue: __VkQueue);
DefNonDispatchableHandle!(VkSemaphore: __VkSemaphore);
DefHandle!(VkCommandBuffer: __VkCommandBuffer);
DefNonDispatchableHandle!(VkFence: __VkFence);
DefNonDispatchableHandle!(VkDeviceMemory: __VkDeviceMemory);
DefNonDispatchableHandle!(VkBuffer: __VkBuffer);
DefNonDispatchableHandle!(VkImage: __VkImage);

DefNonDispatchableHandle!(VkSurfaceKHR: __VkSurfaceKHR);
DefNonDispatchableHandle!(VkSwapchainKHR: VkSwapchainKHR__);

DefNonDispatchableHandle!(VkDebugReportCallbackEXT: __VkDebugReportCallbackEXT);
