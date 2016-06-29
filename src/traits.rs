// Traits

#[cfg(feature = "use_vk")]
pub mod ex
{
	use vulkan::*;
	use vkffi::*;

	pub trait VkSurfaceProvider
	{
		fn create_surface_vk<'a>(&self, instance: &'a vk::Instance) -> Result<vk::Surface<'a>, VkResult>;
	}
}
#[cfg(feature = "use_d3d12")]
pub mod ex
{
	use winapi::*;
	use d3d12_sw::*;
	use render::SwapchainFactory;

	pub trait DXGISwapchainProvider
	{
		fn create_swapchain(&self, backend: &SwapchainFactory<HWND, DXGISwapchain>) -> Result<DXGISwapchain, String>;
	}
}
pub use self::ex::*;
