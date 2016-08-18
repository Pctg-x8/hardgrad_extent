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
	fn execute_rendering(&self, engine: &Engine, g_commands: &GraphicsCommandBuffers, t_commands: Option<&TransferCommandBuffers>, signal_on_complete: &Fence)
		-> Result<u32, EngineError>;
	fn acquire_next_backbuffer_index(&self, wait_semaphore: &QueueFence) -> Result<u32, EngineError>;
	fn present(&self, engine: &Engine, index: u32) -> Result<(), EngineError>;
}
pub struct EntireImage { pub resource: VkImage, pub view: vk::ImageView }
impl ImageResource for EntireImage { fn get_resource(&self) -> VkImage { self.resource } }
pub struct XcbWindow
{
	server: Rc<XServerConnection>, native: XWindowHandle,
	#[allow(dead_code)] device_obj: Rc<vk::Surface>, swapchain: Rc<vk::Swapchain>, rt: Vec<EntireImage>,
	format: VkFormat, extent: VkExtent2D, has_vsync: bool,
	backbuffer_available_signal: QueueFence, transfer_complete_signal: QueueFence
}
impl InternalWindow for XcbWindow
{
	type NativeWindow = XWindowHandle;
	type WindowServer = XServerConnection;

	fn create_unresizable(engine: &Engine, size: VkExtent2D, title: &str) -> Result<Box<Self>, EngineError>
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
				format: format.format, extent: extent, has_vsync: *present_mode == VkPresentModeKHR::FIFO,
				backbuffer_available_signal: try!(engine.create_queue_fence()),
				transfer_complete_signal: try!(engine.create_queue_fence())
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
	fn execute_rendering(&self, engine: &Engine, g_commands: &GraphicsCommandBuffers, t_commands: Option<&TransferCommandBuffers>, signal_on_complete: &Fence)
		-> Result<u32, EngineError>
	{
		self.acquire_next_backbuffer_index(&self.backbuffer_available_signal).and_then(|bb_index|
		{
			if let Some(tcs) = t_commands
			{
				engine.submit_transfer_commands(&tcs, &[], Some(&self.transfer_complete_signal), None)
					.and_then(|()| engine.submit_graphics_commands(&g_commands, &[
						(&self.backbuffer_available_signal, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT),
						(&self.transfer_complete_signal, VK_PIPELINE_STAGE_TRANSFER_BIT)
					], None, Some(signal_on_complete)))
			}
			else
			{
				engine.submit_graphics_commands(&g_commands, &[(&self.backbuffer_available_signal, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)],
					None, Some(signal_on_complete))
			}.map(|()| bb_index)
		})
	}
	fn present(&self, engine: &Engine, index: u32) -> Result<(), EngineError>
	{
		self.swapchain.present(engine.get_device().get_graphics_queue(), index, &[]).map_err(EngineError::from)
	}
	fn acquire_next_backbuffer_index(&self, wait_semaphore: &QueueFence) -> Result<u32, EngineError>
	{
		self.swapchain.acquire_next_image(wait_semaphore.get_internal()).map_err(EngineError::from)
	}
}
