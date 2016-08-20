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
mod prelude;

mod constants;
use constants::*;
mod traits;
use traits::*;
mod vertex_formats;
// mod device_resources;
mod structures;
// mod logical_resources;
mod utils;
use nalgebra::*;
use rand::distributions::*;

use vkffi::*; use ansi_term::*;
use render_vk::wrap as vk;
use render_vk::traits::*;

use std::string::String;
use std::collections::LinkedList;
use std::rc::Rc;

use prelude::traits::*;

/*
// Application Dependent Factories
impl std::ops::Deref for Swapchain
{
    type Target = vk::Swapchain;
    fn deref(&self) -> &Self::Target { &self.object }
}

fn create_image_views<'d, ImageObj: vk::VkImageResource + HasParent<ParentRefType=Rc<vk::Device>>>(images: &'d Vec<ImageObj>, format: VkFormat) -> Vec<vk::ImageView>
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
*/
fn main()
{
	if let Err(e) = app_main()
	{
		prelude::crash(e);
	}
}
fn app_main() -> Result<(), prelude::EngineError>
{
	utils::memory_management_test();

	let engine = try!(prelude::Engine::new("HardGrad->Extent", VK_MAKE_VERSION!(0, 0, 1))).with_assets_in(std::env::current_dir().unwrap());
	let main_frame = try!(engine.create_render_window(VkExtent2D(640, 480), "HardGrad -> Extent"));
	let VkExtent2D(frame_width, frame_height) = main_frame.get_extent();
	let execute_next_signal = try!(engine.create_fence());

	let rp_attachment_descs =
	[
		prelude::AttachmentDesc
		{
			format: main_frame.get_format(), clear_on_load: Some(true),
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::PresentSrcKHR,
			.. Default::default()
		}
	];
	let render_passes = [prelude::PassDesc::single_fragment_output(0)];
	let rp_framebuffer_form = try!(engine.create_render_pass(&rp_attachment_descs, &render_passes, &[]));
	let framebuffers = try!(main_frame.get_back_images().iter()
		.map(|x| engine.create_framebuffer(&rp_framebuffer_form, &[&x.view], VkExtent3D(frame_width, frame_height, 1)))
		.collect::<Result<Vec<_>, _>>());

	// Resources //
	let application_data_prealloc = engine.preallocate(&[
		(std::mem::size_of::<structures::VertexMemoryForWireRender>(), prelude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::IndexMemory>(), prelude::BufferDataType::Index),
		(std::mem::size_of::<structures::UniformMemory>(), prelude::BufferDataType::Uniform)
	]);
	let application_data = try!(engine.create_double_buffer(&application_data_prealloc));

	// Descriptor Set //
	let dslayout_u1 = try!(engine.create_descriptor_set_layout(&[
		prelude::Descriptor::Uniform(1, vec![prelude::ShaderStage::Vertex, prelude::ShaderStage::Geometry])
	]));
	let all_descriptor_sets = try!(engine.preallocate_all_descriptor_sets(&[&dslayout_u1]));
	let ref uniform_memory_descriptor_set = all_descriptor_sets[0];
	
	// Shading Structures //
	let raw_output_vert = try!(engine.create_vertex_shader_from_asset("shaders.RawOutput", "main", &[
		prelude::VertexBinding::PerVertex(std::mem::size_of::<vertex_formats::Position>() as u32),
		prelude::VertexBinding::PerInstance(std::mem::size_of::<u32>() as u32)
	], &[prelude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0), prelude::VertexAttribute(1, VkFormat::R32_UINT, 0)]));
	let backline_duplicator = try!(engine.create_geometry_shader_from_asset("shaders.BackLineDuplicator", "main"));
	let through_color_frag = try!(engine.create_fragment_shader_from_asset("shaders.ThroughColor", "main"));

	let swapchain_viewport = VkViewport(0.0f32, 0.0f32, frame_width as f32, frame_height as f32, 0.0f32, 1.0f32);
	let background_render_layout = try!(engine.create_pipeline_layout(&[&dslayout_u1], &[prelude::PushConstantDesc(VK_SHADER_STAGE_GEOMETRY_BIT, 0 .. 4)]));
	let background_render_state = prelude::GraphicsPipelineBuilder::new(&background_render_layout, &rp_framebuffer_form, 0)
		.vertex_shader(&raw_output_vert).geometry_shader(&backline_duplicator).fragment_shader(&through_color_frag)
		.primitive_topology(prelude::PrimitiveTopology::LineStrip(true))
		.viewport_scissors(&[prelude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
		.blend_state(&[prelude::AttachmentBlendState::PremultipliedAlphaBlend]);
	let pipeline_states = try!(engine.create_graphics_pipelines(&[&background_render_state]));
	let ref background_render_state = pipeline_states[0];
	
	// Initial Layouting for Swapchain Backbuffer Images //
	try!(engine.allocate_transient_transfer_command_buffers(1).and_then(|setup_commands|
	{
		let image_memory_barriers = main_frame.get_back_images().iter()
			.map(|x| prelude::ImageMemoryBarrier::hold_ownership(*x, prelude::ImageSubresourceRange::base_color(),
			0, VK_ACCESS_MEMORY_READ_BIT, VkImageLayout::Undefined, VkImageLayout::PresentSrcKHR)).collect::<Vec<_>>();
		try!(setup_commands.begin(0)).pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, false,
			&[], &[], &image_memory_barriers);
		setup_commands.execute()
	}));

	// Rendering Commands //
	let framebuffer_commands = try!(engine.allocate_graphics_command_buffers(main_frame.get_back_images().len() as u32));
	try!(framebuffer_commands.begin_all().and_then(|iter| iter.map(|(i, recorder)|
	{
		let color_output_barrier = prelude::ImageMemoryBarrier::hold_ownership(
			main_frame.get_back_images()[i], prelude::ImageSubresourceRange::base_color(),
			VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
			VkImageLayout::PresentSrcKHR, VkImageLayout::ColorAttachmentOptimal);
		
		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, false, &[], &[],
				&[color_output_barrier])
			.begin_render_pass(&framebuffers[i], &[prelude::AttachmentClearValue::Color(1.0f32, 1.0f32, 1.0f32, 1.0f32)], false)
			.end_render_pass()
		.end()
	}).fold(Ok(()), |e, x| match (&e, &x) { (&Err(_), _) => e, (_, &Err(_)) => x, _ => Ok(()) })));

	let mut frame_index = try!(main_frame.execute_rendering(&engine, &framebuffer_commands, None, &execute_next_signal));

	while engine.process_messages()
	{
		// Render code...
		if execute_next_signal.get_status().is_ok()
		{
			frame_index = try!
			{
				main_frame.present(&engine, frame_index).and_then(|()|
					main_frame.execute_rendering(&engine, &framebuffer_commands, None, &execute_next_signal))
			};
		}
	}

	Ok(())

	/*

	// Device to Device and Resource to Resource Synchronization //
	let render_target_sem = device.create_semaphore().unwrap();
	let transfer_sem = device.create_semaphore().unwrap();
	let fence = device.create_fence().unwrap();

	// Ready for Rendering
	let surface = create_surface(&instance, &window);
	let swapchain = Swapchain::create(&graphics_queue, &surface).unwrap();
	let render_area = VkRect2D(VkOffset2D(0, 0), swapchain.extent);
	let final_images = swapchain.object.get_images().unwrap();
	let final_image_views = create_image_views(&final_images, swapchain.format);
	let simple_pass = create_simple_render_pass(&device, swapchain.format);
	let final_framebuffers = create_framebuffers(&final_image_views, &simple_pass, swapchain.extent);

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
	let enemy_render = logical_resources::EnemyRenderer::new(&pp_commons, &simple_pass, swapchain.extent);
	let background_render = logical_resources::BackgroundRenderer::new(&pp_commons, &simple_pass, swapchain.extent);
	let player_render = logical_resources::PlayerRenderer::new(&pp_commons, &simple_pass, swapchain.extent);
	let debug_render = logical_resources::DebugRenderer::new(&pp_commons, &simple_pass, swapchain.extent);

	// Logical Resources //
	let di_desc = 1;
	let meshstore = logical_resources::Meshstore::new(memory_preallocator.meshstore_base);
	let projection_matrixes = logical_resources::ProjectionMatrixes::new(swapchain.extent);
	// let mut enemy_datastore = logical_resources::EnemyDatastore::new();
	// let background_datastore = logical_resources::BackgroundDatastore::new();
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
	let (projection_matrixes_ref, enemy_instance_data_ref, background_instance_data_ref, player_center_tf_ref) = uniform_memory_range.partial_borrow();
	let (enemy_instance_mult_ref, background_instance_mult_ref, player_rotq_ref) = instance_memory_range.partial_borrow();

    // Game Engine Instance and Initial Setups //
    let mut engine = Engine::new(uniform_memory_range, instance_memory_range);
    meshstore.initial_stage_data(instance_memory_range);
    engine.setup_parameters();

	// Initial Staging //
	{
		let player_rotq_unit = [UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32)), UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32))];

		meshstore.initial_stage_data(&mapped_range);
		projection_matrixes.initial_stage_data(uniform_memory_range);
	}

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
			.draw(4, MAX_BK_COUNT as u32, 0)
			.bind_pipeline(&enemy_render.state)
			.bind_vertex_buffers_partial(1, &[**memory_bound_resources.buffer], &[memory_preallocator.instance_base])
			.draw(4, MAX_ENEMY_COUNT as u32, 0)
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
			// debug_info_resources.update_text_data(&dt_mapped_range, delta_time.num_microseconds().unwrap() as f32 / 1000.0f32, enemy_counter);
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
            // game.update(delta_time);
		}

		if prev_frame_time.to(time::PreciseTime::now()) >= time::Duration::milliseconds(16)
		{
            // game.fixed_update();
            prev_frame_time = time::PreciseTime::now();
		}
	}
	device.wait_for_idle().unwrap();
	*/
}

