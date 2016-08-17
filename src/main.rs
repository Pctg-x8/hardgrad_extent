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
// mod vertex_formats;
// mod device_resources;
// mod structures;
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

	let prelude = try!(prelude::Engine::new("HardGrad->Extent", VK_MAKE_VERSION!(0, 0, 1)));
	let main_frame = try!(prelude.create_render_window(VkExtent2D(640, 480), "HardGrad -> Extent"));
	let VkExtent2D(frame_width, frame_height) = main_frame.get_extent();
	let execution_coordinator = try!(prelude.create_queue_fence());

	let rp_attachment_descs =
	[
		prelude::AttachmentDesc
		{
			format: main_frame.get_format(),
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::PresentSrcKHR,
			.. Default::default()
		}
	];
	let render_passes = [prelude::PassDesc::single_fragment_output(0)];
	let rp_framebuffer_form = try!(prelude.create_render_pass(&rp_attachment_descs, &render_passes, &[]));
	let framebuffers = try!(main_frame.get_back_images().iter()
		.map(|ref x| prelude.create_framebuffer(&rp_framebuffer_form, &[&x.view], VkExtent3D(frame_width, frame_height, 1)))
		.collect::<Result<Vec<_>, _>>());

	while prelude.process_messages()
	{
		// Render code...
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

mod prelude
{
	use std;
	use vkffi::*;
	use render_vk::wrap as vk;
	use xcbw::*;
	use traits::*;
	use std::rc::Rc;
	use log;
	use ansi_term::*;
	use std::ffi::*;
	use std::os::raw::*;
	use libc::size_t;

	trait InternalWindow where Self: std::marker::Sized
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
	}
	pub struct EntireImage { pub resource: VkImage, pub view: vk::ImageView }
	pub struct XcbWindow
	{
		server: Rc<XServerConnection>, native: XWindowHandle,
		#[allow(dead_code)] device_obj: Rc<vk::Surface>, swapchain: Rc<vk::Swapchain>, rt: Vec<EntireImage>,
		format: VkFormat, extent: VkExtent2D, has_vsync: bool
	}
	impl InternalWindow for XcbWindow
	{
		type NativeWindow = XWindowHandle;
		type WindowServer = XServerConnection;

		fn create_unresizable(engine: &Engine, size: VkExtent2D, title: &str) -> Result<Box<Self>, EngineError>
		{
			let native = engine.window_system.create_unresizable_window(size, title);
			engine.window_system.show_window(native);
			engine.window_system.flush();

			let surface_info = VkXcbSurfaceCreateInfoKHR
			{
				sType: VkStructureType::XcbSurfaceCreateInfoKHR, pNext: std::ptr::null(), flags: 0,
				connection: engine.window_system.get_raw(), window: native
			};
			let surface = Rc::new(try!(vk::Surface::new_xcb(&engine.instance, &surface_info)));

			// capabilities check //
			if !engine.device.adapter.is_surface_support(engine.device.graphics_queue.family_index, &surface)
			{
				Err(EngineError::GenericError("Unsupported Surface"))
			}
			else
			{
				let surface_caps = engine.device.adapter.get_surface_caps(&surface);

				// Making desired parameters //
				let formats = engine.device.adapter.enumerate_surface_formats(&surface);
				let present_modes = engine.device.adapter.enumerate_present_modes(&surface);
				let format = try!(formats.iter().find(|&x| x.format == VkFormat::R8G8B8A8_SRGB || x.format == VkFormat::B8G8R8A8_SRGB)
					.ok_or(EngineError::GenericError("Desired Format(32bpp SRGB) is not supported")));
				let present_mode = try!(present_modes.iter().find(|&&x| x == VkPresentModeKHR::FIFO)
					.or_else(|| present_modes.iter().find(|&&x| x == VkPresentModeKHR::Mailbox))
					.ok_or(EngineError::GenericError("Desired Present Mode is not found")));
				let extent = match surface_caps.currentExtent
				{
					VkExtent2D(std::u32::MAX, _) | VkExtent2D(_, std::u32::MAX) => VkExtent2D(640, 480),
					ce => ce
				};

				// set information and create //
				let queue_family_indices = [engine.device.graphics_queue.family_index];
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
				let sc = try!(vk::Swapchain::new(&engine.device, &surface, &scinfo).map(|x| Rc::new(x)));
				let rt_images = try!(sc.get_images());
				let rt = try!(rt_images.iter().map(|&res|
				{
					vk::ImageView::new(&engine.device, &VkImageViewCreateInfo
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
					server: engine.window_system.clone(),
					native: native, device_obj: surface, swapchain: sc, rt: rt,
					format: format.format, extent: extent, has_vsync: *present_mode == VkPresentModeKHR::FIFO
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
	}
	pub struct QueueFence { internal: vk::Semaphore }
	pub struct Fence { internal: vk::Fence }

	pub struct AttachmentDesc
	{
		pub format: VkFormat, pub samples: VkSampleCountFlagBits,
		pub clear_on_load: Option<bool>, pub preserve_stored_value: bool,
		pub stencil_clear_on_load: Option<bool>, pub preserve_stored_stencil_value: bool,
		pub initial_layout: VkImageLayout, pub final_layout: VkImageLayout
	}
	impl std::default::Default for AttachmentDesc
	{
		fn default() -> Self
		{
			AttachmentDesc
			{
				format: VkFormat::UNDEFINED, samples: VK_SAMPLE_COUNT_1_BIT,
				clear_on_load: None, preserve_stored_value: false,
				stencil_clear_on_load: None, preserve_stored_stencil_value: false,
				initial_layout: VkImageLayout::Undefined, final_layout: VkImageLayout::Undefined
			}
		}
	}
	impl <'a> std::convert::Into<VkAttachmentDescription> for &'a AttachmentDesc
	{
		fn into(self) -> VkAttachmentDescription
		{
			VkAttachmentDescription
			{
				flags: 0, format: self.format, samples: self.samples,
				loadOp: self.clear_on_load.map(|b| if b { VkAttachmentLoadOp::Clear } else { VkAttachmentLoadOp::Load })
					.unwrap_or(VkAttachmentLoadOp::DontCare),
				stencilLoadOp: self.stencil_clear_on_load.map(|b| if b { VkAttachmentLoadOp::Clear } else { VkAttachmentLoadOp::Load })
					.unwrap_or(VkAttachmentLoadOp::DontCare),
				storeOp: if self.preserve_stored_value { VkAttachmentStoreOp::Store } else { VkAttachmentStoreOp::DontCare },
				stencilStoreOp: if self.preserve_stored_stencil_value { VkAttachmentStoreOp::Store } else { VkAttachmentStoreOp::DontCare },
				initialLayout: self.initial_layout, finalLayout: self.final_layout
			}
		}
	}
	pub type AttachmentRef = VkAttachmentReference;
	impl AttachmentRef
	{
		pub fn color(index: u32) -> Self { VkAttachmentReference(index, VkImageLayout::ColorAttachmentOptimal) }
	}
	pub struct PassDesc
	{
		pub input_attachment_indices: Vec<AttachmentRef>,
		pub color_attachment_indices: Vec<AttachmentRef>,
		pub resolved_attachment_indices: Option<Vec<AttachmentRef>>,
		pub depth_stencil_attachment_index: Option<AttachmentRef>,
		pub preserved_attachment_indices: Vec<u32>
	}
	impl std::default::Default for PassDesc
	{
		fn default() -> Self
		{
			PassDesc
			{
				input_attachment_indices: Vec::new(),
				color_attachment_indices: Vec::new(),
				resolved_attachment_indices: None,
				depth_stencil_attachment_index: None,
				preserved_attachment_indices: Vec::new()
			}
		}
	}
	impl PassDesc
	{
		pub fn single_fragment_output(index: u32) -> PassDesc
		{
			PassDesc { color_attachment_indices: vec![AttachmentRef::color(index)], .. Default::default() }
		}
	}
	impl <'a> std::convert::Into<VkSubpassDescription> for &'a PassDesc
	{
		fn into(self) -> VkSubpassDescription
		{
			VkSubpassDescription
			{
				flags: 0, pipelineBindPoint: VkPipelineBindPoint::Graphics,
				inputAttachmentCount: self.input_attachment_indices.len() as u32,
				pInputAttachments: self.input_attachment_indices.as_ptr(),
				colorAttachmentCount: self.color_attachment_indices.len() as u32,
				pColorAttachments: self.color_attachment_indices.as_ptr(),
				pResolveAttachments: self.resolved_attachment_indices.as_ref().map(|x| x.as_ptr()).unwrap_or(std::ptr::null()),
				pDepthStencilAttachment: self.depth_stencil_attachment_index.as_ref().map(|x| x as *const AttachmentRef).unwrap_or(std::ptr::null_mut()),
				preserveAttachmentCount: self.preserved_attachment_indices.len() as u32,
				pPreserveAttachments: self.preserved_attachment_indices.as_ptr()
			}
		}
	}
	pub struct PassDependency
	{
		pub src: u32, pub dst: u32,
		pub src_stage_mask: VkPipelineStageFlags, pub dst_stage_mask: VkPipelineStageFlags,
		pub src_access_mask: VkAccessFlags, pub dst_access_mask: VkAccessFlags,
		pub depend_by_region: bool
	}
	impl std::default::Default for PassDependency
	{
		fn default() -> Self
		{
			PassDependency
			{
				src: 0, dst: 0,
				src_stage_mask: VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
				dst_stage_mask: VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
				src_access_mask: VK_ACCESS_MEMORY_READ_BIT,
				dst_access_mask: VK_ACCESS_MEMORY_READ_BIT,
				depend_by_region: false
			}
		}
	}
	impl <'a> std::convert::Into<VkSubpassDependency> for &'a PassDependency
	{
		fn into(self) -> VkSubpassDependency
		{
			VkSubpassDependency
			{
				srcSubpass: self.src, dstSubpass: self.dst,
				srcStageMask: self.src_stage_mask, dstStageMask: self.dst_stage_mask,
				srcAccessMask: self.src_access_mask, dstAccessMask: self.dst_access_mask,
				dependencyFlags: if self.depend_by_region { VK_DEPENDENCY_BY_REGION_BIT } else { 0 }
			}
		}
	}
	pub struct RenderPass { internal: Rc<vk::RenderPass> }
	pub struct Framebuffer { mold: Rc<vk::RenderPass>, internal: vk::Framebuffer }

	pub struct Device
	{
		adapter: Rc<vk::PhysicalDevice>, internal: Rc<vk::Device>, graphics_queue: vk::Queue, transfer_queue: vk::Queue
	}
	impl std::ops::Deref for Device { type Target = Rc<vk::Device>; fn deref(&self) -> &Self::Target { &self.internal } }
	impl Device
	{
		fn new(adapter: &Rc<vk::PhysicalDevice>, features: &VkPhysicalDeviceFeatures,
			graphics_qf: u32, transfer_qf: Option<u32>, qf_props: &VkQueueFamilyProperties) -> Result<Self, EngineError>
		{
			fn device_queue_create_info(family_index: u32, count: u32, priorities: &[f32]) -> VkDeviceQueueCreateInfo
			{
				VkDeviceQueueCreateInfo
				{
					sType: VkStructureType::DeviceQueueCreateInfo, pNext: std::ptr::null(), flags: 0,
					queueFamilyIndex: family_index, queueCount: count, pQueuePriorities: priorities.as_ptr()
				}
			}
			// Ready Parameters //
			static QUEUE_PRIORITIES: [f32; 2] = [0.0f32; 2];
			match transfer_qf
			{
				Some(t) => info!(target: "Prelude", "Not sharing queue family: g={}, t={}", graphics_qf, t),
				None => info!(target: "Prelude", "Sharing queue family: {}", graphics_qf)
			};
			let queue_info = match transfer_qf
			{
				Some(transfer_qf) => vec![
					device_queue_create_info(graphics_qf, 1, &QUEUE_PRIORITIES[0..1]),
					device_queue_create_info(transfer_qf, 1, &QUEUE_PRIORITIES[1..2])
				],
				None => vec![device_queue_create_info(graphics_qf, std::cmp::min(qf_props.queueCount, 2), &QUEUE_PRIORITIES)]
			};
			vk::Device::new(adapter, &queue_info, &["VK_LAYER_LUNARG_standard_validation"], &["VK_KHR_swapchain"], features).map(|device| Device
			{
				graphics_queue: device.get_queue(graphics_qf, 0),
				transfer_queue: device.get_queue(transfer_qf.unwrap_or(graphics_qf), queue_info[0].queueCount - 1),
				internal: Rc::new(device), adapter: adapter.clone()
			}).map_err(|e| EngineError::from(e))
		}
	}

	pub enum EngineError
	{
		DeviceError(VkResult), GenericError(&'static str)
	}
	impl std::convert::From<VkResult> for EngineError
	{
		fn from(res: VkResult) -> EngineError { EngineError::DeviceError(res) }
	}
	impl std::fmt::Debug for EngineError
	{
		fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
		{
			match self
			{
				&EngineError::DeviceError(ref r) => write!(formatter, "DeviceError: {:?}", r),
				&EngineError::GenericError(ref e) => write!(formatter, "GenericError: {}", e),
			}
		}
	}
	pub fn crash(err: EngineError) -> !
	{
		error!(target: "Prelude", "{:?}", err);
		panic!("Application has exited due to {}", match err
		{
			EngineError::DeviceError(_) => "DeviceError",
			EngineError::GenericError(_) => "GenericError"
		})
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
	pub struct Engine
	{
		window_system: Rc<XServerConnection>, instance: Rc<vk::Instance>, #[allow(dead_code)] debug_callback: vk::DebugReportCallback,
		device: Device
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

			Ok(Engine
			{
				window_system: window_server, instance: instance, debug_callback: dbg_callback, device: device
			})
		}
		pub fn create_render_window(&self, size: VkExtent2D, title: &str) -> Result<Box<RenderWindow>, EngineError>
		{
			info!(target: "Prelude", "Creating Render Window \"{}\" ({}x{})", title, size.0, size.1);
			XcbWindow::create_unresizable(&self, size, title).map(|x| x as Box<RenderWindow>)
		}
		pub fn process_messages(&self) -> bool
		{
			self.window_system.process_messages()
		}
		
		pub fn create_fence(&self) -> Result<Fence, EngineError>
		{
			vk::Fence::new(&self.device).map(|f| Fence { internal: f }).map_err(EngineError::from)
		}
		pub fn create_queue_fence(&self) -> Result<QueueFence, EngineError>
		{
			vk::Semaphore::new(&self.device).map(|f| QueueFence { internal: f }).map_err(EngineError::from)
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
			vk::RenderPass::new(&self.device, &rp_info).map(|p| RenderPass { internal: Rc::new(p) }).map_err(EngineError::from)
		}
		pub fn create_framebuffer(&self, mold: &RenderPass, attachments: &[&vk::ImageView], form: VkExtent3D) -> Result<Framebuffer, EngineError>
		{
			let attachments_native = attachments.into_iter().map(|x| x.get()).collect::<Vec<_>>();
			let VkExtent3D(width, height, layers) = form;
			let info = VkFramebufferCreateInfo
			{
				sType: VkStructureType::FramebufferCreateInfo, pNext: std::ptr::null(), flags: 0,
				renderPass: mold.internal.get(),
				attachmentCount: attachments_native.len() as u32, pAttachments: attachments_native.as_ptr(),
				width: width, height: height, layers: layers
			};
			vk::Framebuffer::new(&self.device, &info).map(|f| Framebuffer { mold: mold.internal.clone(), internal: f }).map_err(EngineError::from)
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
