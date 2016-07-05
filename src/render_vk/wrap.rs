// Safety Vulkan Modules

use vkffi::*;
use std;
use std::ffi::*;
use std::os::raw::*;
use libc::size_t;
use xcb;
use xcb::ffi::*;

pub trait CreationObject<StructureT> where Self: std::marker::Sized
{
	fn create(info: &StructureT) -> Result<Self, VkResult>;
}
pub trait InternalProvider<InternalType>
{
	fn get(&self) -> &InternalType;
}

trait ResultValueToObject where Self: std::marker::Sized { fn to_result(self) -> Result<(), Self>; }
impl ResultValueToObject for VkResult
{
	fn to_result(self) -> Result<(), Self> { return if self == VkResult::Success { Ok(()) } else { Err(self) } }
}
pub trait HasParent { type ParentRefType; fn parent(&self) -> Self::ParentRefType; }

pub struct Instance
{
	obj: VkInstance,
	debug_destructor: PFN_vkDestroyDebugReportCallbackEXT,
	debug: VkDebugReportCallbackEXT
}
impl CreationObject<VkInstanceCreateInfo> for Instance
{
	fn create(info: &VkInstanceCreateInfo) -> Result<Self, VkResult>
	{
		let mut i: VkInstance = std::ptr::null_mut();
		let res = unsafe { vkCreateInstance(info, std::ptr::null_mut(), &mut i) };
		if res != VkResult::Success { Err(res) } else
		{
			let cdrc = unsafe { std::mem::transmute::<_, PFN_vkCreateDebugReportCallbackEXT>(vkGetInstanceProcAddr(i, CString::new("vkCreateDebugReportCallbackEXT").unwrap().as_ptr())) };
			let ddrc = unsafe { std::mem::transmute::<_, PFN_vkDestroyDebugReportCallbackEXT>(vkGetInstanceProcAddr(i, CString::new("vkDestroyDebugReportCallbackEXT").unwrap().as_ptr())) };

			let callback_info = VkDebugReportCallbackCreateInfoEXT
			{
				sType: VkStructureType::DebugReportCallbackCreateInfoEXT, pNext: std::ptr::null(),
				flags: VkDebugReportFlagBitsEXT::Error as u32 | VkDebugReportFlagBitsEXT::Warning as u32 |
					VkDebugReportFlagBitsEXT::PerformanceWarning as u32 | VkDebugReportFlagBitsEXT::Information as u32,
				pfnCallback: Self::debug_callback,
				pUserData: std::ptr::null_mut()
			};

			let mut callback: VkDebugReportCallbackEXT = std::ptr::null_mut();
			let res = unsafe { cdrc(i, &callback_info, std::ptr::null(), &mut callback) };
			if res != VkResult::Success { Err(res) } else
			{
				Ok(Instance
				{
					obj: i, debug: callback,
					debug_destructor: ddrc
				})
			}
		}
	}
}
impl Instance
{
	pub fn enumerate_adapters(&self) -> Result<Vec<VkPhysicalDevice>, VkResult>
	{
		let mut adapter_count: u32 = 0;
		let res = unsafe { vkEnumeratePhysicalDevices(self.obj, &mut adapter_count, std::ptr::null_mut()) };
		if res == VkResult::Success
		{
			let mut adapters: Vec<VkPhysicalDevice> = vec![std::ptr::null_mut(); adapter_count as usize];
			let res = unsafe { vkEnumeratePhysicalDevices(self.obj, &mut adapter_count, adapters.as_mut_ptr()) };
			if res == VkResult::Success
			{
				println!("=== Physical Device Enumeration ===");
				println!("-- Found {} adapters", adapter_count);
				for i in 0 .. adapter_count
				{
					let mut props: VkPhysicalDeviceProperties = unsafe { std::mem::uninitialized() };
					let mut memory_props: VkPhysicalDeviceMemoryProperties = unsafe { std::mem::uninitialized() };

					unsafe
					{
						vkGetPhysicalDeviceProperties(adapters[i as usize], &mut props);
						vkGetPhysicalDeviceMemoryProperties(adapters[i as usize], &mut memory_props);
					}

					println!("#{}: ", i);
					println!("  Name: {}", unsafe { std::ffi::CStr::from_ptr(props.deviceName.as_ptr()).to_str().unwrap() });
					println!("  API Version: {}", props.apiVersion);
				}
				Ok(adapters)
			}
			else { Err(res) }
		}
		else { Err(res) }
	}
	unsafe extern "system" fn debug_callback(flags: VkDebugReportFlagsEXT, object_type: VkDebugReportObjectTypeEXT, _: u64,
		_: size_t, _: i32, _: *const c_char, message: *const c_char, _: *mut c_void) -> VkBool32
	{
		println!("Vulkan DebugCall[{:?}/{:?}]: {}", object_type, flags, CStr::from_ptr(message).to_str().unwrap());
		1
	}
}
impl std::ops::Drop for Instance
{
	fn drop(&mut self)
	{
		unsafe { (self.debug_destructor)(self.obj, self.debug, std::ptr::null()) };
		unsafe { vkDestroyInstance(self.obj, std::ptr::null()) };
	}
}
pub struct PhysicalDevice { obj: VkPhysicalDevice }
impl PhysicalDevice
{
	pub fn wrap(pdev: VkPhysicalDevice) -> Self { PhysicalDevice { obj: pdev } }
	pub fn get_graphics_queue_family_index(&self) -> Option<u32>
	{
		let mut property_count: u32 = 0;
		unsafe { vkGetPhysicalDeviceQueueFamilyProperties(self.obj, &mut property_count, std::ptr::null_mut()) };
		let mut properties: Vec<VkQueueFamilyProperties> = unsafe { vec![std::mem::uninitialized(); property_count as usize] };
		unsafe { vkGetPhysicalDeviceQueueFamilyProperties(self.obj, &mut property_count, properties.as_mut_ptr()) };
		properties.into_iter().enumerate().filter(|&(_, ref x)| (x.queueFlags & (VkQueueFlagBits::Graphics as u32)) != 0).map(|(i, _)| i as u32).next()
	}
	pub fn is_xcb_presentation_support(&self, qf: u32, con: *mut xcb_connection_t, vid: xcb_visualid_t) -> bool
	{
		unsafe { vkGetPhysicalDeviceXcbPresentationSupportKHR(self.obj, qf, con, vid) == 1 }
	}
	pub fn is_surface_support<'i>(&self, queue_family_index: u32, surface: &Surface<'i>) -> bool
	{
		let mut supported: VkBool32 = 0;
		unsafe { vkGetPhysicalDeviceSurfaceSupportKHR(self.obj, queue_family_index, surface.obj, &mut supported) };
		supported == 1
	}
	pub fn get_surface_capabilities<'i>(&self, surface: &Surface<'i>) -> VkSurfaceCapabilitiesKHR
	{
		let mut caps: VkSurfaceCapabilitiesKHR = unsafe { std::mem::uninitialized() };
		unsafe { vkGetPhysicalDeviceSurfaceCapabilitiesKHR(self.obj, surface.obj, &mut caps) };
		caps
	}
	pub fn enumerate_surface_formats<'i>(&self, surface: &Surface<'i>) -> Vec<VkSurfaceFormatKHR>
	{
		let mut format_count: u32 = 0;
		unsafe { vkGetPhysicalDeviceSurfaceFormatsKHR(self.obj, surface.obj, &mut format_count, std::ptr::null_mut()) };
		let mut vformats: Vec<VkSurfaceFormatKHR> = vec![unsafe { std::mem::uninitialized() }; format_count as usize];
		unsafe { vkGetPhysicalDeviceSurfaceFormatsKHR(self.obj, surface.obj, &mut format_count, vformats.as_mut_ptr()) };
		println!("== Enumerate Supported Formats ==");
		for f in &vformats { println!("- {:?}", f.format); }
		vformats
	}
	pub fn enumerate_present_modes<'i>(&self, surface: &Surface<'i>) -> Vec<VkPresentModeKHR>
	{
		let mut present_mode_count: u32 = 0;
		unsafe { vkGetPhysicalDeviceSurfacePresentModesKHR(self.obj, surface.obj, &mut present_mode_count, std::ptr::null_mut()) };
		let mut vmodes: Vec<VkPresentModeKHR> = vec![unsafe { std::mem::uninitialized() }; present_mode_count as usize];
		unsafe { vkGetPhysicalDeviceSurfacePresentModesKHR(self.obj, surface.obj, &mut present_mode_count, vmodes.as_mut_ptr()) };
		println!("== Enumerate Supported Present Modes ==");
		for m in &vmodes { println!("- {:?}", m); }
		vmodes
	}

	pub fn create_device(&self, info: &VkDeviceCreateInfo, queue_index: u32) -> Result<Device, VkResult>
	{
		let mut dev: VkDevice = std::ptr::null_mut();
		let res = unsafe { vkCreateDevice(self.obj, info, std::ptr::null(), &mut dev) };
		if res != VkResult::Success { Err(res) } else { Ok(Device { obj: dev, queue_family_index: queue_index }) }
	}
}
pub struct Device { obj: VkDevice, pub queue_family_index: u32 }
impl std::ops::Drop for Device
{
	fn drop(&mut self) { unsafe { vkDestroyDevice(self.obj, std::ptr::null()) }; }
}
impl Device
{
	pub fn create_image_view<'d>(&'d self, info: &VkImageViewCreateInfo) -> Result<ImageView<'d>, VkResult>
	{
		let mut obj: VkImageView = std::ptr::null_mut();
		unsafe { vkCreateImageView(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| ImageView { device_ref: self, obj: obj })
	}
	pub fn create_render_pass<'d>(&'d self, info: &VkRenderPassCreateInfo) -> Result<RenderPass<'d>, VkResult>
	{
		let mut obj: VkRenderPass = std::ptr::null_mut();
		unsafe { vkCreateRenderPass(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| RenderPass { device_ref: self, obj: obj })
	}
	pub fn create_framebuffer<'d>(&'d self, info: &VkFramebufferCreateInfo) -> Result<Framebuffer<'d>, VkResult>
	{
		let mut obj: VkFramebuffer = std::ptr::null_mut();
		unsafe { vkCreateFramebuffer(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| Framebuffer { device_ref: self, obj: obj })
	}
}

