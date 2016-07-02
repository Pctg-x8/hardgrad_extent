extern crate libc;
extern crate x11;
#[macro_use] mod vkffi;
mod render_vk;
mod xlib;

use vkffi::*;
use render_vk::wrap as vk;
use render_vk::wrap::{CreationObject, InternalProvider, HasParent};
use xlib::XlibWindow;

const APP_NAME: &'static str = "HardGrad -> Extent\0";
const ENGINE_NAME: &'static str = "Hybrid-ML\0";
const DEBUG_LAYER_NAME: &'static str = "VK_LAYER_LUNARG_standard_validation\0";
const SURFACE_EXTENSION_NAME: &'static str = "VK_KHR_surface\0";
const PSURFACE_EXTENSION_NAME: &'static str = "VK_KHR_xlib_surface\0";
const DEBUG_EXTENSION_NAME: &'static str = "VK_EXT_debug_report\0";
const SWAPCHAIN_EXTENSION_NAME: &'static str = "VK_KHR_swapchain\0";

// Application Dependent Factories
fn create_instance() -> vk::Instance
{
	let layers = [DEBUG_LAYER_NAME.as_ptr()];
	let extensions = [SURFACE_EXTENSION_NAME.as_ptr(), PSURFACE_EXTENSION_NAME.as_ptr(), DEBUG_EXTENSION_NAME.as_ptr()];
	let app_info = VkApplicationInfo
	{
		sType: VkStructureType::ApplicationInfo, pNext: std::ptr::null(),
		pApplicationName: APP_NAME.as_ptr() as *const i8,
		applicationVersion: VK_MAKE_VERSION!(0, 0, 1),
		pEngineName: ENGINE_NAME.as_ptr() as *const i8,
		engineVersion: VK_MAKE_VERSION!(0, 0, 1),
		apiVersion: VK_API_VERSION_1_0
	};
	let instance_info = VkInstanceCreateInfo
	{
		sType: VkStructureType::InstanceCreateInfo, pNext: std::ptr::null(), flags: 0,
		pApplicationInfo: &app_info,
		enabledLayerCount: layers.len() as u32, ppEnabledLayerNames: layers.as_ptr() as *const *const i8,
		enabledExtensionCount: extensions.len() as u32, ppEnabledExtensionNames: extensions.as_ptr() as *const *const i8
	};

	vk::Instance::create(&instance_info).expect("Unable to create instance")
}
fn create_graphics_device(adapter_ref: &vk::PhysicalDevice) -> vk::Device
{
	let gqf_index = adapter_ref.get_graphics_queue_family_index().expect("unable to find graphics queue on device");
	println!("-- Queue Index: {}", gqf_index);
	let q_priorities = [0.0f32];
	let dev_layers = [DEBUG_LAYER_NAME.as_ptr()];
	let dev_extensions = [SWAPCHAIN_EXTENSION_NAME.as_ptr()];
	let queue_info = VkDeviceQueueCreateInfo
	{
		sType: VkStructureType::DeviceQueueCreateInfo, pNext: std::ptr::null(), flags: 0,
		queueCount: 1, queueFamilyIndex: gqf_index, pQueuePriorities: q_priorities.as_ptr()
	};
	let device_info = VkDeviceCreateInfo
	{
		sType: VkStructureType::DeviceCreateInfo, pNext: std::ptr::null(), flags: 0,
		queueCreateInfoCount: 1, pQueueCreateInfos: &queue_info,
		enabledLayerCount: dev_layers.len() as u32, ppEnabledLayerNames: dev_layers.as_ptr() as *const *const i8,
		enabledExtensionCount: dev_extensions.len() as u32, ppEnabledExtensionNames: dev_extensions.as_ptr() as *const *const i8,
		pEnabledFeatures: std::ptr::null()
	};

	adapter_ref.create_device(&device_info, gqf_index).unwrap()
}
fn create_surface<'i, 'dpy>(instance_ref: &'i vk::Instance, window: &xlib::WindowWithColormap<'dpy>) -> vk::Surface<'i>
{
	let x11_surface_info = VkXlibSurfaceCreateInfoKHR
	{
		sType: VkStructureType::XlibSurfaceCreateInfoKHR, pNext: std::ptr::null(),
		dpy: window.display_ref.internal, window: window.internal, flags: 0
	};
	vk::Surface::create(instance_ref, &x11_surface_info).unwrap()
}
fn create_swapchain<'d>(adapter: &vk::PhysicalDevice, device_ref: &'d vk::Device, surface: &vk::Surface) -> (vk::Swapchain<'d>, VkFormat, VkExtent2D)
{
	// capabilities check //
	if !adapter.is_surface_support(device_ref.queue_family_index, surface) { panic!("Unsupported Surface"); }
	let surface_caps = adapter.get_surface_capabilities(surface);

	// making desired parameters //
	let format = adapter.enumerate_surface_formats(surface).into_iter().filter(|ref x| x.format == VkFormat::B8G8R8A8_UNORM || x.format == VkFormat::R8G8B8A8_UNORM)
		.next().expect("Desired format is not found");
	let present_mode = adapter.enumerate_present_modes(surface).into_iter().filter(|ref x| **x == VkPresentModeKHR::Mailbox || **x == VkPresentModeKHR::FIFO)
		.next().expect("Desired Present Mode is not found");
	let sc_extent = match surface_caps.currentExtent
	{
		VkExtent2D(w, h) if w == std::u32::MAX || h == std::u32::MAX => { VkExtent2D(640, 480) },
		e => e
	};

	// set information and create //
	let queue_family_indices = [device_ref.queue_family_index];
	let swapchain_info = VkSwapchainCreateInfoKHR
	{
		sType: VkStructureType::SwapchainCreateInfoKHR, pNext: std::ptr::null(),
		minImageCount: surface_caps.minImageCount + 1, imageFormat: format.format, imageColorSpace: format.colorSpace,
		imageExtent: sc_extent, imageArrayLayers: 1, imageUsage: VkImageUsageFlagBits::ColorAttachment as u32,
		imageSharingMode: VkSharingMode::Exclusive, compositeAlpha: VkCompositeAlphaFlagBitsKHR::Opaque,
		preTransform: VkSurfaceTransformFlagBitsKHR::Identity,
		presentMode: present_mode, clipped: 1,
		pQueueFamilyIndices: queue_family_indices.as_ptr(), queueFamilyIndexCount: queue_family_indices.len() as u32,
		oldSwapchain: std::ptr::null_mut(), flags: 0, surface: *surface.get()
	};

	(vk::Swapchain::create(device_ref, &swapchain_info).unwrap(), format.format, sc_extent)
}
fn create_image_views<'d, ImageObj: vk::VkImageResource + vk::HasParent<ParentRefType=&'d vk::Device>>(images: &'d Vec<ImageObj>, format: VkFormat) -> Vec<vk::ImageView>
{
	images.into_iter().map(|o|
	{
		let view_info = VkImageViewCreateInfo
		{
			sType: VkStructureType::ImageViewCreateInfo, pNext: std::ptr::null(),
			image: o.get(), viewType: VkImageViewType::Dim2, format: format,
			components: VkComponentMapping { r: VkComponentSwizzle::R, g: VkComponentSwizzle::G, b: VkComponentSwizzle::B, a: VkComponentSwizzle::A },
			subresourceRange: VkImageSubresourceRange
			{
				aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
				baseArrayLayer: 0, layerCount: 1,
				baseMipLevel: 0, levelCount: 1
			},
			flags: 0
		};
		o.parent().create_image_view(&view_info).unwrap()
	}).collect::<Vec<_>>()
}
fn create_render_pass<'d>(dev: &'d vk::Device, attachments: &[VkAttachmentDescription], subpasses: &[VkSubpassDescription], dependencies: &[VkSubpassDependency])
	-> Result<vk::RenderPass<'d>, VkResult>
{
	dev.create_render_pass(&VkRenderPassCreateInfo
	{
		sType: VkStructureType::RenderPassCreateInfo, pNext: std::ptr::null(), flags: 0,
		attachmentCount: attachments.len() as u32, pAttachments: attachments.as_ptr(),
		subpassCount: subpasses.len() as u32, pSubpasses: subpasses.as_ptr(),
		dependencyCount: dependencies.len() as u32, pDependencies: dependencies.as_ptr()
	})
}
fn create_simple_render_pass<'d>(dev: &'d vk::Device, format: VkFormat) -> vk::RenderPass<'d>
{
	let color_attref = VkAttachmentReference { attachment: 0, layout: VkImageLayout::ColorAttachmentOptimal };
	let subpasses = [
		VkSubpassDescription
		{
			inputAttachmentCount: 0, pInputAttachments: std::ptr::null(),
			colorAttachmentCount: 1, pColorAttachments: &color_attref,
			pDepthStencilAttachment: std::ptr::null(), pResolveAttachments: std::ptr::null(),
			preserveAttachmentCount: 0, pPreserveAttachments: std::ptr::null(),
			pipelineBindPoint: VkPipelineBindPoint::Graphics, flags: 0
		}
	];
	let attachment_descs = [
		VkAttachmentDescription
		{
			format: format, samples: VK_SAMPLE_COUNT_1_BIT, flags: 0,
			loadOp: VkAttachmentLoadOp::Clear, storeOp: VkAttachmentStoreOp::Store,
			stencilLoadOp: VkAttachmentLoadOp::DontCare, stencilStoreOp: VkAttachmentStoreOp::DontCare,
			initialLayout: VkImageLayout::ColorAttachmentOptimal, finalLayout: VkImageLayout::PresentSrcKHR
		}
	];
	create_render_pass(dev, &attachment_descs, &subpasses, &[]).unwrap()
}
fn create_framebuffers<'d>(views: &Vec<vk::ImageView<'d>>, rp: &vk::RenderPass<'d>, extent: VkExtent2D) -> Vec<vk::Framebuffer<'d>>
{
	let VkExtent2D(width, height) = extent;

	views.into_iter().map(|v|
	{
		let fb_info = VkFramebufferCreateInfo
		{
			sType: VkStructureType::FramebufferCreateInfo, pNext: std::ptr::null(),
			attachmentCount: 1, pAttachments: v.get(), renderPass: *rp.get(),
			width: width, height: height, layers: 1, flags: 0
		};
		v.parent().create_framebuffer(&fb_info).unwrap()
	}).collect::<Vec<_>>()
}

