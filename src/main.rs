extern crate libc;
extern crate xcb;
extern crate nalgebra;
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
mod logical_resources;
use nalgebra::*;

use vkffi::*;
use render_vk::wrap as vk;
use render_vk::traits::*;

impl std::default::Default for VkApplicationInfo
{
	fn default() -> Self
	{
		VkApplicationInfo
		{
			sType: VkStructureType::ApplicationInfo, pNext: std::ptr::null(),
			apiVersion: VK_API_VERSION_1_0,
			pApplicationName: std::ptr::null(), pEngineName: std::ptr::null(),
			applicationVersion: 0, engineVersion: 0
		}
	}
}
impl std::default::Default for VkInstanceCreateInfo
{
	fn default() -> Self
	{
		VkInstanceCreateInfo
		{
			sType: VkStructureType::InstanceCreateInfo, pNext: std::ptr::null(), flags: 0,
			pApplicationInfo: std::ptr::null(),
			enabledLayerCount: 0, enabledExtensionCount: 0,
			ppEnabledLayerNames: std::ptr::null(), ppEnabledExtensionNames: std::ptr::null()
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

	vk::Instance::create(&instance_info).expect("Unable to create instance")
}
fn diagnose_adapter(server_con: &XServerConnection, adapter: &vk::PhysicalDevice, queue_index: u32)
{
	// Feature Check //
	let features = adapter.get_features();
	if features.depthClamp == false as VkBool32 { panic!("DepthClamp Feature is required in device"); }

	// Vulkan and XCB Integration Check //
	if !server_con.is_vk_presentation_support(adapter, queue_index) { panic!("Unsupported Display Format"); }
}
fn create_graphics_device<'a>(adapter_ref: &'a vk::PhysicalDevice) -> vk::Device<'a>
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
fn create_surface<'i, 'c>(instance_ref: &'i vk::Instance, window: &XWindow<'c>) -> vk::Surface<'i>
{
	let xcb_surface_info = VkXcbSurfaceCreateInfoKHR
	{
		sType: VkStructureType::XcbSurfaceCreateInfoKHR, pNext: std::ptr::null(), flags: 0,
		connection: window.parent().get_raw(), window: window.get()
	};
	vk::Surface::create(instance_ref, &xcb_surface_info).unwrap()
}
fn create_swapchain<'d>(adapter: &vk::PhysicalDevice, device_ref: &'d vk::Device, surface: &vk::Surface) -> (vk::Swapchain<'d>, VkFormat, VkExtent2D)
{
	// capabilities check //
	if !adapter.is_surface_support(device_ref.queue_family_index, surface) { panic!("Unsupported Surface"); }
	let surface_caps = adapter.get_surface_capabilities(surface);

	// making desired parameters //
	let format = adapter.enumerate_surface_formats(surface).into_iter()
		.filter(|ref x| x.format == VkFormat::B8G8R8A8_SRGB || x.format == VkFormat::R8G8B8A8_SRGB)
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
		oldSwapchain: std::ptr::null_mut(), flags: 0, surface: surface.get()
	};

	(vk::Swapchain::create(device_ref, &swapchain_info).unwrap(), format.format, sc_extent)
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

// barrier buffer memory without transitioning ownership
fn buffer_memory_barrier(buffer: &vk::Buffer, view_range: std::ops::Range<VkDeviceSize>, src_access_mask: VkAccessFlags, dst_access_mask: VkAccessFlags) -> VkBufferMemoryBarrier
{
	VkBufferMemoryBarrier
	{
		sType: VkStructureType::BufferMemoryBarrier, pNext: std::ptr::null(),
		srcAccessMask: src_access_mask, dstAccessMask: dst_access_mask,
		srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
		buffer: buffer.get(), offset: view_range.start, size: view_range.end - view_range.start
	}
}

fn main()
{
	// init xcb(connection to display)
	let xcon = XServerConnection::connect();

	// init vulkan
	let instance = create_instance();
	let adapter = vk::PhysicalDevice::wrap(instance.enumerate_adapters().unwrap()[0]);
	let qf = adapter.get_graphics_queue_family_index().unwrap();
	diagnose_adapter(&xcon, &adapter, qf);
	let device = create_graphics_device(&adapter);

	// init display
	let window = xcon.new_unresizable_window(VkExtent2D(640, 480), APP_NAME);
	window.map();
	xcon.flush();

	// Device to Device and Resource to Resource Synchronization //
	let semaphore = device.create_semaphore().unwrap();
	let fence = device.create_fence().unwrap();

	// Ready for Rendering
	let surface = create_surface(&instance, &window);
	let (swapchain, sc_format, sc_extent) = create_swapchain(&adapter, &device, &surface);
	let render_area = VkRect2D(VkOffset2D(0, 0), sc_extent);
	let final_images = swapchain.get_images().unwrap();
	let final_image_views = create_image_views(&final_images, sc_format);
	let simple_pass = create_simple_render_pass(&device, sc_format);
	let final_framebuffers = create_framebuffers(&final_image_views, &simple_pass, sc_extent);

	// Device Resources //
	let memory_preallocator = device_resources::MemoryPreallocator::new(&adapter);
	let memory_bound_resources = device_resources::MemoryBoundResources::new(&device, &memory_preallocator);
	let descriptor_sets = device_resources::DescriptorSets::new(&device);

	// Ready for Shading
	let pp_commons = logical_resources::PipelineCommonStore::new(&device, &descriptor_sets);
	let enemy_render = logical_resources::EnemyRenderer::new(&pp_commons, &simple_pass, sc_extent);

	// Logical Resources //
	let meshstore = logical_resources::Meshstore::new(memory_preallocator.meshstore_range.start);
	let projection_matrixes = logical_resources::ProjectionMatrixes::new(&memory_bound_resources.buffer, memory_preallocator.projection_matrixes_range.start, 0, sc_extent);
	let enemy_datastore = logical_resources::EnemyDatastore::new(&memory_bound_resources.buffer, memory_preallocator.enemy_datastore_range.start, 1);

	// Setup Descriptors //
	device.update_descriptor_sets(&[
		projection_matrixes.write_descriptor_info(&descriptor_sets), enemy_datastore.write_descriptor_info(&descriptor_sets)
	], &[]);

	// Initial Staging //
	{
		let mapped_range = memory_bound_resources.staging_memory.map(0 .. memory_preallocator.total_size).unwrap();
		meshstore.initial_stage_data(&mapped_range);
		projection_matrixes.initial_stage_data(&mapped_range);
		enemy_datastore.initial_stage_data(&mapped_range);
		enemy_datastore.update_instance_data(&mapped_range, 1,
			UnitQuaternion::new(Vector3::new(1.0f32, 0.0f32, 1.0f32)).quaternion(), UnitQuaternion::new(Vector3::new(-1.0f32, 0.5f32, -0.75f32)).quaternion(),
			&Vector4::new(0.0f32, 5.0f32, 0.0f32, 0.0f32));
		enemy_datastore.update_instance_data(&mapped_range, 2,
			UnitQuaternion::new(Vector3::new(1.0f32, 0.0f32, 1.0f32) * 2.0f32).quaternion(), UnitQuaternion::new(Vector3::new(-1.0f32, 0.5f32, -0.75f32)).quaternion(),
			&Vector4::new(10.0f32, 15.0f32, 0.0f32, 0.0f32));
		
		// debugging tweak
		mapped_range.range_mut::<u32>(enemy_datastore.character_indices_offset, 2)[0] = 1;
		mapped_range.range_mut::<u32>(enemy_datastore.character_indices_offset, 2)[1] = 2;
	}

	// Ready for command recording //
	let pool = device.create_command_pool(true).unwrap();
	let final_commands = pool.allocate_primary_buffers(final_framebuffers.len()).unwrap();
	let clear_values = [VkClearValue(VkClearColorValue(0.0f32, 0.0f32, 0.015625f32, 1.0f32))];
	// let clear_values = [VkClearValue(VkClearColorValue(0.0f32, 0.0f32, 0.0f32, 1.0f32))];
	for cb_index in 0 .. final_framebuffers.len()
	{
		let image_barrier = VkImageMemoryBarrier
		{
			sType: VkStructureType::ImageMemoryBarrier, pNext: std::ptr::null(),
			image: final_images[cb_index].get(), subresourceRange: VkImageSubresourceRange
			{
				aspectMask: VK_IMAGE_ASPECT_COLOR_BIT, baseMipLevel: 0, baseArrayLayer: 0,
				levelCount: 1, layerCount: 1
			},
			oldLayout: VkImageLayout::PresentSrcKHR, newLayout: VkImageLayout::ColorAttachmentOptimal,
			srcAccessMask: VK_ACCESS_MEMORY_READ_BIT, dstAccessMask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
			srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
		};

		let vertex_buffers = [memory_bound_resources.buffer.get(), memory_bound_resources.buffer.get()];
		final_commands.begin(cb_index).unwrap()
			.resource_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, &[], &[], &[image_barrier])
			.begin_render_pass(&final_framebuffers[cb_index], &simple_pass, render_area, &clear_values, false)
			.bind_pipeline(&enemy_render.state)
			.bind_descriptor_sets(enemy_render.layout_ref, &[descriptor_sets.sets[0], descriptor_sets.sets[1]], &[])
			.bind_vertex_buffers(&vertex_buffers, &[meshstore.unit_cube_vertices_offset, enemy_datastore.character_indices_offset])
			.bind_index_buffer(&memory_bound_resources.buffer, meshstore.unit_cube_indices_offset)
			.push_constants(enemy_render.layout_ref, VK_SHADER_STAGE_VERTEX_BIT, 0, &[0u32])
			.draw_indexed(24, MAX_ENEMY_COUNT as u32)
			.push_constants(enemy_render.layout_ref, VK_SHADER_STAGE_VERTEX_BIT, 0, &[1u32])
			.draw_indexed(24, MAX_ENEMY_COUNT as u32)
			.end_render_pass();
	}
	// Initial execution of setup layouts and transfer stagings
	{
		let cb = pool.allocate_primary_buffers(1).unwrap();
		let image_barriers = final_images.iter().map(|o| VkImageMemoryBarrier
		{
			sType: VkStructureType::ImageMemoryBarrier, pNext: std::ptr::null(),
			image: o.get(), oldLayout: VkImageLayout::Undefined, newLayout: VkImageLayout::PresentSrcKHR,
			srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
			srcAccessMask: 0, dstAccessMask: VK_ACCESS_MEMORY_READ_BIT,
			subresourceRange: VkImageSubresourceRange
			{
				aspectMask: VK_IMAGE_ASPECT_COLOR_BIT, baseMipLevel: 0, baseArrayLayer: 0,
				levelCount: 1, layerCount: 1
			}
		}).collect::<Vec<_>>();
		let entire_range = 0 .. memory_preallocator.total_size;
		let buffer_barriers = [
			buffer_memory_barrier(&memory_bound_resources.stage_buffer, entire_range.clone(), 0, VK_ACCESS_TRANSFER_READ_BIT),
			buffer_memory_barrier(&memory_bound_resources.buffer, entire_range.clone(), 0, VK_ACCESS_TRANSFER_WRITE_BIT)
		];
		let buffer_barriers_to_use = [
			buffer_memory_barrier(&memory_bound_resources.buffer, entire_range.clone(), VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_SHADER_READ_BIT),
			buffer_memory_barrier(&memory_bound_resources.stage_buffer, entire_range.clone(), VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_HOST_WRITE_BIT)
		];
		let copy_regions = [VkBufferCopy(0, 0, memory_preallocator.total_size)];
		cb.begin(0).unwrap()
			.resource_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, &[], &buffer_barriers, image_barriers.as_slice())
			.copy_buffer(&memory_bound_resources.stage_buffer, &memory_bound_resources.buffer, &copy_regions)
			.resource_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, &[], &buffer_barriers_to_use, &[]);
		device.submit_commands(&[cb[0]], &[], None).unwrap();
		device.wait_queue_for_idle().unwrap();
	}

	// initial execution(coordinated execution order by semaphore)
	let mut index_render_to = swapchain.acquire_next_image(&semaphore).unwrap();
	device.submit_commands(&[final_commands[index_render_to as usize]], &[semaphore.get()], Some(&fence)).unwrap();

	// Application Loop
	while xcon.process_messages()
	{
		// Present -> Render //
		if fence.get_status().is_ok()
		{
			fence.reset().unwrap();
			swapchain.present(index_render_to, &[]).unwrap();
			index_render_to = swapchain.acquire_next_image(&semaphore).unwrap();
			device.submit_commands(&[final_commands[index_render_to as usize]], &[semaphore.get()], Some(&fence)).unwrap();
		}
	}
	device.wait_for_idle().unwrap();
}
