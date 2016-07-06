extern crate libc;
extern crate xcb;
extern crate nalgebra;
#[macro_use] mod vkffi;
mod render_vk;

mod traits;
use traits::*;
mod xcbw;
use xcbw::*;
mod vertex_formats;
use vertex_formats::*;
mod meshstore;
use meshstore::MeshStore;
mod projection_matrixes;
use projection_matrixes::ProjectionMatrixes;

use vkffi::*;
use render_vk::wrap as vk;
use render_vk::wrap::CreationObject;
use nalgebra::*;

const APP_NAME: &'static str = "HardGrad -> Extent\0";
const ENGINE_NAME: &'static str = "Hybrid-ML\0";
const DEBUG_LAYER_NAME: &'static str = "VK_LAYER_LUNARG_standard_validation\0";
const SURFACE_EXTENSION_NAME: &'static str = "VK_KHR_surface\0";
const PSURFACE_EXTENSION_NAME: &'static str = "VK_KHR_xcb_surface\0";
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
fn create_image_views<'d, ImageObj: vk::VkImageResource + HasParent<ParentRefType=&'d vk::Device>>(images: &'d Vec<ImageObj>, format: VkFormat) -> Vec<vk::ImageView>
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

fn ShaderSpecializationEntry<T: std::marker::Sized>(id: u32, index_offset: u32) -> VkSpecializationMapEntry
{
	VkSpecializationMapEntry(id, std::mem::size_of::<T>() as u32 * index_offset, std::mem::size_of::<T>())
}

fn main()
{
	// init xcb(connection to display)
	let xcon = XServerConnection::connect();

	// init vulkan
	let instance = create_instance();
	let adapter = vk::PhysicalDevice::wrap(instance.enumerate_adapters().unwrap()[0]);
	let qf = adapter.get_graphics_queue_family_index().unwrap();
	if !xcon.is_vk_presentation_support(&adapter, qf) { panic!("Unsupported Display Format"); }
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
	let VkExtent2D(sc_width, sc_height) = sc_extent;
	let render_area = VkRect2D(VkOffset2D(0, 0), sc_extent);
	let final_images = swapchain.get_images().unwrap();
	let final_image_views = create_image_views(&final_images, sc_format);
	let simple_pass = create_simple_render_pass(&device, sc_format);
	let final_framebuffers = create_framebuffers(&final_image_views, &simple_pass, sc_extent);

	// Uniform Descriptors //
	let descriptor_pool = device.create_descriptor_pool(1, &[VkDescriptorPoolSize(VkDescriptorType::UniformBuffer, 1)]).unwrap();
	let dsl_bindings =
	[
		VkDescriptorSetLayoutBinding
		{
			binding: 0, descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			stageFlags: VK_SHADER_STAGE_VERTEX_BIT, pImmutableSamplers: std::ptr::null()
		}
	];
	let dsl_info = VkDescriptorSetLayoutCreateInfo
	{
		sType: VkStructureType::DescriptorSetLayoutCreateInfo, pNext: std::ptr::null(), flags: 0,
		bindingCount: dsl_bindings.len() as u32, pBindings: dsl_bindings.as_ptr()
	};
	let layout_for_projection = device.create_descriptor_set_layout(&dsl_info).unwrap();

	// Ready for Shading
	let vshader = device.create_shader_module_from_file("shaders/RawOutput.spv").unwrap();
	let pshader = device.create_shader_module_from_file("shaders/ThroughColor.spv").unwrap();
	let layout = device.create_pipeline_layout(&[layout_for_projection.get()], &[]).unwrap();
	let cache = device.create_empty_pipeline_cache().unwrap();
	let shader_entry = std::ffi::CString::new("main").unwrap();
	let vertex_bindings =
	[
		VkVertexInputBindingDescription(0, std::mem::size_of::<Position>() as u32, VkVertexInputRate::Vertex)
	];
	let vertex_inputs =
	[
		VkVertexInputAttributeDescription(0, 0, VkFormat::R32G32B32A32_SFLOAT, 0)
	];
	let viewports = [VkViewport(0.0f32, 0.0f32, sc_width as f32, sc_height as f32, -1.0f32, 1.0f32)];
	let scissors = [render_area];
	let shader_specialization_map_entries =
	[
		ShaderSpecializationEntry::<f32>(10, 0),
		ShaderSpecializationEntry::<f32>(11, 1),
		ShaderSpecializationEntry::<f32>(12, 2),
		ShaderSpecializationEntry::<f32>(13, 3)
	];
	let shader_specialization_data = [0.25f32, 0.9875f32, 1.5f32, 1.0f32];
	let shader_const_specialization = VkSpecializationInfo
	{
		mapEntryCount: shader_specialization_map_entries.len() as u32, pMapEntries: shader_specialization_map_entries.as_ptr(),
		dataSize: std::mem::size_of::<[f32; 4]>(), pData: unsafe { std::mem::transmute(shader_specialization_data.as_ptr()) }
	};
	let shader_stages =
	[
		VkPipelineShaderStageCreateInfo
		{
			sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
			stage: VK_SHADER_STAGE_VERTEX_BIT, module: vshader.get(), pName: shader_entry.as_ptr(),
			pSpecializationInfo: &shader_const_specialization
		}, VkPipelineShaderStageCreateInfo
		{
			sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
			stage: VK_SHADER_STAGE_FRAGMENT_BIT, module: pshader.get(), pName: shader_entry.as_ptr(),
			pSpecializationInfo: std::ptr::null()
		}
	];
	let vertex_input_state = VkPipelineVertexInputStateCreateInfo
	{
		sType: VkStructureType::Pipeline_VertexInputStateCreateInfo, pNext: std::ptr::null(), flags: 0,
		vertexBindingDescriptionCount: vertex_bindings.len() as u32, pVertexBindingDescriptions: vertex_bindings.as_ptr(),
		vertexAttributeDescriptionCount: vertex_inputs.len() as u32, pVertexAttributeDescriptions: vertex_inputs.as_ptr()
	};
	let input_assembly_state = VkPipelineInputAssemblyStateCreateInfo
	{
		sType: VkStructureType::Pipeline_InputAssemblyStateCreateInfo, pNext: std::ptr::null(), flags: 0,
		topology: VkPrimitiveTopology::LineList, primitiveRestartEnable: false as VkBool32
	};
	let viewport_state = VkPipelineViewportStateCreateInfo
	{
		sType: VkStructureType::Pipeline_ViewportStateCreateInfo, pNext: std::ptr::null(), flags: 0,
		viewportCount: viewports.len() as u32, pViewports: viewports.as_ptr(),
		scissorCount: scissors.len() as u32, pScissors: scissors.as_ptr()
	};
	let rasterization_state = VkPipelineRasterizationStateCreateInfo
	{
		sType: VkStructureType::Pipeline_RasterizationStateCreateInfo, pNext: std::ptr::null(), flags: 0,
		polygonMode: VkPolygonMode::Fill, cullMode: VK_CULL_MODE_NONE, frontFace: VkFrontFace::CounterClockwise,
		rasterizerDiscardEnable: false as VkBool32, depthClampEnable: false as VkBool32, depthBiasEnable: false as VkBool32,
		lineWidth: 1.0f32, depthBiasConstantFactor: 0.0f32, depthBiasClamp: 0.0f32, depthBiasSlopeFactor: 0.0f32
	};
	let multisample_state = VkPipelineMultisampleStateCreateInfo
	{
		sType: VkStructureType::Pipeline_MultisampleStateCreateInfo, pNext: std::ptr::null(), flags: 0,
		rasterizationSamples: VK_SAMPLE_COUNT_1_BIT, sampleShadingEnable: false as VkBool32,
		alphaToCoverageEnable: false as VkBool32, alphaToOneEnable: false as VkBool32,
		pSampleMask: std::ptr::null(), minSampleShading: 0.0f32
	};
	let attachment_blend_states =
	[
		VkPipelineColorBlendAttachmentState
		{
			blendEnable: false as VkBool32,
			colorWriteMask: VK_COLOR_COMPONENT_A_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_R_BIT,
			srcColorBlendFactor: VkBlendFactor::One, dstColorBlendFactor: VkBlendFactor::One, colorBlendOp: VkBlendOp::Add,
			srcAlphaBlendFactor: VkBlendFactor::One, dstAlphaBlendFactor: VkBlendFactor::One, alphaBlendOp: VkBlendOp::Add
		}
	];
	let blend_state = VkPipelineColorBlendStateCreateInfo
	{
		sType: VkStructureType::Pipeline_ColorBlendStateCreateInfo, pNext: std::ptr::null(), flags: 0,
		logicOpEnable: false as VkBool32, logicOp: VkLogicOp::NOP, blendConstants: [0.0f32; 4],
		attachmentCount: attachment_blend_states.len() as u32, pAttachments: attachment_blend_states.as_ptr()
	};
	let pipeline_info = VkGraphicsPipelineCreateInfo
	{
		sType: VkStructureType::GraphicsPipelineCreateInfo, pNext: std::ptr::null(), flags: 0,
		stageCount: shader_stages.len() as u32, pStages: shader_stages.as_ptr(),
		pVertexInputState: &vertex_input_state,
		pInputAssemblyState: &input_assembly_state,
		pTessellationState: std::ptr::null(),
		pDepthStencilState: std::ptr::null(),
		pViewportState: &viewport_state,
		pRasterizationState: &rasterization_state,
		pMultisampleState: &multisample_state,
		pColorBlendState: &blend_state,
		pDynamicState: std::ptr::null(),
		layout: layout.get(), renderPass: simple_pass.get(), subpass: 0,
		basePipelineHandle: std::ptr::null_mut(), basePipelineIndex: 0
	};
	let pipeline = device.create_graphics_pipelines(&cache, &[pipeline_info]).unwrap().into_iter().next().unwrap();

	// Rendering Resources //
	let meshstore = MeshStore::new(&adapter, &device);

	// Projection Matrixes //
	let projection_matrixes = ProjectionMatrixes::new(&adapter, &device, &descriptor_pool, &layout_for_projection, sc_extent);

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

		final_commands.begin(cb_index).unwrap()
			.resource_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, &[], &[], &[image_barrier])
			.begin_render_pass(&final_framebuffers[cb_index], &simple_pass, render_area, &clear_values, false)
			.bind_pipeline(&pipeline)
			.bind_descriptor_sets(&layout, &[projection_matrixes.uniform_desc_set[0]], &[])
			.bind_vertex_buffers(&[meshstore.buffer.get()], &[meshstore.unit_cube_vertices_offset])
			.bind_index_buffer(&meshstore.buffer, meshstore.unit_cube_indices_offset)
			.draw_indexed(24, 2)
			.end_render_pass();
	}
	// Initial execution of setup layouts
	{
		let cb = pool.allocate_primary_buffers(1).unwrap();
		let mut image_barriers: Vec<VkImageMemoryBarrier> = vec![unsafe { std::mem::uninitialized() }; final_images.len()];
		for i in 0 .. final_images.len()
		{
			image_barriers[i] = VkImageMemoryBarrier
			{
				sType: VkStructureType::ImageMemoryBarrier, pNext: std::ptr::null(),
				image: final_images[i].get(), oldLayout: VkImageLayout::Undefined, newLayout: VkImageLayout::PresentSrcKHR,
				srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
				srcAccessMask: 0, dstAccessMask: VK_ACCESS_MEMORY_READ_BIT,
				subresourceRange: VkImageSubresourceRange
				{
					aspectMask: VK_IMAGE_ASPECT_COLOR_BIT, baseMipLevel: 0, baseArrayLayer: 0,
					levelCount: 1, layerCount: 1
				}
			};
		}
		cb.begin(0).unwrap().resource_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
			&[], &[], image_barriers.as_slice());
		device.submit_commands(&[cb[0]], &[], None).unwrap();
		device.wait_queue_for_idle().unwrap();
	}

	// Application Loop
	'app_loop: loop
	{
		'event_loop: loop
		{
			match xcon.poll_event()
			{
				Some(ev) =>
				{
					match unsafe { (*ev.ptr).response_type & 0x7f }
					{
						xcb::ffi::xproto::XCB_CLIENT_MESSAGE =>
						{
							let event_ptr = unsafe { std::mem::transmute::<_, *mut xcb::ffi::xproto::xcb_client_message_event_t>(ev.ptr) };
							if xcon.is_delete_window_message(event_ptr) { break 'app_loop; }
						},
						_ => println!("xcb event response: {}", unsafe { (*ev.ptr).response_type })
					}
				},
				None => break 'event_loop
			}
		}

		// Render //
		// coordinated execution order by semaphore
		let index_render_to = swapchain.acquire_next_image(&semaphore).unwrap();
		device.submit_commands(&[final_commands[index_render_to as usize]], &[semaphore.get()], Some(&fence)).unwrap();
		fence.wait().unwrap(); fence.reset().unwrap();
		swapchain.present(index_render_to, &[]).unwrap();
	}
}