macro_rules! SafeObjectDerivedFromDevice
{
	($name: ident for $t: tt destructed by $dfn: ident) =>
	{
		SafeObjectDerivedFromDevice!($name for $t);
		impl <'d> std::ops::Drop for $name<'d> { fn drop(&mut self) { unsafe { $dfn(self.device_ref.obj, self.obj, std::ptr::null()) }; } }
	};
	($name: ident for $t: ident) =>
	{
		pub struct $name<'d> { device_ref: &'d Device, obj: $t }
		impl <'d> HasParent for $name<'d> { type ParentRefType = &'d Device; fn parent(&self) -> &'d Device { self.device_ref } }
		impl <'d> InternalProvider<$t> for $name<'d> { fn get(&self) -> &$t { &self.obj } }
	};
}

pub trait VkImageResource
{
	fn get(&self) -> VkImage;
	fn create_view(&self, info: &VkImageViewCreateInfo) -> Result<ImageView, VkResult>;
}
SafeObjectDerivedFromDevice!(ImageRef for VkImage);
impl <'d> VkImageResource for ImageRef<'d>
{
	fn get(&self) -> VkImage { self.obj }
	fn create_view(&self, info: &VkImageViewCreateInfo) -> Result<ImageView, VkResult>
	{
		let mut obj: VkImageView = std::ptr::null_mut();
		unsafe { vkCreateImageView(self.device_ref.obj, info, std::ptr::null_mut(), &mut obj) }.to_result()
			.map(|()| ImageView { device_ref: self.device_ref, obj: obj })
	}
}
SafeObjectDerivedFromDevice!(ImageView for VkImageView destructed by vkDestroyImageView);
SafeObjectDerivedFromDevice!(RenderPass for VkRenderPass destructed by vkDestroyRenderPass);
SafeObjectDerivedFromDevice!(Framebuffer for VkFramebuffer destructed by vkDestroyFramebuffer);

