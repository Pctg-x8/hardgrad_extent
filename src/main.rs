extern crate libc;
extern crate xcb;
extern crate nalgebra;
extern crate rand;
extern crate time;
extern crate freetype;
extern crate unicode_normalization;
extern crate thread_scoped;
#[macro_use] extern crate log;
extern crate ansi_term;
#[macro_use] mod vkffi;
mod render_vk;

mod constants;
use constants::*;
mod traits;
use traits::*;
mod xcbw;
use xcbw::*;
mod vertex_formats;
mod device_resources;
mod structures;
mod logical_resources;
mod utils;
use nalgebra::*;
use rand::distributions::*;

use vkffi::*; use ansi_term::*;
use render_vk::wrap as vk;
use render_vk::traits::*;

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

// Application Dependent Factories
fn create_instance() -> vk::Instance
{
	let layers = [DEBUG_LAYER_NAME.as_ptr()];
	let extensions = [SURFACE_EXTENSION_NAME.as_ptr(), PSURFACE_EXTENSION_NAME.as_ptr(), DEBUG_EXTENSION_NAME.as_ptr()];
	let app_info = VkApplicationInfo
	{
		pApplicationName: APP_NAME.as_ptr() as *const i8,
		applicationVersion: VK_MAKE_VERSION!(0, 0, 1),
		pEngineName: ENGINE_NAME.as_ptr() as *const i8,
		engineVersion: VK_MAKE_VERSION!(0, 0, 1),
		.. Default::default()
	};
	let instance_info = VkInstanceCreateInfo
	{
		pApplicationInfo: &app_info,
		enabledLayerCount: layers.len() as u32, ppEnabledLayerNames: layers.as_ptr() as *const *const i8,
		enabledExtensionCount: extensions.len() as u32, ppEnabledExtensionNames: extensions.as_ptr() as *const *const i8,
		.. Default::default()
	};

	info!("creating Vulkan Instance");

	vk::Instance::create(&instance_info).expect("Unable to create instance")
}
fn diagnose_adapter(server_con: &XServerConnection, adapter: &vk::PhysicalDevice, queue_index: u32)
{
	// Feature Check //
	let features = adapter.get_features();
	println!("-- adapter features");
	println!("---- independentBlend: {}", features.independentBlend);
	println!("---- geometryShader: {}", features.geometryShader);
	println!("---- multiDrawIndirect: {}", features.multiDrawIndirect);
	println!("---- drawIndirectFirstInstance: {}", features.drawIndirectFirstInstance);
	println!("---- shaderTessellationAndGeometryPointSize: {}", features.shaderTessellationAndGeometryPointSize);
	println!("---- depthClamp: {}", features.depthClamp);
	println!("---- depthBiasClamp: {}", features.depthBiasClamp);
	println!("---- wideLines: {}", features.wideLines);
	println!("---- alphaToOne: {}", features.alphaToOne);
	println!("---- multiViewport: {}", features.multiViewport);
	println!("---- shaderCullDistance: {}", features.shaderCullDistance);
	println!("---- shaderClipDistance: {}", features.shaderClipDistance);
	println!("---- shaderResourceResidency: {}", features.shaderResourceResidency);
	// if features.depthClamp == false as VkBool32 { panic!("DepthClamp Feature is required in device"); }

	// Vulkan and XCB Integration Check //
	if !server_con.is_vk_presentation_support(adapter, queue_index) { panic!("Unsupported Display Format"); }
}
fn create_surface<'i, 'c>(instance_ref: &'i vk::Instance, window: &XWindow<'c>) -> vk::Surface<'i>
{
	let xcb_surface_info = VkXcbSurfaceCreateInfoKHR
	{
		sType: VkStructureType::XcbSurfaceCreateInfoKHR, pNext: std::ptr::null(), flags: 0,
		connection: window.parent().get_raw(), window: window.get()
	};
	vk::Surface::create(instance_ref, &xcb_surface_info).unwrap()
}
fn create_swapchain<'d>(queue: &vk::Queue<'d>, surface: &vk::Surface) -> (vk::Swapchain<'d>, VkFormat, VkExtent2D)
{
	// capabilities check //
	if !queue.parent().parent().is_surface_support(queue.family_index, surface) { panic!("Unsupported Surface"); }
	let surface_caps = queue.parent().parent().get_surface_capabilities(surface);

	// making desired parameters //
	let format = queue.parent().parent().enumerate_surface_formats(surface).into_iter()
		.filter(|ref x| x.format == VkFormat::B8G8R8A8_SRGB || x.format == VkFormat::R8G8B8A8_SRGB)
		.next().expect("Desired format is not found");
	let present_mode = queue.parent().parent().enumerate_present_modes(surface).into_iter().filter(|ref x| **x == VkPresentModeKHR::Mailbox || **x == VkPresentModeKHR::FIFO)
		.next().expect("Desired Present Mode is not found");
	let sc_extent = match surface_caps.currentExtent
	{
		VkExtent2D(w, h) if w == std::u32::MAX || h == std::u32::MAX => { VkExtent2D(640, 480) },
		e => e
	};

	// set information and create //
	let queue_family_indices = [queue.family_index];
	let swapchain_info = VkSwapchainCreateInfoKHR
	{
		sType: VkStructureType::SwapchainCreateInfoKHR, pNext: std::ptr::null(),
		minImageCount: surface_caps.minImageCount + 1, imageFormat: format.format, imageColorSpace: format.colorSpace,
		imageExtent: sc_extent, imageArrayLayers: 1, imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as u32,
		imageSharingMode: VkSharingMode::Exclusive, compositeAlpha: VkCompositeAlphaFlagBitsKHR::Opaque,
		preTransform: VkSurfaceTransformFlagBitsKHR::Identity,
		presentMode: present_mode, clipped: 1,
		pQueueFamilyIndices: queue_family_indices.as_ptr(), queueFamilyIndexCount: queue_family_indices.len() as u32,
		oldSwapchain: std::ptr::null_mut(), flags: 0, surface: surface.get()
	};

	(vk::Swapchain::create(queue.parent(), &swapchain_info).unwrap(), format.format, sc_extent)
}
fn create_image_views<'d, ImageObj: vk::VkImageResource + HasParent<ParentRefType=&'d vk::Device<'d>>>(images: &'d Vec<ImageObj>, format: VkFormat) -> Vec<vk::ImageView>
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
		let attachments = [v.get()];
		let fb_info = VkFramebufferCreateInfo
		{
			sType: VkStructureType::FramebufferCreateInfo, pNext: std::ptr::null(),
			attachmentCount: attachments.len() as u32, pAttachments: attachments.as_ptr(), renderPass: rp.get(),
			width: width, height: height, layers: 1, flags: 0
		};
		v.parent().create_framebuffer(&fb_info).unwrap()
	}).collect::<Vec<_>>()
}