pub fn start_app() -> i32
{
	// init x11
	let display = xlib::Display::open(None).unwrap();
	let root = display.get_default_root_window();
	let wa = root.get_window_attributes();
	let vid = unsafe { x11::xlib::XVisualIDFromVisual(wa.visual) };
	println!("visual_id: {}", vid);

	// init vulkan
	let instance = create_instance();
	let adapter = vk::PhysicalDevice::wrap(instance.enumerate_adapters().unwrap()[0]);
	let qf = adapter.get_graphics_queue_family_index().unwrap();
	if !adapter.is_xlib_presentation_support(qf, display.internal, vid) { panic!("Unsupported Display Format"); }
	let device = create_graphics_device(&adapter);

	// create window
	let window = display.create_window(&root, &wa).unwrap();
	let atom_delete_window_msg = display.intern_atom("WM_DELETE_WINDOW", false);
	window.set_wm_protocols(&mut [atom_delete_window_msg]);
	window.map();
	window.set_title_raw(APP_NAME);

	// Ready for Rendering
	let surface = create_surface(&instance, &window);
	let (swapchain, sc_format, sc_extent) = create_swapchain(&adapter, &device, &surface);
	let final_images = swapchain.get_images().unwrap();
	let final_image_views = create_image_views(&final_images, sc_format);
	let simple_pass = create_simple_render_pass(&device, sc_format);
	let final_framebuffers = create_framebuffers(&final_image_views, &simple_pass, sc_extent);

	// Application Loop
	'app_loop: loop
	{
		while unsafe { x11::xlib::XPending(display.internal) } > 0
		{
			let mut event: x11::xlib::XEvent = unsafe { std::mem::uninitialized() };
			unsafe { x11::xlib::XNextEvent(display.internal, &mut event) };

			match event.get_type()
			{
				x11::xlib::ClientMessage =>
				{
					let xc: x11::xlib::XClientMessageEvent = unsafe { *std::mem::transmute::<*const x11::xlib::XEvent, *const _>(&event) };
					let first_long: u32 = unsafe { *std::mem::transmute::<*const _, *const _>(&xc.data) };
					if first_long as x11::xlib::Atom == atom_delete_window_msg { break 'app_loop; }
				},
				_ => ()
			}
		}
		// println!("Render");
	}

	0
}
fn main() { std::process::exit(start_app()); }
