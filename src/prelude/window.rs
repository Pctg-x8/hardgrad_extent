// Prelude: Window and RenderWindow

use prelude::internals::*;
use std;
use std::rc::Rc;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use xcbw::*;

pub trait InternalWindow where Self: std::marker::Sized
{
	type NativeWindow;
	type WindowServer: WindowProvider<Self::NativeWindow>;

	fn create_unresizable(engine: &Engine, size: VkExtent2D, title: &str) -> Result<Box<Self>, EngineError>;
}
pub trait Window
{
	fn show(&self);
}
pub trait RenderWindow : Window
{
	fn get_back_images(&self) -> Vec<&EntireImage>;
	fn get_format(&self) -> VkFormat;
	fn get_extent(&self) -> VkExtent2D;
}
pub struct EntireImage { pub resource: VkImage, pub view: vk::ImageView }
impl ImageResource for EntireImage { fn get_resource(&self) -> VkImage { self.resource } }
pub struct XcbWindow
{
	server: Rc<XServerConnection>, native: XWindowHandle,
	#[allow(dead_code)] device_obj: Rc<vk::Surface>, swapchain: Rc<vk::Swapchain>, rt: Vec<EntireImage>,
	format: VkFormat, extent: VkExtent2D, has_vsync: bool
}
impl InternalWindow for XcbWindow
{
	type NativeWindow = XWindowHandle;
	type WindowServer = XServerConnection;

	fn create_unresizable(engine: &Engine, size: VkExtent2D, title: &str)
		-> Result<Box<Self>, EngineError>
	{
		let server = engine.get_window_server();
		let native = server.create_unresizable_window(size, title);
		server.show_window(native);
		server.flush();

		let surface_info = VkXcbSurfaceCreateInfoKHR
		{
			sType: VkStructureType::XcbSurfaceCreateInfoKHR, pNext: std::ptr::null(), flags: 0,
			connection: server.get_raw(), window: native
		};
		let surface = Rc::new(try!(vk::Surface::new_xcb(engine.get_instance(), &surface_info)));

		let adapter = engine.get_device().get_adapter();

		// capabilities check //
		if !engine.get_device().is_surface_support(&surface) { Err(EngineError::GenericError("Unsupported Surface")) }
		else
		{
			let surface_caps = adapter.get_surface_caps(&surface);

			// Making desired parameters //
			let format = try!(adapter.enumerate_surface_formats(&surface).into_iter()
				.find(|x| x.format == VkFormat::R8G8B8A8_SRGB || x.format == VkFormat::B8G8R8A8_SRGB)
				.ok_or(EngineError::GenericError("Desired Format(32bpp SRGB) is not supported")));
			let present_modes = adapter.enumerate_present_modes(&surface);
			let present_mode = try!(present_modes.iter().find(|&&x| x == VkPresentModeKHR::FIFO)
				.or_else(|| present_modes.iter().find(|&&x| x == VkPresentModeKHR::Mailbox))
				.ok_or(EngineError::GenericError("Desired Present Mode is not found")));
			let extent = match surface_caps.currentExtent
			{
				VkExtent2D(std::u32::MAX, _) | VkExtent2D(_, std::u32::MAX) => VkExtent2D(640, 480),
				ce => ce
			};

			// set information and create //
			let queue_family_indices = [engine.get_device().get_graphics_queue().family_index];
			let scinfo = VkSwapchainCreateInfoKHR
			{
				sType: VkStructureType::SwapchainCreateInfoKHR, pNext: std::ptr::null(),
				minImageCount: std::cmp::max(surface_caps.minImageCount, 2), imageFormat: format.format, imageColorSpace: format.colorSpace,
				imageExtent: extent, imageArrayLayers: 1, imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
				imageSharingMode: VkSharingMode::Exclusive, compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT,
				preTransform: VK_SURFACE_TRANSFORM_IDENTITY_BIT, presentMode: *present_mode, clipped: true as VkBool32,
				pQueueFamilyIndices: queue_family_indices.as_ptr(), queueFamilyIndexCount: queue_family_indices.len() as u32,
				oldSwapchain: std::ptr::null_mut(), flags: 0, surface: surface.get()
			};
			let sc = try!(vk::Swapchain::new(engine.get_device().get_internal(), &surface, &scinfo).map(|x| Rc::new(x)));
			let rt_images = try!(sc.get_images());
			let rt = try!(rt_images.iter().map(|&res|
			{
				vk::ImageView::new(engine.get_device().get_internal(), &VkImageViewCreateInfo
				{
					sType: VkStructureType::ImageViewCreateInfo, pNext: std::ptr::null(), flags: 0,
					image: res, subresourceRange: vk::ImageSubresourceRange::default_color(),
					format: format.format, viewType: VkImageViewType::Dim2,
					components: VkComponentMapping::default()
				}).map(|v| EntireImage { resource: res, view: v })
			}).collect::<Result<Vec<_>, _>>());
			info!(target: "Prelude", "Swapchain Backbuffer Count: {}", rt.len());
			Ok(Box::new(XcbWindow
			{
				server: engine.get_window_server().clone(),
				native: native, device_obj: surface, swapchain: sc, rt: rt,
				format: format.format, extent: extent, has_vsync: *present_mode == VkPresentModeKHR::FIFO
			}))
		}
	}
}
impl Window for XcbWindow
{
	fn show(&self)
	{
		self.server.show_window(self.native);
		self.server.flush();
	}
}
impl RenderWindow for XcbWindow
{
	fn get_back_images(&self) -> Vec<&EntireImage> { self.rt.iter().collect() }
	fn get_format(&self) -> VkFormat { self.format }
	fn get_extent(&self) -> VkExtent2D { self.extent }
}