struct Enemy
{
	block_index: u32, left: f32, appear_time: time::PreciseTime
}
impl Enemy
{
	pub fn new(datastore: &mut logical_resources::EnemyDatastore, memory_ref: &mut structures::InstanceMemory, uniform_memory_ref: &mut structures::UniformMemory,
		init_left: f32) -> Option<Self>
	{
		datastore.allocate_block(memory_ref).map(|index|
		{
			datastore.update_instance_data(uniform_memory_ref, index,
				UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32)).quaternion(), UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32)).quaternion(),
				&Vector4::new(init_left, 0.0f32, 0.0f32, 0.0f32));
			Enemy
			{
				block_index: index, left: init_left, appear_time: time::PreciseTime::now()
			}
		})
	}
	pub fn update(&mut self, datastore: &logical_resources::EnemyDatastore, memory_ref: &mut structures::UniformMemory) -> bool
	{
		let delta_time = self.appear_time.to(time::PreciseTime::now());
		let living_seconds = delta_time.num_milliseconds() as f32 / 1000.0f32;
		let current_y = if living_seconds < 0.875f32
		{
			15.0f32 * (1.0f32 - (1.0f32 - living_seconds / 0.875f32).powi(2)) - 3.0f32
		}
		else
		{
			15.0f32 + (living_seconds - 0.875f32) * 2.5f32 - 3.0f32
		};
		datastore.update_instance_data(memory_ref, self.block_index,
			UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32).normalize() * (260.0f32 * living_seconds).to_radians()).quaternion(),
			UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32).normalize() * (-260.0f32 * living_seconds + 13.0f32).to_radians()).quaternion(),
			&Vector4::new(self.left, current_y, 0.0f32, 0.0f32));

		current_y >= 50.0f32
	}
	pub fn die(&self, datastore: &mut logical_resources::EnemyDatastore, memory_ref: &mut structures::InstanceMemory)
	{
		datastore.free_block(self.block_index, memory_ref);
	}
}
struct Player
{
	start_time: time::PreciseTime
}
impl Player
{
	fn new(uniform_ref: &mut structures::UniformMemory, instance_ref: &mut structures::InstanceMemory) -> Self
	{
		let u_quaternion = UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32));
		let quaternion_ref = u_quaternion.quaternion();

		instance_ref.player_rotq[0] = [quaternion_ref.i, quaternion_ref.j, quaternion_ref.k, quaternion_ref.w];
		instance_ref.player_rotq[1] = [quaternion_ref.i, quaternion_ref.j, quaternion_ref.k, quaternion_ref.w];
		uniform_ref.player_center_tf = [0.0f32, 38.0f32, 0.0f32, 0.0f32];

		Player { start_time: time::PreciseTime::now() }
	}
	fn update(&self, instance_ref: &mut structures::InstanceMemory, uniform_ref: &mut structures::UniformMemory)
	{
		let delta_time = self.start_time.to(time::PreciseTime::now());
		let living_secs = delta_time.num_milliseconds() as f64 / 1000.0f64;
		let u_quaternions = [
			UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32).normalize() * (260.0f32 * living_secs as f32).to_radians()),
			UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32).normalize() * (-260.0f32 * living_secs as f32 + 13.0f32).to_radians())
		];
		let mut quaternions = u_quaternions.iter().map(|x| x.quaternion()).map(|q| [q.i, q.j, q.k, q.w]);

		instance_ref.player_rotq[0] = quaternions.next().unwrap();
		instance_ref.player_rotq[1] = quaternions.next().unwrap();
	}
}

