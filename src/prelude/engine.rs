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
use std::io::prelude::*;

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
			println!("{}", match record.level()
			{
				log::LogLevel::Error => Style::new().bold().fg(Color::Red).paint(format!("!! [{}:{}] {}", record.target(), record.level(), record.args())),
				_ => Style::new().bold().paint(format!("** [{}:{}] {}", record.target(), record.level(), record.args()))
			});
		}
	}
}

pub trait EngineExports
{
	fn get_window_server(&self) -> &Rc<WindowServer>;
	fn get_instance(&self) -> &Rc<vk::Instance>;
	fn get_device(&self) -> &DeviceExports;
	fn get_memory_type_index_for_device_local(&self) -> u32;
	fn get_memory_type_index_for_host_visible(&self) -> u32;
}
pub struct Engine
{
	window_system: Rc<WindowServer>, instance: Rc<vk::Instance>, #[allow(dead_code)] debug_callback: vk::DebugReportCallback,
	device: Device, pools: CommandPool, pipeline_cache: Rc<vk::PipelineCache>,
	asset_dir: std::path::PathBuf,
	physical_device_limits: VkPhysicalDeviceLimits,
	memory_type_index_for_device_local: u32, memory_type_index_for_host_visible: u32
}
impl std::ops::Drop for Engine
{
	fn drop(&mut self) { self.device.wait_for_idle().unwrap(); }
}
impl EngineExports for Engine
{
	fn get_window_server(&self) -> &Rc<WindowServer> { &self.window_system }
	fn get_instance(&self) -> &Rc<vk::Instance> { &self.instance }
	fn get_device(&self) -> &DeviceExports { &self.device }
	fn get_memory_type_index_for_device_local(&self) -> u32 { self.memory_type_index_for_device_local }
	fn get_memory_type_index_for_host_visible(&self) -> u32 { self.memory_type_index_for_host_visible }
}
impl Engine
{
	pub fn new(app_name: &str, app_version: u32) -> Result<Box<Engine>, EngineError>
	{
		// Setup Engine Logger //
		log::set_logger(|max_log_level| { max_log_level.set(log::LogLevelFilter::Info); Box::new(EngineLogger) }).unwrap();
		info!(target: "Prelude", "Initializing Engine...");

		let window_server = try!(connect_to_window_server());

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
			Self::diagnose_adapter(&*window_server, &adapter, graphics_qf);
			let device_features = VkPhysicalDeviceFeatures { geometryShader: 1, .. Default::default() };
			try!(Device::new(&adapter, &device_features, graphics_qf, transfer_qf, &queue_family_properties[graphics_qf as usize]))
		};
		let pools = try!(CommandPool::new(&device));
		let pipeline_cache = Rc::new(try!(vk::PipelineCache::new_empty(&device)));

		let memory_types = adapter.get_memory_properties();
		let mt_index_for_device_local = try!(memory_types.memoryTypes[..memory_types.memoryTypeCount as usize].iter()
			.enumerate().find(|&(_, &VkMemoryType(flags, _))| (flags & VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT) != 0)
			.map(|(i, _)| i as u32).ok_or(EngineError::GenericError("Device Local Memory is not found")));
		let mt_index_for_host_visible = try!(memory_types.memoryTypes[..memory_types.memoryTypeCount as usize].iter()
			.enumerate().find(|&(_, &VkMemoryType(flags, _))| (flags & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) != 0)
			.map(|(i, _)| i as u32).ok_or(EngineError::GenericError("Host Visible Memory is not found")));

