#![allow(non_snake_case)]

// Vulkan C to Rust FFI (Dispatchable/Non-Dispatchable) Objects

// Defines Dispatchable Handles(by Opaque Structs representing in Rust)
macro_rules! DefHandle
{
	($name: ident behinds in $bname: ident) =>
	{
		mod $bname { pub enum _T {} }
		pub type $name = *mut $bname::_T;
	}
}
#[cfg(target_pointer_width = "64")]
macro_rules! DefNonDispatchableHandle
{
	($name: ident behinds in $bname: ident) =>
	{
		mod $bname { pub enum _T {} }
		pub type $name = *mut $bname::_T;
	}
}
#[cfg(target_pointer_width = "32")]
macro_rules! DefNonDispatchableHandle
{
	($name: ident behinds in $bname: ident) =>
	{
		pub type $name = u64;
	}
}

DefHandle!(VkInstance behinds in __VkInstance);
DefHandle!(VkPhysicalDevice behinds in __VkPhysicalDevice);
DefHandle!(VkDevice behinds in __VkDevice);
DefHandle!(VkQueue behinds in __VkQueue);

DefNonDispatchableHandle!(VkSurfaceKHR behinds in __VkSurfaceKHR);