/*
// Engine Instance
struct Engine<'d>
{
    prev_update_time: time::PreciseTime, prev_fixed_time: time::PreciseTime,
    instance_memory: &'d mut structures::InstanceMemory, uniform_memory: &'d mut structures::UniformMemory,
    projection_matrixes: logical_resources::ProjectionMatrixes, game: Game
}
impl <'d> Engine<'d>
{
    fn new(memory_range: &'d vk::MemoryMappedRange, preallocator: &'d device_resources::MemoryPreallocator) -> Self
    {
        Engine
        {
            instance_memory: memory_range.map_mut::<structures::InstanceMemory>(preallocator.instance_base),
            uniform_memory: memory_range.map_mut::<structures::UniformMemory>(preallocator.uniform_memory_base),
            projection_matrixes: logical_resources::ProjectionMatrixes::new(),
            prev_update_time: time::PreciseTime::now(), prev_fixed_time: time::PreciseTime::now(),
            game: Game::new()
        }
    }
    fn setup_parameters(&mut self)
    {
        self.projection_matrixes.initial_stage_data(self.uniform_memory);
        self.game.setup_parameters(self.instance_memory);
    }
    fn update(&mut self)
    {
        self.prev_fixed_time = if self.prev_fixed_time.to(time::PreciseTime::now()) >= time::Duration::milliseconds(16)
        {
            self.game.fixed_update();
            time::PreciseTime::now()
        } else { self.prev_fixed_time };

        let delta_time = self.prev_update_time.to(time::PreciseTime::now());
        self.game.update(self.instance_memory, self.uniform_memory, delta_time);
        self.prev_update_time = time::PreciseTime::now();
    }
}

// Game Instance
struct Game
{
    player: Player, enemy_list: LinkedList<Enemy>, enemy_counter: u32,
    enemy_datastore: logical_resources::EnemyDatastore, background_datastore: logical_resources::BackgroundDatastore,
    randomizer: rand::ThreadRng,
    require_appear_enemy: bool, require_appear_background: bool
}
impl Game
{
    fn new(instance_memory: &mut structures::InstanceMemory, uniform_memory: &mut structures::UniformMemory) -> Self
    {
        Game
        {
            enemy_datastore: logical_resources::EnemyDatastore::new(), background_datastore: logical_resources::BackgroundDatastore::new(),
            player: Player::new(uniform_memory, instance_memory), enemy_list: LinkedList::<Enemy>::new(), enemy_counter: 0,
            randomizer: rand::thread_rng(),
            require_appear_enemy: false, require_appear_background: false
        }
    }
    fn setup_parameters(&self, instance_memory: &mut structures::InstanceMemory)
    {
        instance_memory.enemy_instance_mult = [0; MAX_ENEMY_COUNT];
        instance_memory.background_instance_mult = [0; MAX_BK_COUNT];
    }
    fn update(&mut self, instance_memory: &mut structures::InstanceMemory, uniform_memory: &mut structures::UniformMemory, delta_time: time::Duration)
    {
        self.background_datastore.update(self.uniform_memory, instance_memory, &mut self.randomizer, delta_time, self.require_appear_background);
        self.player.update(instance_memory, uniform_memory);

        if self.require_appear_enemy
        {
            let left_range = rand::distributions::Range::new(-25.0f32, 25.0f32);
            if let Some(enemy) = Enemy::new(&mut self.enemy_datastore, instance_memory, uniform_memory, left_range.sample(&mut self.randomizer))
            {
                self.enemy_list.push_back(enemy);
                self.enemy_counter += 1;
            } else { println!("Warning: Unable to allocate memory block for enemy"); }
        }
        let living_list = LinkedList::<Enemy>::new();
        while let Some(e) = self.enemy_list.pop_front()
        {
            let died = e.update(&self.enemy_datastore, self.uniform_memory);
            if !died { living_list.push_back(e); } else { e.die(&mut self.enemy_datastore, self.instance_memory); self.enemy_counter -= 1; }
        }
        self.enemy_list = living_list;

        self.require_appear_enemy = false; self.require_appear_background = false;
    }
    fn fixed_update(&mut self)
    {
        let enemy_incidence_range = rand::distributions::Range::new(0, 40);
        let background_incidence_range = rand::distributions::Range::new(0, 4);

        self.require_appear_enemy = enemy_incidence_range.sample(&mut self.randomizer) == 0;
        self.require_appear_background = background_incidence_range.sample(&mut self.randomizer) == 0;
    }
}
*/
