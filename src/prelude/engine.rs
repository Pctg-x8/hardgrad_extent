// Prelude: Engine and EngineLogger

use prelude::internals::*;
use {std, log};
use ansi_term::*;
use std::rc::Rc;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use libc::size_t;
use std::os::raw::*;
use std::ffi::CStr;

// Platform Depends //
use xcbw::XServerConnection;

struct EngineLogger;
impl log::Log for EngineLogger
{
	fn enabled(&self, metadata: &log::LogMetadata) -> bool
	{
		metadata.level() <= log::LogLevel::Info
	}
	fn log(&self, record: &log::LogRecord)
	{
		if self.enabled(record.metadata())
		{
			println!("{}", Style::new().bold().paint(format!("** [{}:{}]{}", record.target(), record.level(), record.args())));
		}
	}
}

pub trait EngineExports
{
	fn get_window_server(&self) -> &Rc<XServerConnection>;
	fn get_instance(&self) -> &Rc<vk::Instance>;
	fn get_device(&self) -> &DeviceExports;
}
pub struct Engine
{
	window_system: Rc<XServerConnection>, instance: Rc<vk::Instance>, #[allow(dead_code)] debug_callback: vk::DebugReportCallback,
	device: Device, pools: CommandPool
}
impl EngineExports for Engine
{
	fn get_window_server(&self) -> &Rc<XServerConnection> { &self.window_system }
	fn get_instance(&self) -> &Rc<vk::Instance> { &self.instance }
	fn get_device(&self) -> &DeviceExports { &self.device }
}
impl Engine
{
	pub fn new(app_name: &str, app_version: u32) -> Result<Engine, EngineError>
	{
		// Setup Engine Logger //
		log::set_logger(|max_log_level| { max_log_level.set(log::LogLevelFilter::Info); Box::new(EngineLogger) }).unwrap();
		info!(target: "Prelude", "Initializing Engine...");

		// ready for window system //
		let window_server = Rc::new(XServerConnection::connect());

		let instance = try!(vk::Instance::new(app_name, app_version, "Prelude Computer-Graphics Engine", VK_MAKE_VERSION!(0, 0, 1),
			&["VK_LAYER_LUNARG_standard_validation"], &["VK_KHR_surface", "VK_KHR_xcb_surface", "VK_EXT_debug_report"]).map(|x| Rc::new(x)));
		let dbg_callback = try!(vk::DebugReportCallback::new(&instance, device_report_callback));
		let adapter = try!(instance.enumerate_adapters().map_err(|e| EngineError::from(e))
			.and_then(|aa| aa.into_iter().next().ok_or(EngineError::GenericError("PhysicalDevices are not found")))
			.map(|a| Rc::new(vk::PhysicalDevice::from(a, &instance))));
		let device =
		{
			let queue_family_properties = adapter.enumerate_queue_family_properties();
			let graphics_qf = try!(queue_family_properties.iter().enumerate().find(|&(_, fp)| (fp.queueFlags & VK_QUEUE_GRAPHICS_BIT) != 0)
				.map(|(i, _)| i as u32).ok_or(EngineError::GenericError("Unable to find Graphics Queue")));
			let transfer_qf = queue_family_properties.iter().enumerate().filter(|&(i, _)| i as u32 != graphics_qf)
				.find(|&(_, fp)| (fp.queueFlags & VK_QUEUE_TRANSFER_BIT) != 0).map(|(i, _)| i as u32);
			Self::diagnose_adapter(&window_server, &adapter, graphics_qf);
			let device_features = VkPhysicalDeviceFeatures { geometryShader: 1, .. Default::default() };
			try!(Device::new(&adapter, &device_features, graphics_qf, transfer_qf, &queue_family_properties[graphics_qf as usize]))
		};
		let pools = try!(CommandPool::new(&device));

		Ok(Engine
		{
			window_system: window_server, instance: instance, debug_callback: dbg_callback, device: device, pools: pools
		})
	}
	pub fn create_render_window(&self, size: VkExtent2D, title: &str) -> Result<Box<RenderWindow>, EngineError>
	{
		info!(target: "Prelude", "Creating Render Window \"{}\" ({}x{})", title, size.0, size.1);
		XcbWindow::create_unresizable(self, size, title).map(|x| x as Box<RenderWindow>)
	}
	pub fn process_messages(&self) -> bool
	{
		self.window_system.process_messages()
	}
	