fn main()
{
	log::set_logger(|max_log_level|
	{
		max_log_level.set(log::LogLevelFilter::Info);
		Box::new(EngineLogger)
	});

	// init xcb(connection to display)
	let xcon = XServerConnection::connect();

	utils::memory_management_test();

	// init vulkan
	let instance = create_instance();
	let adapter = vk::PhysicalDevice::wrap(instance.enumerate_adapters().unwrap()[0]);

	// Create Device and Queues //
	let queue_indices = adapter.get_queue_family_indices();
	let mut queue_indices_iter = queue_indices.into_iter().enumerate();
	let gqf_index = queue_indices_iter.by_ref().filter(|&(_, ref x)| (x.queueFlags & VK_QUEUE_GRAPHICS_BIT) != 0)
		.map(|(i, _)| i as u32).next().expect("unable to find queue for graphics on device");
	let tqf_index = queue_indices_iter.by_ref().filter(|&(_, ref x)| (x.queueFlags & VK_QUEUE_TRANSFER_BIT) != 0)
		.map(|(i, _)| i as u32).next().unwrap_or(gqf_index);
	diagnose_adapter(&xcon, &adapter, gqf_index);
	let queue_properties = adapter.enumerate_queue_family_properties();
	println!("=== Device Queue Family Properties ===");
	for (i, qp) in queue_properties.iter().enumerate()
	{
		println!("-- Queue #{}", i);
		println!("---- QueueCount: {}", qp.queueCount);
	}

	let dev_layers = [DEBUG_LAYER_NAME.as_ptr()];
	let dev_extensions = [SWAPCHAIN_EXTENSION_NAME.as_ptr()];
	let features = VkPhysicalDeviceFeatures
	{
		geometryShader: 1,
		.. Default::default()
	};
	let (device, transfer_queue_index) = if gqf_index == tqf_index
	{
		// Use same queue
		println!("-- Using same queue family: {}", gqf_index);
		let q_priorities = [0.0f32, 0.0f32];
		let queue_count = std::cmp::min(2, queue_properties[gqf_index as usize].queueCount);
		let queue_infos = [VkDeviceQueueCreateInfo
		{
			sType: VkStructureType::DeviceQueueCreateInfo, pNext: std::ptr::null(), flags: 0,
			queueCount: queue_count, queueFamilyIndex: gqf_index, pQueuePriorities: q_priorities.as_ptr()
		}];
		let device_info = VkDeviceCreateInfo
		{
			sType: VkStructureType::DeviceCreateInfo, pNext: std::ptr::null(), flags: 0,
			queueCreateInfoCount: queue_infos.len() as u32, pQueueCreateInfos: queue_infos.as_ptr(),
			enabledLayerCount: dev_layers.len() as u32, ppEnabledLayerNames: dev_layers.as_ptr() as *const *const i8,
			enabledExtensionCount: dev_extensions.len() as u32, ppEnabledExtensionNames: dev_extensions.as_ptr() as *const *const i8,
			pEnabledFeatures: &features
		};
		(adapter.create_device(&device_info).unwrap(), queue_count - 1)
	}
	else
	{
		// Use different queue
		println!("-- Using difference queue family: graphics = {} / transfer = {}", gqf_index, tqf_index);
		let q_priorities = [0.0f32];
		let queue_infos = [
			VkDeviceQueueCreateInfo
			{
				sType: VkStructureType::DeviceQueueCreateInfo, pNext: std::ptr::null(), flags: 0,
				queueCount: 1, queueFamilyIndex: gqf_index, pQueuePriorities: q_priorities.as_ptr()
			},
			VkDeviceQueueCreateInfo
			{
				sType: VkStructureType::DeviceQueueCreateInfo, pNext: std::ptr::null(), flags: 0,
				queueCount: 1, queueFamilyIndex: tqf_index, pQueuePriorities: q_priorities.as_ptr()
			}
		];
		let device_info = VkDeviceCreateInfo
		{
			sType: VkStructureType::DeviceCreateInfo, pNext: std::ptr::null(), flags: 0,
			queueCreateInfoCount: queue_infos.len() as u32, pQueueCreateInfos: queue_infos.as_ptr(),
			enabledLayerCount: dev_layers.len() as u32, ppEnabledLayerNames: dev_layers.as_ptr() as *const *const i8,
			enabledExtensionCount: dev_extensions.len() as u32, ppEnabledExtensionNames: dev_extensions.as_ptr() as *const *const i8,
			pEnabledFeatures: &features
		};
		(adapter.create_device(&device_info).unwrap(), 0)
	};
	let graphics_queue = device.get_queue(gqf_index, 0);
	let transfer_queue = device.get_queue(tqf_index, transfer_queue_index);

	// init display
	let window = xcon.new_unresizable_window(VkExtent2D(640, 480), APP_NAME);
	window.map();
	xcon.flush();

	// Device to Device and Resource to Resource Synchronization //
	let render_target_sem = device.create_semaphore().unwrap();
	let transfer_sem = device.create_semaphore().unwrap();
	let fence = device.create_fence().unwrap();

	// Ready for Rendering
	let surface = create_surface(&instance, &window);
	let (swapchain, sc_format, sc_extent) = create_swapchain(&graphics_queue, &surface);
	let render_area = VkRect2D(VkOffset2D(0, 0), sc_extent);
	let final_images = swapchain.get_images().unwrap();
	let final_image_views = create_image_views(&final_images, sc_format);
	let simple_pass = create_simple_render_pass(&device, sc_format);
	let final_framebuffers = create_framebuffers(&final_image_views, &simple_pass, sc_extent);

	// Command Pools //
	let pool = device.create_command_pool(&graphics_queue, true, false).unwrap();
	let transfer_pool = device.create_command_pool(&transfer_queue, false, false).unwrap();
	let initializer_pool = device.create_command_pool(&transfer_queue, false, true).unwrap();

	// Device Resources //
	let memory_preallocator = device_resources::MemoryPreallocator::new(&adapter);
	let memory_bound_resources = device_resources::MemoryBoundResources::new(&device, &memory_preallocator);
	let descriptor_sets = device_resources::DescriptorSets::new(&device);

	// Ready for Shading
	let pp_commons = logical_resources::PipelineCommonStore::new(&device, &descriptor_sets);
	let enemy_render = logical_resources::EnemyRenderer::new(&pp_commons, &simple_pass, sc_extent);
	let background_render = logical_resources::BackgroundRenderer::new(&pp_commons, &simple_pass, sc_extent);
	let player_render = logical_resources::PlayerRenderer::new(&pp_commons, &simple_pass, sc_extent);
	let debug_render = logical_resources::DebugRenderer::new(&pp_commons, &simple_pass, sc_extent);

	// Logical Resources //
	let di_desc = 1;
	let meshstore = logical_resources::Meshstore::new(memory_preallocator.meshstore_base);
	let projection_matrixes = logical_resources::ProjectionMatrixes::new(sc_extent);
	let mut enemy_datastore = logical_resources::EnemyDatastore::new();
	let background_datastore = logical_resources::BackgroundDatastore::new();
	let debug_info_resources = logical_resources::DebugInfoResources::new(&device, &transfer_queue, &initializer_pool, &transfer_pool, di_desc);

	// Setup Descriptors //
	let uniform_buffer_info = VkDescriptorBufferInfo(**memory_bound_resources.buffer, memory_preallocator.uniform_memory_base, memory_preallocator.uniform_memory_size);
	let descriptor_infos = debug_info_resources.write_descriptor_info(&descriptor_sets).iter().chain(&[
		VkWriteDescriptorSet
		{
			sType: VkStructureType::WriteDescriptorSet, pNext: std::ptr::null(),
			dstSet: descriptor_sets.sets[0], dstBinding: 0, dstArrayElement: 0,
			descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			pBufferInfo: &uniform_buffer_info, pImageInfo: std::ptr::null(), pTexelBufferView: std::ptr::null()
		}
	]).map(|x| *x).collect::<Vec<_>>();
	device.update_descriptor_sets(&descriptor_infos, &[]);

	// Memory to Structure Mapping //
	let mapped_range = memory_bound_resources.stage_buffer.map(0 .. memory_preallocator.total_size).unwrap();
	let (uniform_memory_range, instance_memory_range) = (
		mapped_range.map_mut::<structures::UniformMemory>(memory_preallocator.uniform_memory_base),
		mapped_range.map_mut::<structures::InstanceMemory>(memory_preallocator.instance_base)
	);

	// Initial Staging //
	{
		let player_rotq_unit = [UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32)), UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32))];

		meshstore.initial_stage_data(&mapped_range);
		projection_matrixes.initial_stage_data(uniform_memory_range);
		enemy_datastore.initial_stage_data(uniform_memory_range);
		instance_memory_range.enemy_instance_mult = [1; structures::MAX_ENEMY_COUNT];
		instance_memory_range.background_instance_mult = [1; structures::MAX_BACKGROUND_COUNT];
	}

	// Player Instance //
	let player = Player::new(uniform_memory_range, instance_memory_range);

	// Double-buffered object storages //
	let mut olist_index = 0;
	let mut enemy_list = [
		std::collections::LinkedList::<Enemy>::new(),
		std::collections::LinkedList::new()
	];
	let mut enemy_counter = 0;

	println!("-- Background Instance Offs: {}", structures::background_instance_offs());

	// Ready for command recording //
	let final_commands = pool.allocate_primary_buffers(final_framebuffers.len()).unwrap();
	let clear_values = [VkClearValue(VkClearColorValue(0.0f32, 0.0f32, 0.015625f32, 1.0f32))];
	// let clear_values = [VkClearValue(VkClearColorValue(0.0f32, 0.0f32, 0.0f32, 1.0f32))];
	for cb_index in 0 .. final_framebuffers.len()
	{
		let image_barrier = final_images[cb_index].memory_barrier(vk::ImageSubresourceRange::default_color(), VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT)
			.layout(VkImageLayout::PresentSrcKHR, VkImageLayout::ColorAttachmentOptimal);

		final_commands.begin(cb_index).unwrap()
			.resource_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, &[], &[], &[image_barrier])
			.begin_render_pass(&final_framebuffers[cb_index], &simple_pass, render_area, &clear_values, false)
			.bind_vertex_buffers(&[**memory_bound_resources.buffer], &[meshstore.wire_render_offset])
			.bind_descriptor_sets(background_render.layout_ref, &[descriptor_sets.sets[0]], &[])
			.bind_pipeline(&background_render.state)
			.bind_vertex_buffers_partial(1, &[**memory_bound_resources.buffer], &[memory_preallocator.instance_base + structures::background_instance_offs() as VkDeviceSize])
			.draw(4, structures::MAX_BACKGROUND_COUNT as u32, 0)
			.bind_pipeline(&enemy_render.state)
			.bind_vertex_buffers_partial(1, &[**memory_bound_resources.buffer], &[memory_preallocator.instance_base])
			.draw(4, structures::MAX_ENEMY_COUNT as u32, 0)
			.bind_pipeline(&player_render.state)
			.bind_vertex_buffers(&[**memory_bound_resources.buffer, **memory_bound_resources.buffer],
				&[meshstore.wire_render_offset + structures::player_cube_vertex_offs() as VkDeviceSize, memory_preallocator.instance_base + structures::player_instance_offs() as VkDeviceSize])
			.bind_index_buffer(&memory_bound_resources.buffer, meshstore.index_offset)
			.draw_indexed(24, 2, 0)
			.bind_pipeline(&debug_render.state)
			.bind_descriptor_sets(debug_render.layout_ref, &[descriptor_sets.sets[0], descriptor_sets.sets[di_desc as usize]], &[])
			.bind_vertex_buffers(&[**debug_info_resources.buffer], &[0])
			.bind_index_buffer(&debug_info_resources.buffer, debug_info_resources.index_offset)
			.draw_indexed(12, 1, 0)
			.bind_pipeline(&debug_render.state_instanced)
			.bind_vertex_buffers(&[**debug_info_resources.buffer, **debug_info_resources.buffer], &[debug_info_resources.unit_vertices_offset, debug_info_resources.instance_offset])
			.draw_indexed_indirect(&debug_info_resources.buffer, debug_info_resources.indirect_offset)
			.end_render_pass();
	}
	let device_buffer_access_mask = VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT;
	let transfer_commands = transfer_pool.allocate_primary_buffers(1).unwrap();
	{
		let entire_range = memory_preallocator.instance_base .. memory_preallocator.total_size;
		let buffer_barriers = [
			memory_bound_resources.stage_buffer.memory_barrier(entire_range.clone(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_TRANSFER_READ_BIT),
			memory_bound_resources.buffer.memory_barrier(entire_range.clone(), device_buffer_access_mask, VK_ACCESS_TRANSFER_WRITE_BIT)
		];
		let buffer_barriers_ret = [
			memory_bound_resources.stage_buffer.memory_barrier(entire_range.clone(), VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_HOST_WRITE_BIT),
			memory_bound_resources.buffer.memory_barrier(entire_range, VK_ACCESS_TRANSFER_WRITE_BIT, device_buffer_access_mask)
		];
		let copy_regions = [VkBufferCopy(memory_preallocator.instance_base, memory_preallocator.instance_base, memory_preallocator.total_size - memory_preallocator.instance_base)];
		transfer_commands.begin(0).unwrap()
			.resource_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, &[], &buffer_barriers, &[])
			.copy_buffer(&memory_bound_resources.stage_buffer, &memory_bound_resources.buffer, &copy_regions)
			.resource_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_ALL_COMMANDS_BIT, &[], &buffer_barriers_ret, &[]);
	}
	// Initial execution of setup layouts and transfer stagings
	{
		let tcb = initializer_pool.allocate_primary_buffers(1).unwrap();
		let entire_range = 0 .. memory_preallocator.total_size;

		let image_barriers = final_images.iter().map(|o|
		{
			o.memory_barrier(vk::ImageSubresourceRange::default_color(), 0, VK_ACCESS_MEMORY_READ_BIT).layout(VkImageLayout::Undefined, VkImageLayout::PresentSrcKHR)
		}).collect::<Vec<_>>();
		let buffer_barriers = [
			memory_bound_resources.stage_buffer.memory_barrier(entire_range.clone(), 0, VK_ACCESS_TRANSFER_READ_BIT),
			memory_bound_resources.buffer.memory_barrier(entire_range.clone(), 0, VK_ACCESS_TRANSFER_WRITE_BIT)
		];
		let buffer_barriers_to_use = [
			memory_bound_resources.stage_buffer.memory_barrier(entire_range.clone(), VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_HOST_WRITE_BIT),
			memory_bound_resources.buffer.memory_barrier(entire_range.clone(), VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_SHADER_READ_BIT)
		];

		let copy_regions = [VkBufferCopy(0, 0, memory_preallocator.total_size)];
		tcb.begin(0).unwrap()
			.resource_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, &[], &buffer_barriers, image_barriers.as_slice())
			.copy_buffer(&memory_bound_resources.stage_buffer, &memory_bound_resources.buffer, &copy_regions)
			.resource_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, &[], &buffer_barriers_to_use, &[]);
		transfer_queue.submit_commands(&[tcb[0]], &[], &[], None).unwrap();
		transfer_queue.wait_for_idle().unwrap();
	}

	// initial execution(coordinated execution order by semaphore)
	let mut index_render_to = swapchain.acquire_next_image(&render_target_sem).unwrap();
	graphics_queue.submit_commands(&[final_commands[index_render_to as usize]], &[render_target_sem.get()], &[], Some(&fence)).unwrap();
	let mut render_start_time = time::PreciseTime::now();

	// randomizers //
	let mut randomizer = rand::thread_rng();
	let mut left_range = rand::distributions::Range::new(-25.0f32, 25.0f32);
	let mut appear_percent_range = rand::distributions::Range::new(0, 40);

	// Application Loop
	let mut prev_frame_time = time::PreciseTime::now();
	let dt_mapped_range = debug_info_resources.map_for_instance();
	let mut require_transfer = false;
	while xcon.process_messages()
	{
		// Present -> Render //
		if fence.get_status().is_ok()
		{
			let delta_time = render_start_time.to(time::PreciseTime::now());
			debug_info_resources.update_text_data(&dt_mapped_range, delta_time.num_microseconds().unwrap() as f32 / 1000.0f32, enemy_counter);
			fence.reset().unwrap();
			swapchain.present(&graphics_queue, index_render_to, &[]).unwrap();
			index_render_to = swapchain.acquire_next_image(&render_target_sem).unwrap();
			let mut wait_semaphores = vec![*render_target_sem];
			// if require_transfer
			// {
				transfer_queue.submit_commands(&[transfer_commands[0], debug_info_resources.transfer_commands[0]], &[], &[*transfer_sem], None).unwrap();
				wait_semaphores.push(*transfer_sem);
			// }
			graphics_queue.submit_commands(&[final_commands[index_render_to as usize]], &wait_semaphores, &[], Some(&fence)).unwrap();
			render_start_time = time::PreciseTime::now();
			require_transfer = false;
			background_datastore.update(uniform_memory_range, instance_memory_range, &mut randomizer, delta_time);
			player.update(instance_memory_range, uniform_memory_range);
		}

		if prev_frame_time.to(time::PreciseTime::now()) >= time::Duration::milliseconds(8)
		{
			if appear_percent_range.sample(&mut randomizer) == 0
			{
				if let Some(enemy) = Enemy::new(&mut enemy_datastore, instance_memory_range, uniform_memory_range, left_range.sample(&mut randomizer))
				{
					enemy_list[olist_index].push_back(enemy);
					enemy_counter += 1;
				}
				else { println!("Warning: Unable to allocate memory block for enemy"); }
			}
			let next_index = if olist_index == 0 { 1 } else { 0 };
			while let Some(mut e) = enemy_list[olist_index].pop_front()
			{
				if !e.update(&enemy_datastore, uniform_memory_range)
				{
					enemy_list[next_index].push_front(e);
				}
				else
				{
					e.die(&mut enemy_datastore, instance_memory_range);
					enemy_counter -= 1;
				}
			}
			olist_index = next_index;
			require_transfer = true;
			prev_frame_time = time::PreciseTime::now();
		}
	}
	device.wait_for_idle().unwrap();
}
