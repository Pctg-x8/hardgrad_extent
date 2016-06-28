// Traits

#[cfg(feature = "use_vk")]
use vulkan::*;
#[cfg(feature = "use_vk")]
use vkffi::*;

#[cfg(feature = "use_vk")]
pub trait VkSurfaceProvider
{
	fn create_surface_vk(&self) -> Result<vk::Surface, VkResult>;
}