	pub fn create_fence(&self) -> Result<Fence, EngineError>
	{
		vk::Fence::new(&self.device).map(Fence::new).map_err(EngineError::from)
	}
	pub fn create_queue_fence(&self) -> Result<QueueFence, EngineError>
	{
		vk::Semaphore::new(&self.device).map(QueueFence::new).map_err(EngineError::from)
	}
	pub fn create_render_pass(&self, attachments: &[AttachmentDesc], passes: &[PassDesc], deps: &[PassDependency])
		-> Result<RenderPass, EngineError>
	{
		let attachments_native = attachments.into_iter().map(|x| x.into()).collect::<Vec<_>>();
		let subpasses_native = passes.into_iter().map(|x| x.into()).collect::<Vec<_>>();
		let deps_native = deps.into_iter().map(|x| x.into()).collect::<Vec<_>>();
		let rp_info = VkRenderPassCreateInfo
		{
			sType: VkStructureType::RenderPassCreateInfo, pNext: std::ptr::null(), flags: 0,
			attachmentCount: attachments_native.len() as u32, pAttachments: attachments_native.as_ptr(),
			subpassCount: subpasses_native.len() as u32, pSubpasses: subpasses_native.as_ptr(),
			dependencyCount: deps_native.len() as u32, pDependencies: deps_native.as_ptr()
		};
		vk::RenderPass::new(&self.device, &rp_info).map(RenderPass::new).map_err(EngineError::from)
	}
	pub fn create_framebuffer(&self, mold: &RenderPass, attachments: &[&vk::ImageView], form: VkExtent3D) -> Result<Framebuffer, EngineError>
	{
		let attachments_native = attachments.into_iter().map(|x| x.get()).collect::<Vec<_>>();
		let VkExtent3D(width, height, layers) = form;
		let info = VkFramebufferCreateInfo
		{
			sType: VkStructureType::FramebufferCreateInfo, pNext: std::ptr::null(), flags: 0,
			renderPass: mold.get_internal().get(),
			attachmentCount: attachments_native.len() as u32, pAttachments: attachments_native.as_ptr(),
			width: width, height: height, layers: layers
		};
		vk::Framebuffer::new(&self.device, &info).map(|f| Framebuffer::new(f, mold.get_internal())).map_err(EngineError::from)
	}
	pub fn allocate_graphics_command_buffers(&self, count: u32) -> Result<GraphicsCommandBuffers, EngineError>
	{
		self.pools.for_graphics().allocate_buffers(&self.device, VkCommandBufferLevel::Primary, count).map_err(EngineError::from)
			.map(|v| GraphicsCommandBuffers::new(self.pools.for_graphics(), v))
	}
	pub fn allocate_transfer_command_buffers(&self, count: u32) -> Result<TransferCommandBuffers, EngineError>
	{
		self.pools.for_transfer().allocate_buffers(&self.device, VkCommandBufferLevel::Primary, count).map_err(EngineError::from)
			.map(|v| TransferCommandBuffers::new(self.pools.for_transfer(), v))
	}
	pub fn allocate_transient_transfer_command_buffers(&self, count: u32) -> Result<TransientTransferCommandBuffers, EngineError>
	{
		self.pools.for_transient().allocate_buffers(&self.device, VkCommandBufferLevel::Primary, count).map_err(EngineError::from)
			.map(|v| TransientTransferCommandBuffers::new(self.pools.for_transient(), self.device.get_transfer_queue(), v))
	}

	fn diagnose_adapter(server_con: &XServerConnection, adapter: &vk::PhysicalDevice, queue_index: u32)
	{
		// Feature Check //
		let features = adapter.get_features();
		info!(target: "Prelude::DiagAdapter", "adapter features");
		info!(target: "Prelude::DiagAdapter", "-- independentBlend: {}", features.independentBlend);
		info!(target: "Prelude::DiagAdapter", "-- geometryShader: {}", features.geometryShader);
		info!(target: "Prelude::DiagAdapter", "-- multiDrawIndirect: {}", features.multiDrawIndirect);
		info!(target: "Prelude::DiagAdapter", "-- drawIndirectFirstInstance: {}", features.drawIndirectFirstInstance);
		info!(target: "Prelude::DiagAdapter", "-- shaderTessellationAndGeometryPointSize: {}", features.shaderTessellationAndGeometryPointSize);
		info!(target: "Prelude::DiagAdapter", "-- depthClamp: {}", features.depthClamp);
		info!(target: "Prelude::DiagAdapter", "-- depthBiasClamp: {}", features.depthBiasClamp);
		info!(target: "Prelude::DiagAdapter", "-- wideLines: {}", features.wideLines);
		info!(target: "Prelude::DiagAdapter", "-- alphaToOne: {}", features.alphaToOne);
		info!(target: "Prelude::DiagAdapter", "-- multiViewport: {}", features.multiViewport);
		info!(target: "Prelude::DiagAdapter", "-- shaderCullDistance: {}", features.shaderCullDistance);
		info!(target: "Prelude::DiagAdapter", "-- shaderClipDistance: {}", features.shaderClipDistance);
		info!(target: "Prelude::DiagAdapter", "-- shaderResourceResidency: {}", features.shaderResourceResidency);
		// if features.depthClamp == false as VkBool32 { panic!("DepthClamp Feature is required in device"); }

		// Vulkan and XCB Integration Check //
		if !server_con.is_vk_presentation_support(adapter, queue_index) { panic!("Vulkan Presentation is not supported by window system"); }
	}
}

unsafe extern "system" fn device_report_callback(flags: VkDebugReportFlagsEXT, object_type: VkDebugReportObjectTypeEXT, _: u64,
	_: size_t, _: i32, _: *const c_char, message: *const c_char, _: *mut c_void) -> VkBool32
{
	if (flags & VK_DEBUG_REPORT_ERROR_BIT_EXT) != 0
	{
		error!(target: format!("Vulkan DebugCall [{:?}]", object_type).as_str(), "{}", CStr::from_ptr(message).to_str().unwrap());
	}
	else if (flags & VK_DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT) != 0
	{
		warn!(target: format!("Vulkan PerformanceDebug [{:?}]", object_type).as_str(), "{}", CStr::from_ptr(message).to_str().unwrap());
	}
	else if (flags & VK_DEBUG_REPORT_WARNING_BIT_EXT) != 0
	{
		warn!(target: format!("Vulkan DebugCall [{:?}]", object_type).as_str(), "{}", CStr::from_ptr(message).to_str().unwrap());
	}
	else
	{
		info!(target: format!("Vulkan DebugCall [{:?}]", object_type).as_str(), "{}", CStr::from_ptr(message).to_str().unwrap());
	}
	true as VkBool32
}