pub struct Surface<'a>
{
	instance_ref: &'a Instance,
	obj: VkSurfaceKHR
}
impl <'a> Surface<'a>
{
	pub fn create(instance: &'a Instance, info: &VkXcbSurfaceCreateInfoKHR) -> Result<Self, VkResult>
	{
		let mut obj: VkSurfaceKHR = std::ptr::null_mut();
		let res = unsafe { vkCreateXcbSurfaceKHR(instance.obj, info, std::ptr::null(), &mut obj) };
		if res != VkResult::Success { Err(res) } else { Ok(Surface { instance_ref: instance, obj: obj }) }
	}
}
impl <'a> std::ops::Drop for Surface<'a>
{
	fn drop(&mut self) { unsafe { vkDestroySurfaceKHR(self.instance_ref.obj, self.obj, std::ptr::null()) }; }
}
impl <'a> InternalProvider<VkSurfaceKHR> for Surface<'a>
{
	fn get(&self) -> &VkSurfaceKHR { &self.obj }
}

SafeObjectDerivedFromDevice!(Swapchain for VkSwapchainKHR destructed by vkDestroySwapchainKHR);
impl <'a> Swapchain<'a>
{
	pub fn create(device_ref: &'a Device, info: &VkSwapchainCreateInfoKHR) -> Result<Self, VkResult>
	{
		let mut p: VkSwapchainKHR = std::ptr::null_mut();
		let res = unsafe { vkCreateSwapchainKHR(device_ref.obj, info, std::ptr::null(), &mut p) };
		if res == VkResult::Success { Ok(Swapchain { device_ref: device_ref, obj: p }) } else { Err(res) }
	}
	pub fn get_images<'i>(&self) -> Result<Vec<ImageRef>, VkResult>
	{
		let mut image_count: u32 = 0;
		unsafe { vkGetSwapchainImagesKHR(self.device_ref.obj, self.obj, &mut image_count, std::ptr::null_mut()) }.to_result().and_then(|()|
		{
			let mut v: Vec<VkImage> = vec![std::ptr::null_mut(); image_count as usize];
			unsafe { vkGetSwapchainImagesKHR(self.device_ref.obj, self.obj, &mut image_count, v.as_mut_ptr()) }.to_result()
				.map(|()| v.into_iter().map(|x| ImageRef { device_ref: self.device_ref, obj: x }).collect::<Vec<_>>())
		})
	}
}
