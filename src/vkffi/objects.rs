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

DefHandle!(VkInstance behinds in __VkInstance);
DefHandle!(VkPhysicalDevice behinds in __VkPhysicalDevice);
DefHandle!(VkDevice behinds in __VkDevice);
DefHandle!(VkQueue behinds in __VkQueue);