		Ok(Box::new(Engine
		{
			window_system: window_server, instance: instance, debug_callback: dbg_callback, device: device, pools: pools,
			pipeline_cache: pipeline_cache,
			asset_dir: std::env::current_exe().unwrap().parent().unwrap().to_path_buf().join("assets"),
			physical_device_limits: adapter.get_properties().limits,
			memory_type_index_for_device_local: mt_index_for_device_local,
			memory_type_index_for_host_visible: mt_index_for_host_visible
		}))
	}
	pub fn with_assets_in(mut self, asset_dir_base: std::path::PathBuf) -> Self
	{
		self.asset_dir = asset_dir_base.join("assets");
		self
	}
	pub fn process_messages(&self) -> bool
	{
		self.window_system.process_events() == ApplicationState::Continued
	}
	pub fn create_render_window(&self, size: VkExtent2D, title: &str) -> Result<Box<RenderWindow>, EngineError>
	{
		info!(target: "Prelude", "Creating Render Window \"{}\" ({}x{})", title, size.0, size.1);
		Window::create_unresizable(self, size, title).map(|x| x as Box<RenderWindow>)
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
		vk::Framebuffer::new(&self.device, &info).map(|f| Framebuffer::new(f, mold.get_internal(), VkExtent2D(width, height))).map_err(EngineError::from)
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
	pub fn create_vertex_shader_from_asset(&self, asset_path: &str, entry_point: &str,
		vertex_bindings: &[VertexBinding], vertex_attributes: &[VertexAttribute]) -> Result<ShaderProgram, EngineError>
	{
		let entity_path = self.parse_asset(asset_path, "spv");
		info!(target: "Prelude", "Loading Vertex Shader {:?}", entity_path);
		std::fs::File::open(entity_path).map_err(EngineError::from).and_then(|mut fp|
		{
			let mut bin: Vec<u8> = Vec::new();
			fp.read_to_end(&mut bin).map(move |_| bin).map_err(EngineError::from)
		}).and_then(|b| vk::ShaderModule::new(self.device.get_internal(), &b).map_err(EngineError::from))
		.map(|m| ShaderProgram::new_vertex(m, entry_point, vertex_bindings, vertex_attributes))
	}
	pub fn create_geometry_shader_from_asset(&self, asset_path: &str, entry_point: &str) -> Result<ShaderProgram, EngineError>
	{
		let entity_path = self.parse_asset(asset_path, "spv");
		info!(target: "Prelude", "Loading Geometry Shader {:?}", entity_path);
		std::fs::File::open(entity_path).map_err(EngineError::from).and_then(|mut fp|
		{
			let mut bin: Vec<u8> = Vec::new();
			fp.read_to_end(&mut bin).map(move |_| bin).map_err(EngineError::from)
		}).and_then(|b| vk::ShaderModule::new(self.device.get_internal(), &b).map_err(EngineError::from))
		.map(|m| ShaderProgram::new_geometry(m, entry_point))
	}
	pub fn create_fragment_shader_from_asset(&self, asset_path: &str, entry_point: &str) -> Result<ShaderProgram, EngineError>
	{
		let entity_path = self.parse_asset(asset_path, "spv");
		info!(target: "Prelude", "Loading Fragment Shader {:?}", entity_path);
		std::fs::File::open(entity_path).map_err(EngineError::from).and_then(|mut fp|
		{
			let mut bin: Vec<u8> = Vec::new();
			fp.read_to_end(&mut bin).map(|_| bin).map_err(EngineError::from)
		}).and_then(|b| vk::ShaderModule::new(self.device.get_internal(), &b).map_err(EngineError::from))
		.map(|m| ShaderProgram::new_fragment(m, entry_point))
	}
	pub fn create_pipeline_layout(&self, descriptor_sets: &[&DescriptorSetLayout], push_constants: &[PushConstantDesc])
		-> Result<PipelineLayout, EngineError>
	{
		vk::PipelineLayout::new(self.device.get_internal(),
			&descriptor_sets.into_iter().map(|x| x.get_internal().get()).collect::<Vec<_>>(),
			&push_constants.into_iter().map(|x| x.into()).collect::<Vec<_>>()).map(PipelineLayout::new).map_err(EngineError::from)
	}
	pub fn create_graphics_pipelines(&self, builders: &[&GraphicsPipelineBuilder]) -> Result<Vec<GraphicsPipeline>, EngineError>
	{
		let builder_into_natives = builders.into_iter().map(|&x| x.into()).collect::<Vec<IntoNativeGraphicsPipelineCreateInfoStruct>>();
		vk::Pipeline::new(self.device.get_internal(), &self.pipeline_cache,
			&builder_into_natives.iter().map(|x| x.into()).collect::<Vec<_>>())
			.map(|v| v.into_iter().map(GraphicsPipeline::new).collect::<Vec<_>>()).map_err(EngineError::from)
	}
	pub fn create_double_buffer(&self, preallocator: &MemoryPreallocator) -> Result<(DeviceBuffer, StagingBuffer), EngineError>
	{
		info!(target: "Prelude", "Allocated device memory: {} bytes(double-buffered)", preallocator.get_total_size());
		DeviceBuffer::new(self, preallocator.get_total_size(), preallocator.get_usage()).and_then(|db|
		StagingBuffer::new(self, preallocator.get_total_size()).map(|sb| (db, sb)))
	}
	pub fn create_descriptor_set_layout(&self, bindings: &[Descriptor]) -> Result<DescriptorSetLayout, EngineError>
	{
		let native = bindings.into_iter().enumerate().map(|(i, x)| x.into_binding(i as u32)).collect::<Vec<_>>();
		vk::DescriptorSetLayout::new(self.device.get_internal(), &native)
			.map(|d| DescriptorSetLayout::new(d, bindings)).map_err(EngineError::from)
	}
	pub fn preallocate_all_descriptor_sets(&self, layouts: &[&DescriptorSetLayout]) -> Result<DescriptorSets, EngineError>
	{
		let set_count = layouts.len();
		let (uniform_total, combined_sampler_total) = layouts.iter().map(|x| x.descriptors().into_iter().fold((0, 0), |(u, cs), desc| match desc
		{
			&Descriptor::Uniform(n, _) => (u + n, cs),
			&Descriptor::CombinedSampler(n, _) => (u, cs + n)
		})).fold((0, 0), |(u, cs), (u2, cs2)| (u + u2, cs + cs2));
		let pool_sizes = [Descriptor::Uniform(uniform_total, vec![]), Descriptor::CombinedSampler(combined_sampler_total, vec![])]
			.into_iter().filter(|&desc| desc.count() != 0).map(|desc| desc.into_pool_size()).collect::<Vec<_>>();
		
		vk::DescriptorPool::new(self.device.get_internal(), set_count as u32, &pool_sizes).and_then(|pool|
		pool.allocate_sets(self.device.get_internal(), &layouts.into_iter().map(|x| x.get_internal().get()).collect::<Vec<_>>())
			.map(|sets| DescriptorSets::new(pool, sets))).map_err(EngineError::from)
	}

	fn parse_asset(&self, asset_path: &str, extension: &str) -> std::ffi::OsString
	{
		// ident ("." ident)* -> ident ("/" ident)*
		self.asset_dir.join(asset_path.replace(".", "/")).with_extension(extension).into()
	}

	pub fn preallocate(&self, structure_sizes: &[(usize, BufferDataType)]) -> MemoryPreallocator
	{
		let uniform_alignment = self.physical_device_limits.minUniformBufferOffsetAlignment as usize;
		let usage_flags = structure_sizes.iter().fold(0, |flags_accum, &(_, data_type)| match data_type
		{
			BufferDataType::Vertex => flags_accum | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
			BufferDataType::Index => flags_accum | VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
			BufferDataType::Uniform => flags_accum | VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT
		});
		let offsets = structure_sizes.into_iter().chain(&[(0, BufferDataType::Vertex)]).scan(0usize, |offset_accum, &(size, data_type)|
		{
			let current = *offset_accum;
			*offset_accum = match data_type
			{
				BufferDataType::Vertex | BufferDataType::Index => *offset_accum,
				BufferDataType::Uniform => ((*offset_accum as f64 / uniform_alignment as f64).ceil() as usize) * uniform_alignment as usize
			} + size;
			Some(current)
		}).collect::<Vec<_>>();

		MemoryPreallocator::new(usage_flags, offsets)
	}

	pub fn submit_graphics_commands(&self, commands: &GraphicsCommandBuffers, wait_for_execute: &[(&QueueFence, VkPipelineStageFlags)],
		signal_on_complete: Option<&QueueFence>, signal_on_complete_host: Option<&Fence>) -> Result<(), EngineError>
	{
		self.device.get_graphics_queue().submit_commands(commands.get_internal(),
			&wait_for_execute.into_iter().map(|&(q, _)| q.get_internal().get()).collect::<Vec<_>>(),
			&wait_for_execute.into_iter().map(|&(_, s)| s).collect::<Vec<_>>(),
			&signal_on_complete.map(|q| vec![q.get_internal().get()]).unwrap_or(vec![]), signal_on_complete_host.map(|f| f.get_internal()))
			.map_err(EngineError::from)
	}
	pub fn submit_transfer_commands(&self, commands: &TransferCommandBuffers, wait_for_execute: &[(&QueueFence, VkPipelineStageFlags)],
		signal_on_complete: Option<&QueueFence>, signal_on_complete_host: Option<&Fence>) -> Result<(), EngineError>
	{
		self.device.get_transfer_queue().submit_commands(commands.get_internal(),
			&wait_for_execute.into_iter().map(|&(q, _)| q.get_internal().get()).collect::<Vec<_>>(),
			&wait_for_execute.into_iter().map(|&(_, s)| s).collect::<Vec<_>>(),
			&signal_on_complete.map(|q| vec![q.get_internal().get()]).unwrap_or(vec![]), signal_on_complete_host.map(|f| f.get_internal()))
			.map_err(EngineError::from)
	}

	fn diagnose_adapter(server_con: &WindowServer, adapter: &vk::PhysicalDevice, queue_index: u32)
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
