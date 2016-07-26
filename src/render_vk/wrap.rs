// Safety Vulkan Modules

use std::io::prelude::*;
use vkffi::*;
use std;
use std::ffi::*;
use std::os::raw::*;
use libc::size_t;
use traits::*;
use render_vk::traits::*;

impl ResultValueToObject for VkResult
{
	fn to_result(self) -> Result<(), Self> { return if self == VkResult::Success { Ok(()) } else { Err(self) } }
}

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
					VkDebugReportFlagBitsEXT::PerformanceWarning as u32/* | VkDebugReportFlagBitsEXT::Information as u32*/,
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
pub struct PhysicalDevice { obj: VkPhysicalDevice, memory_properties: VkPhysicalDeviceMemoryProperties }
impl PhysicalDevice
{
	pub fn wrap(pdev: VkPhysicalDevice) -> Self
	{
		let mut mem_props: VkPhysicalDeviceMemoryProperties = unsafe { std::mem::uninitialized() };
		unsafe { vkGetPhysicalDeviceMemoryProperties(pdev, &mut mem_props) };
		PhysicalDevice { obj: pdev, memory_properties: mem_props }
	}
	pub fn get_queue_family_indices(&self) -> Vec<VkQueueFamilyProperties>
	{
		let mut property_count: u32 = 0;
		unsafe { vkGetPhysicalDeviceQueueFamilyProperties(self.obj, &mut property_count, std::ptr::null_mut()) };
		let mut properties: Vec<VkQueueFamilyProperties> = unsafe { vec![std::mem::uninitialized(); property_count as usize] };
		unsafe { vkGetPhysicalDeviceQueueFamilyProperties(self.obj, &mut property_count, properties.as_mut_ptr()) };
		properties
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
	pub fn get_properties(&self) -> VkPhysicalDeviceProperties
	{
		let mut props: VkPhysicalDeviceProperties = unsafe { std::mem::uninitialized() };
		unsafe { vkGetPhysicalDeviceProperties(self.obj, &mut props) };
		props
	}
	pub fn get_features(&self) -> VkPhysicalDeviceFeatures
	{
		let mut features: VkPhysicalDeviceFeatures = unsafe { std::mem::uninitialized() };
		unsafe { vkGetPhysicalDeviceFeatures(self.obj, &mut features) };
		features
	}
	pub fn enumerate_queue_family_properties(&self) -> Vec<VkQueueFamilyProperties>
	{
		let mut count: u32 = 0;
		unsafe { vkGetPhysicalDeviceQueueFamilyProperties(self.obj, &mut count, std::ptr::null_mut()) };
		let mut props: Vec<VkQueueFamilyProperties> = vec![unsafe { std::mem::uninitialized() }; count as usize];
		unsafe { vkGetPhysicalDeviceQueueFamilyProperties(self.obj, &mut count, props.as_mut_ptr()) };
		props
	}

	pub fn create_device(&self, info: &VkDeviceCreateInfo) -> Result<Device, VkResult>
	{
		let mut dev: VkDevice = std::ptr::null_mut();
		unsafe { vkCreateDevice(self.obj, info, std::ptr::null(), &mut dev) }.to_result().map(|()| Device { adapter_ref: self, obj: dev })
	}
	pub fn get_memory_type_index(&self, desired_property_flags: VkMemoryPropertyFlags) -> Option<usize>
	{
		self.memory_properties.memoryTypes[0 .. self.memory_properties.memoryTypeCount as usize]
			.iter().enumerate().filter(|&(_, &VkMemoryType(property_flags, _))| (property_flags & desired_property_flags) != 0)
			.map(|(i, _)| i).next()
	}
}
impl InternalProvider<VkPhysicalDevice> for PhysicalDevice
{
	fn get(&self) -> VkPhysicalDevice { self.obj }
}
pub struct Device<'a> { adapter_ref: &'a PhysicalDevice, obj: VkDevice }
impl <'a> std::ops::Drop for Device<'a>
{
	fn drop(&mut self) { unsafe { vkDestroyDevice(self.obj, std::ptr::null()) }; }
}
impl <'a> HasParent for Device<'a>
{
	type ParentRefType = &'a PhysicalDevice;
	fn parent(&self) -> Self::ParentRefType { self.adapter_ref }
}
impl <'a> Device<'a>
{
	pub fn get_queue(&self, family_index: u32, index: u32) -> Queue
	{
		let mut q: VkQueue = std::ptr::null_mut();
		unsafe { vkGetDeviceQueue(self.obj, family_index, index, &mut q) };
		Queue { device_ref: self, obj: q, family_index: family_index }
	}
	pub fn create_image_view(&self, info: &VkImageViewCreateInfo) -> Result<ImageView, VkResult>
	{
		let mut obj: VkImageView = std::ptr::null_mut();
		unsafe { vkCreateImageView(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| ImageView { device_ref: self, obj: obj })
	}
	pub fn create_render_pass(&self, info: &VkRenderPassCreateInfo) -> Result<RenderPass, VkResult>
	{
		let mut obj: VkRenderPass = std::ptr::null_mut();
		unsafe { vkCreateRenderPass(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| RenderPass { device_ref: self, obj: obj })
	}
	pub fn create_framebuffer(&self, info: &VkFramebufferCreateInfo) -> Result<Framebuffer, VkResult>
	{
		let mut obj: VkFramebuffer = std::ptr::null_mut();
		unsafe { vkCreateFramebuffer(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| Framebuffer { device_ref: self, obj: obj })
	}
	/// Creates command pool
	pub fn create_command_pool(&self, queue: &Queue, allow_resetting_per_buffer: bool, transient: bool) -> Result<CommandPool, VkResult>
	{
		let flags = if allow_resetting_per_buffer { VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT } else { 0 }
			| if transient { VK_COMMAND_POOL_CREATE_TRANSIENT_BIT } else { 0 };
		let info = VkCommandPoolCreateInfo
		{
			sType: VkStructureType::CommandPoolCreateInfo, pNext: std::ptr::null(),
			flags: flags, queueFamilyIndex: queue.family_index
		};
		let mut obj: VkCommandPool = std::ptr::null_mut();
		unsafe { vkCreateCommandPool(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| CommandPool { device_ref: self, obj: obj })
	}
	pub fn create_shader_module_from_file(&self, path_to_spirv: &str) -> Result<ShaderModule, VkResult>
	{
		let bin =
		{
			let mut fp = std::fs::File::open(path_to_spirv).expect("Shader binary not found");
			let mut bin: Vec<u8> = Vec::new();
			fp.read_to_end(&mut bin).expect("Unable to read from binary file");
			bin
		};
		let info = VkShaderModuleCreateInfo
		{
			sType: VkStructureType::ShaderModuleCreateInfo, pNext: std::ptr::null(),
			flags: 0, codeSize: bin.len() as size_t, pCode: bin.as_ptr() as *const u32
		};
		let mut obj: VkShaderModule = std::ptr::null_mut();
		unsafe { vkCreateShaderModule(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| ShaderModule { device_ref: self, obj: obj })
	}
	pub fn create_pipeline_layout(&self, descriptor_set_layouts: &[VkDescriptorSetLayout], push_constants: &[VkPushConstantRange]) -> Result<PipelineLayout, VkResult>
	{
		let info = VkPipelineLayoutCreateInfo
		{
			sType: VkStructureType::PipelineLayoutCreateInfo, pNext: std::ptr::null(), flags: 0,
			setLayoutCount: descriptor_set_layouts.len() as u32, pSetLayouts: descriptor_set_layouts.as_ptr(),
			pushConstantRangeCount: push_constants.len() as u32, pPushConstantRanges: push_constants.as_ptr()
		};
		let mut obj: VkPipelineLayout = std::ptr::null_mut();
		unsafe { vkCreatePipelineLayout(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| PipelineLayout { device_ref: self, obj: obj })
	}
	pub fn create_empty_pipeline_cache(&self) -> Result<PipelineCache, VkResult>
	{
		let mut obj: VkPipelineCache = std::ptr::null_mut();
		let info = VkPipelineCacheCreateInfo
		{
			sType: VkStructureType::PipelineCacheCreateInfo, pNext: std::ptr::null(), flags: 0,
			pInitialData: std::ptr::null(), initialDataSize: 0
		};
		unsafe { vkCreatePipelineCache(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| PipelineCache { device_ref: self, obj: obj })
	}
	pub fn create_graphics_pipelines(&self, cache: &PipelineCache, infos: &[VkGraphicsPipelineCreateInfo]) -> Result<Vec<Pipeline>, VkResult>
	{
		let mut objs: Vec<VkPipeline> = vec![std::ptr::null_mut(); infos.len()];
		unsafe { vkCreateGraphicsPipelines(self.obj, cache.get(), infos.len() as u32, infos.as_ptr(), std::ptr::null(), objs.as_mut_ptr()) }
			.to_result().map(|()| objs.into_iter().map(|p| Pipeline { device_ref: self, obj: p }).collect::<Vec<_>>())
	}
	pub fn create_fence(&self) -> Result<Fence, VkResult>
	{
		let info = VkFenceCreateInfo
		{
			sType: VkStructureType::FenceCreateInfo, pNext: std::ptr::null(), flags: 0
		};
		let mut obj: VkFence = std::ptr::null_mut();
		unsafe { vkCreateFence(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| Fence { device_ref: self, obj: obj })
	}
	pub fn create_semaphore(&self) -> Result<Semaphore, VkResult>
	{
		let info = VkSemaphoreCreateInfo
		{
			sType: VkStructureType::SemaphoreCreateInfo, pNext: std::ptr::null(), flags: 0
		};
		let mut obj: VkSemaphore = std::ptr::null_mut();
		unsafe { vkCreateSemaphore(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| Semaphore { device_ref: self, obj: obj })
	}
	/// Creates Exclusive buffer
	pub fn create_buffer(&self, usage_bits: VkBufferUsageFlags, size: VkDeviceSize) -> Result<Buffer, VkResult>
	{
		let buffer_info = VkBufferCreateInfo
		{
			sType: VkStructureType::BufferCreateInfo, pNext: std::ptr::null(), flags: 0,
			usage: usage_bits, size: size, sharingMode: VkSharingMode::Exclusive,
			queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null()
		};
		let mut obj: VkBuffer = std::ptr::null_mut();
		unsafe { vkCreateBuffer(self.obj, &buffer_info, std::ptr::null(), &mut obj) }.to_result().map(|()| Buffer { device_ref: self, obj: obj })
	}
	/// Creates an image with info
	fn create_descripted_image(&self, info: &VkImageCreateInfo) -> Result<Image, VkResult>
	{
		let mut obj: VkImage = unsafe { std::mem::uninitialized() };
		unsafe { vkCreateImage(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| Image { device_ref: self, obj: obj })
	}
	/// Creates Exclusive image
	pub fn create_image(&self, extent: VkExtent2D, tiling: VkImageTiling, usage_bits: VkImageUsageFlags) -> Result<Image, VkResult>
	{
		let VkExtent2D(w, h) = extent;
		self.create_descripted_image(&VkImageCreateInfo
		{
			sType: VkStructureType::ImageCreateInfo, pNext: std::ptr::null(), flags: 0,
			imageType: VkImageType::Dim2, format: VkFormat::R8G8B8A8_UNORM,
			extent: VkExtent3D(w, h, 1), mipLevels: 1, arrayLayers: 1,
			samples: VK_SAMPLE_COUNT_1_BIT, tiling: tiling,
			usage: usage_bits, sharingMode: VkSharingMode::Exclusive,
			queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null(), initialLayout: VkImageLayout::Preinitialized
		})
	}
	pub fn create_single_image(&self, extent: VkExtent2D, tiling: VkImageTiling, usage_bits: VkImageUsageFlags) -> Result<Image, VkResult>
	{
		let VkExtent2D(w, h) = extent;
		self.create_descripted_image(&VkImageCreateInfo
		{
			sType: VkStructureType::ImageCreateInfo, pNext: std::ptr::null(), flags: 0,
			imageType: VkImageType::Dim2, format: VkFormat::R8_UNORM,
			extent: VkExtent3D(w, h, 1), mipLevels: 1, arrayLayers: 1,
			samples: VK_SAMPLE_COUNT_1_BIT, tiling: tiling,
			usage: usage_bits, sharingMode: VkSharingMode::Exclusive,
			queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null(), initialLayout: VkImageLayout::Preinitialized
		})
	}
	pub fn allocate_memory(&self, info: &VkMemoryAllocateInfo) -> Result<DeviceMemory, VkResult>
	{
		let mut obj: VkDeviceMemory = std::ptr::null_mut();
		unsafe { vkAllocateMemory(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| DeviceMemory { device_ref: self, obj: obj })
	}
	pub fn allocate_memory_for_buffer(&self, buffer: &Buffer, memory_property_mask: VkMemoryPropertyFlags) -> Result<DeviceMemory, VkResult>
	{
		let mut obj: VkDeviceMemory = std::ptr::null_mut();
		let info = VkMemoryAllocateInfo
		{
			sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
			allocationSize: buffer.get_memory_requirements().size,
			memoryTypeIndex: self.adapter_ref.get_memory_type_index(memory_property_mask).unwrap() as u32
		};
		if (memory_property_mask & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) == 0 { println!("-- Buffer Memory Consuming: {} bytes", info.allocationSize); }
		unsafe { vkAllocateMemory(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| DeviceMemory { device_ref: self, obj: obj })
	}
	pub fn allocate_memory_for_image(&self, image: &Image, memory_property_mask: VkMemoryPropertyFlags) -> Result<DeviceMemory, VkResult>
	{
		let mut obj: VkDeviceMemory = std::ptr::null_mut();
		let info = VkMemoryAllocateInfo
		{
			sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
			allocationSize: image.get_memory_requirements().size,
			memoryTypeIndex: self.adapter_ref.get_memory_type_index(memory_property_mask).unwrap() as u32
		};
		if (memory_property_mask & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) == 0 { println!("-- Image Memory Consuming: {} bytes", info.allocationSize); }
		unsafe { vkAllocateMemory(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| DeviceMemory { device_ref: self, obj: obj })
	}
	pub fn create_descriptor_set_layout(&self, bindings: &[VkDescriptorSetLayoutBinding]) -> Result<DescriptorSetLayout, VkResult>
	{
		let layout_info = VkDescriptorSetLayoutCreateInfo
		{
			sType: VkStructureType::DescriptorSetLayoutCreateInfo, pNext: std::ptr::null(), flags: 0,
			bindingCount: bindings.len() as u32, pBindings: bindings.as_ptr()
		};
		let mut obj: VkDescriptorSetLayout = std::ptr::null_mut();
		unsafe { vkCreateDescriptorSetLayout(self.obj, &layout_info, std::ptr::null(), &mut obj) }.to_result().map(|()| DescriptorSetLayout { device_ref: self, obj: obj })
	}
	pub fn create_descriptor_pool(&self, max_sets: u32, pool_sizes: &[VkDescriptorPoolSize]) -> Result<DescriptorPool, VkResult>
	{
		let mut obj: VkDescriptorPool = std::ptr::null_mut();
		let info = VkDescriptorPoolCreateInfo
		{
			sType: VkStructureType::DescriptorPoolCreateInfo, pNext: std::ptr::null(),
			flags: 0, maxSets: max_sets,
			poolSizeCount: pool_sizes.len() as u32, pPoolSizes: pool_sizes.as_ptr()
		};
		unsafe { vkCreateDescriptorPool(self.obj, &info, std::ptr::null(), &mut obj) }.to_result().map(|()| DescriptorPool { device_ref: self, obj: obj })
	}
	pub fn create_sampler(&self, info: &VkSamplerCreateInfo) -> Result<Sampler, VkResult>
	{
		let mut obj: VkSampler = std::ptr::null_mut();
		unsafe { vkCreateSampler(self.obj, info, std::ptr::null(), &mut obj) }.to_result().map(|()| Sampler { device_ref: self, obj: obj })
	}

	pub fn update_descriptor_sets(&self, write_infos: &[VkWriteDescriptorSet], copy_infos: &[VkCopyDescriptorSet])
	{
		unsafe { vkUpdateDescriptorSets(self.obj, write_infos.len() as u32, write_infos.as_ptr(), copy_infos.len() as u32, copy_infos.as_ptr()) };
	}

	pub fn wait_for_idle(&self) -> Result<(), VkResult> { unsafe { vkDeviceWaitIdle(self.obj) }.to_result() }
}
pub struct Queue<'d> { device_ref: &'d Device<'d>, obj: VkQueue, pub family_index: u32 }
impl <'d> HasParent for Queue<'d> { type ParentRefType = &'d Device<'d>; fn parent(&self) -> &'d Device<'d> { self.device_ref } }
impl <'d> Queue<'d>
{
	pub fn submit_commands(&self, buffers: &[VkCommandBuffer], device_synchronizer: &[VkSemaphore], device_signalizer: &[VkSemaphore], event_receiver: Option<&Fence>) -> Result<(), VkResult>
	{
		let pipeline_stage = VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT;
		let submit_info = VkSubmitInfo
		{
			sType: VkStructureType::SubmitInfo, pNext: std::ptr::null(),
			waitSemaphoreCount: device_synchronizer.len() as u32, pWaitSemaphores: device_synchronizer.as_ptr(), pWaitDstStageMask: &pipeline_stage,
			commandBufferCount: buffers.len() as u32, pCommandBuffers: buffers.as_ptr(),
			signalSemaphoreCount: device_signalizer.len() as u32, pSignalSemaphores: device_signalizer.as_ptr()
		};
		unsafe { vkQueueSubmit(self.obj, 1, &submit_info, event_receiver.map(|x| x.get()).unwrap_or(std::ptr::null_mut())) }.to_result()
	}
	pub fn wait_for_idle(&self) -> Result<(), VkResult>
	{
		unsafe { vkQueueWaitIdle(self.obj) }.to_result()
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
		pub struct $name<'d> { device_ref: &'d Device<'d>, obj: $t }
		impl <'d> HasParent for $name<'d> { type ParentRefType = &'d Device<'d>; fn parent(&self) -> &'d Device<'d> { self.device_ref } }
		impl <'d> std::ops::Deref for $name<'d> { type Target = $t; fn deref(&self) -> &$t { &self.obj } }
		impl <'d> InternalProvider<$t> for $name<'d> { fn get(&self) -> $t { self.obj } }
	};
}

pub trait VkImageResource
{
	fn get(&self) -> VkImage;
	fn create_view(&self, info: &VkImageViewCreateInfo) -> Result<ImageView, VkResult>;
}
SafeObjectDerivedFromDevice!(Image for VkImage destructed by vkDestroyImage);
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
	fn get(&self) -> VkSurfaceKHR { self.obj }
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
	pub fn acquire_next_image(&self, device_synchronizer: &Semaphore) -> Result<u32, VkResult>
	{
		let mut index: u32 = 0;
		unsafe { vkAcquireNextImageKHR(self.device_ref.obj, self.obj, std::u64::MAX, device_synchronizer.get(), std::ptr::null_mut(), &mut index) }
			.to_result().map(|()| index)
	}
	pub fn present(&self, queue: &Queue, index: u32, device_synchronizer: &[VkSemaphore]) -> Result<(), VkResult>
	{
		let present_info = VkPresentInfoKHR
		{
			sType: VkStructureType::PresentInfoKHR, pNext: std::ptr::null(),
			swapchainCount: 1, pSwapchains: &self.obj, pImageIndices: &index,
			waitSemaphoreCount: device_synchronizer.len() as u32, pWaitSemaphores: device_synchronizer.as_ptr(), pResults: std::ptr::null_mut()
		};
		unsafe { vkQueuePresentKHR(queue.obj, &present_info) }.to_result()
	}
}

SafeObjectDerivedFromDevice!(CommandPool for VkCommandPool destructed by vkDestroyCommandPool);
SafeObjectDerivedFromDevice!(ShaderModule for VkShaderModule destructed by vkDestroyShaderModule);
SafeObjectDerivedFromDevice!(PipelineLayout for VkPipelineLayout destructed by vkDestroyPipelineLayout);
SafeObjectDerivedFromDevice!(PipelineCache for VkPipelineCache destructed by vkDestroyPipelineCache);
SafeObjectDerivedFromDevice!(DescriptorSetLayout for VkDescriptorSetLayout destructed by vkDestroyDescriptorSetLayout);
SafeObjectDerivedFromDevice!(Pipeline for VkPipeline destructed by vkDestroyPipeline);
SafeObjectDerivedFromDevice!(Fence for VkFence destructed by vkDestroyFence);
SafeObjectDerivedFromDevice!(Semaphore for VkSemaphore destructed by vkDestroySemaphore);
SafeObjectDerivedFromDevice!(Buffer for VkBuffer destructed by vkDestroyBuffer);
SafeObjectDerivedFromDevice!(DeviceMemory for VkDeviceMemory destructed by vkFreeMemory);
SafeObjectDerivedFromDevice!(DescriptorPool for VkDescriptorPool destructed by vkDestroyDescriptorPool);
SafeObjectDerivedFromDevice!(Sampler for VkSampler destructed by vkDestroySampler);

impl <'d> CommandPool<'d>
{
	pub fn allocate_primary_buffers(&self, count: usize) -> Result<CommandBuffers, VkResult>
	{
		let allocate_info = VkCommandBufferAllocateInfo
		{
			sType: VkStructureType::CommandBufferAllocateInfo, pNext: std::ptr::null(),
			commandPool: self.obj, level: VkCommandBufferLevel::Primary, commandBufferCount: count as u32
		};
		let mut objs: Vec<VkCommandBuffer> = vec![std::ptr::null_mut(); count];
		unsafe { vkAllocateCommandBuffers(self.device_ref.obj, &allocate_info, objs.as_mut_ptr()) }.to_result()
			.map(|()| CommandBuffers { allocator_ref: self, objects: objs })
	}
}
impl <'d> Fence<'d>
{
	pub fn wait(&self) -> Result<(), VkResult>
	{
		unsafe { vkWaitForFences(self.device_ref.obj, 1, &self.obj, true as VkBool32, std::u64::MAX) }.to_result()
	}
	pub fn reset(&self) -> Result<(), VkResult>
	{
		unsafe { vkResetFences(self.device_ref.obj, 1, &self.obj) }.to_result()
	}
	pub fn get_status(&self) -> Result<(), VkResult>
	{
		unsafe { vkGetFenceStatus(self.device_ref.obj, self.obj) }.to_result()
	}
}
impl <'d> MemoryAllocationRequired for Buffer<'d>
{
	fn get_memory_requirements(&self) -> VkMemoryRequirements
	{
		let mut memreq: VkMemoryRequirements = unsafe { std::mem::uninitialized() };
		unsafe { vkGetBufferMemoryRequirements(self.device_ref.obj, self.obj, &mut memreq) };
		memreq
	}
}
impl <'d> MemoryAllocationRequired for Image<'d>
{
	fn get_memory_requirements(&self) -> VkMemoryRequirements
	{
		let mut memreq: VkMemoryRequirements = unsafe { std::mem::uninitialized() };
		unsafe { vkGetImageMemoryRequirements(self.device_ref.obj, self.obj, &mut memreq) };
		memreq
	}
}
pub struct MemoryMappedRange<'b>
{
	memory_ref: &'b DeviceMemory<'b>, ptr: *mut c_void
}
impl <'d> DeviceMemory<'d>
{
	pub fn map(&'d self, range: std::ops::Range<VkDeviceSize>) -> Result<MemoryMappedRange<'d>, VkResult>
	{
		let mut data_ptr: *mut c_void = std::ptr::null_mut();
		unsafe { vkMapMemory(self.device_ref.obj, self.obj, range.start, range.end - range.start, 0, std::mem::transmute(&mut data_ptr)) }
			.to_result().map(|()| MemoryMappedRange { memory_ref: self, ptr: data_ptr })
	}
	pub fn bind_buffer(&self, buffer: &Buffer, offset: VkDeviceSize) -> Result<(), VkResult>
	{
		unsafe { vkBindBufferMemory(self.device_ref.obj, buffer.obj, self.obj, offset) }.to_result()
	}
	pub fn bind_image(&self, image: &Image, offset: VkDeviceSize) -> Result<(), VkResult>
	{
		unsafe { vkBindImageMemory(self.device_ref.obj, image.obj, self.obj, offset) }.to_result()
	}
}
impl <'b> MemoryMappedRange<'b>
{
	pub fn range_mut<T>(&self, offset: VkDeviceSize, elements: usize) -> &mut [T]
	{
		unsafe
		{
			std::slice::from_raw_parts_mut::<T>(std::mem::transmute(std::mem::transmute::<_, VkDeviceSize>(self.ptr) + offset), elements)
		}
	}
}
impl <'b> std::ops::Drop for MemoryMappedRange<'b>
{
	fn drop(&mut self) { unsafe { vkUnmapMemory(self.memory_ref.device_ref.obj, self.memory_ref.obj) }; }
}
impl <'d> DescriptorPool<'d>
{
	pub fn allocate_sets(&self, layouts: &[VkDescriptorSetLayout]) -> Result<DescriptorSets<'d>, VkResult>
	{
		let mut objs: Vec<VkDescriptorSet> = vec![unsafe { std::mem::uninitialized() }; layouts.len()];
		let info = VkDescriptorSetAllocateInfo
		{
			sType: VkStructureType::DescriptorSetAllocateInfo, pNext: std::ptr::null(),
			descriptorPool: self.obj, descriptorSetCount: layouts.len() as u32, pSetLayouts: layouts.as_ptr()
		};
		unsafe { vkAllocateDescriptorSets(self.device_ref.obj, &info, objs.as_mut_ptr()) }.to_result().map(|()| DescriptorSets { device_ref: self.device_ref, objs: objs })
	}
}

// Set of Command Buffers and Reference //
pub struct CommandBuffers<'d>
{
	allocator_ref: &'d CommandPool<'d>, objects: Vec<VkCommandBuffer>
}
pub struct CommandBufferRef { obj: VkCommandBuffer }
impl <'d> std::ops::Drop for CommandBuffers<'d>
{
	fn drop(&mut self) { unsafe { vkFreeCommandBuffers(self.allocator_ref.device_ref.obj, self.allocator_ref.obj, self.objects.len() as u32, self.objects.as_ptr()) }; }
}
impl <'d> CommandBuffers<'d>
{
	pub fn begin(&self, i: usize) -> Result<CommandBufferRef, VkResult>
	{
		let begin_info = VkCommandBufferBeginInfo
		{
			sType: VkStructureType::CommandBufferBeginInfo, pNext: std::ptr::null(),
			flags: 0, pInheritanceInfo: std::ptr::null()
		};
		unsafe { vkBeginCommandBuffer(self.objects[i], &begin_info) }.to_result().map(|()| CommandBufferRef { obj: self.objects[i] })
	}
}
impl <'d> std::ops::Index<usize> for CommandBuffers<'d>
{
	type Output = VkCommandBuffer;

	fn index(&self, i: usize) -> &VkCommandBuffer { &self.objects[i] }
}
impl CommandBufferRef
{
	pub fn begin_render_pass(self, fb: &Framebuffer, rp: &RenderPass, area: VkRect2D, clear_values: &[VkClearValue], use_secondary_buffers: bool) -> Self
	{
		let rp_begin_info = VkRenderPassBeginInfo
		{
			sType: VkStructureType::RenderPassBeginInfo, pNext: std::ptr::null(),
			framebuffer: fb.get(), renderPass: rp.get(), renderArea: area,
			clearValueCount: clear_values.len() as u32, pClearValues: clear_values.as_ptr()
		};
		let content_flag = if use_secondary_buffers { VkSubpassContents::SecondaryCommandBuffers } else { VkSubpassContents::Inline };
		unsafe { vkCmdBeginRenderPass(self.obj, &rp_begin_info, content_flag) };
		self
	}
	pub fn end_render_pass(self) -> Self
	{
		unsafe { vkCmdEndRenderPass(self.obj) };
		self
	}
	pub fn resource_barrier(self, src_stage: VkPipelineStageFlags, dst_stage: VkPipelineStageFlags,
		memory_barriers: &[VkMemoryBarrier], buffer_barriers: &[VkBufferMemoryBarrier], image_barriers: &[VkImageMemoryBarrier]) -> Self
	{
		unsafe
		{
			vkCmdPipelineBarrier(self.obj, src_stage, dst_stage, 0,
				memory_barriers.len() as u32, memory_barriers.as_ptr(), buffer_barriers.len() as u32, buffer_barriers.as_ptr(),
				image_barriers.len() as u32, image_barriers.as_ptr())
		};
		self
	}
	pub fn bind_pipeline(self, pipeline: &Pipeline) -> Self
	{
		unsafe { vkCmdBindPipeline(self.obj, VkPipelineBindPoint::Graphics, pipeline.get()) };
		self
	}
	pub fn bind_descriptor_sets(self, layout: &PipelineLayout, sets: &[VkDescriptorSet], dynamic_offsets: &[u32]) -> Self
	{
		unsafe { vkCmdBindDescriptorSets(self.obj, VkPipelineBindPoint::Graphics, layout.get(), 0, sets.len() as u32, sets.as_ptr(),
			dynamic_offsets.len() as u32, dynamic_offsets.as_ptr()) };
		self
	}
	pub fn bind_vertex_buffers(self, buffers: &[VkBuffer], offsets: &[VkDeviceSize]) -> Self
	{
		unsafe { vkCmdBindVertexBuffers(self.obj, 0, buffers.len() as u32, buffers.as_ptr(), offsets.as_ptr()) };
		self
	}
	pub fn bind_index_buffer(self, buffer: &Buffer, offset: VkDeviceSize) -> Self
	{
		unsafe { vkCmdBindIndexBuffer(self.obj, buffer.get(), offset, VkIndexType::U16) };
		self
	}
	pub fn draw(self, vertex_count: u32, instance_count: u32) -> Self
	{
		unsafe { vkCmdDraw(self.obj, vertex_count, instance_count, 0, 0) };
		self
	}
	pub fn draw_indexed(self, vertex_count: u32, instance_count: u32, instance_start_index: u32) -> Self
	{
		unsafe { vkCmdDrawIndexed(self.obj, vertex_count, instance_count, 0, 0, instance_start_index) };
		self
	}
	pub fn draw_indexed_indirect(self, buffer: &VkBuffer, offset: VkDeviceSize) -> Self
	{
		unsafe { vkCmdDrawIndexedIndirect(self.obj, *buffer, offset, 1, std::mem::size_of::<VkDrawIndexedIndirectCommand>() as u32) };
		self
	}
	pub fn push_constants<T: std::marker::Sized>(self, layout: &PipelineLayout, stage: VkShaderStageFlags, offset: u32, values: &[T]) -> Self
	{
		unsafe { vkCmdPushConstants(self.obj, layout.get(), stage, offset, (std::mem::size_of::<T>() * values.len()) as u32, std::mem::transmute(values.as_ptr())) };
		self
	}

	// Copy Commands //
	pub fn copy_buffer(self, src: &Buffer, dst: &Buffer, regions: &[VkBufferCopy]) -> Self
	{
		unsafe { vkCmdCopyBuffer(self.obj, src.get(), dst.get(), regions.len() as u32, regions.as_ptr()) };
		self
	}
	pub fn copy_image(self, src: &Image, src_layout: VkImageLayout, dst: &Image, dst_layout: VkImageLayout, regions: &[VkImageCopy]) -> Self
	{
		unsafe { vkCmdCopyImage(self.obj, src.get(), src_layout, dst.get(), dst_layout, regions.len() as u32, regions.as_ptr()) };
		self
	}
}
impl std::ops::Drop for CommandBufferRef
{
	fn drop(&mut self) { unsafe { vkEndCommandBuffer(self.obj) }.to_result().unwrap() }
}

pub struct DescriptorSets<'d>
{
	#[allow(dead_code)] device_ref: &'d Device<'d>, objs: Vec<VkDescriptorSet>
}
impl <'d> std::ops::Index<usize> for DescriptorSets<'d>
{
	type Output = VkDescriptorSet;
	fn index<'a>(&'a self, index: usize) -> &'a VkDescriptorSet { &self.objs[index] }
}
