// Render backend

use vkffi::*;
use vulkan as vk;
use vulkan::*;
use ::std;
use std::ffi::CString;
use render::SwapchainFactory;

const VK_KHR_SURFACE_EXTENSION_NAME: &'static str = "VK_KHR_surface\0";
#[cfg(feature = "use_win32")]
const VK_KHR_PLATFORM_SURFACE_EXTENSION_NAME: &'static str = "VK_KHR_win32_surface\0";
#[cfg(feature = "use_x11")]
const VK_KHR_PLATFORM_SURFACE_EXTENSION_NAME: &'static str = "VK_KHR_xlib_surface\0";
const VK_EXT_DEBUG_REPORT_EXTENSION_NAME: &'static str = "VK_EXT_debug_report\0";
const VK_LAYER_LUNARG_STANDARD_VALIDATION_NAME: &'static str = "VK_LAYER_LUNARG_standard_validation\0";

pub struct RenderBackend
{
	instance: vk::Instance
}
impl RenderBackend
{
	pub fn init() -> Self
	{
		println!("-- Initializing RenderBackend with Vulkan");

		let app_name = CString::new("hardgrad_extent").unwrap();
		let engine_name = CString::new("hybrid_ml").unwrap();
		let instance_extensions = [
			VK_KHR_SURFACE_EXTENSION_NAME.as_ptr(), VK_KHR_PLATFORM_SURFACE_EXTENSION_NAME.as_ptr(),
			VK_EXT_DEBUG_REPORT_EXTENSION_NAME.as_ptr()
		];
		let instance_layers = [VK_LAYER_LUNARG_STANDARD_VALIDATION_NAME.as_ptr()];
		let app_info = VkApplicationInfo
		{
			sType: VkStructureType::ApplicationInfo,
			pApplicationName: app_name.as_ptr(),
			applicationVersion: VK_MAKE_VERSION!(0, 0, 1),
			pEngineName: engine_name.as_ptr(),
			engineVersion: VK_MAKE_VERSION!(0, 0, 1),
			apiVersion: VK_API_VERSION_1_0,
			pNext: std::ptr::null()
		};
		let instance_info = VkInstanceCreateInfo
		{
			sType: VkStructureType::InstanceCreateInfo,
			pApplicationInfo: &app_info,
			enabledExtensionCount: instance_extensions.len() as u32,
			ppEnabledExtensionNames: instance_extensions.as_ptr() as *const *const i8,
			enabledLayerCount: instance_layers.len() as u32,
			ppEnabledLayerNames: instance_layers.as_ptr() as *const *const i8,
			pNext: std::ptr::null(), flags: 0
		};
		let inst = vk::Instance::create(&instance_info).expect("Creating Instance");

		RenderBackend
		{
			instance: inst
		}
	}
}
impl <'a> SwapchainFactory<vk::Surface<'a>, vk::SwapchainKHR<'a>> for RenderBackend
{
	fn create_swapchain(&self, target: &vk::Surface) -> Result<vk::SwapchainKHR, String>
	{
		unreachable!();
	}
}
