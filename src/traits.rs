// Traits

pub mod ex
{
	#![cfg(feature = "use_vk")]

	pub use vulkan::*;
	pub use vkffi::*;

	pub trait VkSurfaceProvider
	{
		fn create_surface_vk<'a>(&self, instance: &'a vk::Instance) -> Result<vk::Surface<'a>, VkResult>;
	}
}
pub use self::ex::*;
