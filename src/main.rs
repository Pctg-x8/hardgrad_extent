extern crate libc;
extern crate xcb;
extern crate nalgebra;
extern crate rand;
extern crate time;
extern crate unicode_normalization;
extern crate thread_scoped;
#[macro_use] extern crate log;
extern crate ansi_term;
extern crate freetype_sys;
extern crate glob;
extern crate epoll;
extern crate socket;
#[macro_use] mod vkffi;
mod render_vk;
mod interlude;
use interlude::traits::*;
use interlude::{InputKeys, InputAxis, InputType};

mod constants;
use constants::*;
mod traits;
mod vertex_formats;
use vertex_formats::*;
mod structures;
mod logical_resources;
mod utils;
use nalgebra::*;
use rand::distributions::*;

mod smaa_extra_textures;
use smaa_extra_textures::*;
mod block_compression;
use block_compression::*;

use vkffi::*;

use std::collections::LinkedList;
use std::cell::RefCell;

// For InputSystem
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum LogicalInputTypes
{
	Horizontal, Vertical, Shoot, Slowdown, Overdrive
}

struct Enemy<'a>
{
	datastore_ref: &'a RefCell<logical_resources::EnemyDatastore<'a>>,
	block_index: u32, left: f32, living_secs: f32
}
impl <'a> Enemy<'a>
{
	pub fn new(datastore: &'a RefCell<logical_resources::EnemyDatastore<'a>>, init_left: f32) -> Option<Self>
	{
		let mut datastore_ref = datastore.borrow_mut();
		datastore_ref.allocate_block().map(move |index|
		{
			datastore_ref.update_instance_data(index,
				UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32)).quaternion(), UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32)).quaternion(),
				&Vector4::new(init_left, 0.0f32, 0.0f32, 0.0f32));
			Enemy
			{
				datastore_ref: datastore, block_index: index, left: init_left, living_secs: 0.0f32
			}
		})
	}
	pub fn update(&mut self, delta_time: f32) -> bool
	{
		let current_y = if self.living_secs < 0.875f32
		{
			15.0f32 * (1.0f32 - (1.0f32 - self.living_secs / 0.875f32).powi(2)) - 3.0f32
		}
		else
		{
			15.0f32 + (self.living_secs - 0.875f32) * 2.5f32 - 3.0f32
		};
		self.datastore_ref.borrow_mut().update_instance_data(self.block_index,
			UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32).normalize() * (260.0f32 * self.living_secs).to_radians()).quaternion(),
			UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32).normalize() * (-260.0f32 * self.living_secs + 13.0f32).to_radians()).quaternion(),
			&Vector4::new(self.left, current_y, 0.0f32, 0.0f32));
		self.living_secs += delta_time;

		current_y >= 50.0f32
	}
	pub fn die(self)
	{
		self.datastore_ref.borrow_mut().free_block(self.block_index);
	}
}

struct Player<'a>
{
	uniform_memory: &'a mut structures::CVector4, instance_memory: &'a mut [structures::CVector4; 2],
	living_secs: f32
}
impl <'a> Player<'a>
{
	fn new(uniform_ref: &'a mut structures::CVector4, instance_ref: &'a mut [structures::CVector4; 2]) -> Self
	{
		let u_quaternion = UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32));
		let quaternion_ref = u_quaternion.quaternion();

		instance_ref[0] = [quaternion_ref.i, quaternion_ref.j, quaternion_ref.k, quaternion_ref.w];
		instance_ref[1] = [quaternion_ref.i, quaternion_ref.j, quaternion_ref.k, quaternion_ref.w];
		*uniform_ref = [0.0f32, 38.0f32, 0.0f32, 0.0f32];

		Player
		{
			uniform_memory: uniform_ref, instance_memory: instance_ref,
			living_secs: 0.0f32
		}
	}
	fn update(&mut self, frame_delta: f32, input: &interlude::InputSystem<LogicalInputTypes>)
	{
		let u_quaternions = [
			UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32).normalize() * (260.0f32 * self.living_secs as f32).to_radians()),
			UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32).normalize() * (-260.0f32 * self.living_secs as f32 + 13.0f32).to_radians())
		];
		let mut quaternions = u_quaternions.iter().map(|x| x.quaternion()).map(|q| [q.i, q.j, q.k, q.w]);
		self.living_secs += frame_delta;

		self.uniform_memory[0] =
			(self.uniform_memory[0] + input[LogicalInputTypes::Horizontal] * 40.0f32 * frame_delta).max(-33.0f32).min(33.0f32);
		self.uniform_memory[1] =
			(self.uniform_memory[1] + input[LogicalInputTypes::Vertical] * 40.0f32 * frame_delta).max(1.5f32).min(45.0f32);

		self.instance_memory[0] = quaternions.next().unwrap();
		self.instance_memory[1] = quaternions.next().unwrap();
	}
}

fn main() { if let Err(e) = app_main() { interlude::crash(e); } }
fn app_main() -> Result<(), interlude::EngineError>
{
	utils::memory_management_test();

	let engine = try!{
		interlude::Engine::new_with_features("HardGrad->Extent", VK_MAKE_VERSION!(0, 0, 1), interlude::DeviceFeatures::new().enable_block_texture_compression())
			.map(|e| e.with_assets_in(std::env::current_dir().unwrap()))
	};
	let main_frame = try!(engine.create_render_window(VkExtent2D(640, 480), "HardGrad -> Extent"));
	let VkExtent2D(frame_width, frame_height) = main_frame.get_extent();
	let execute_next_signal = try!(engine.create_fence());

	let gbuffer_desc = interlude::ImageDescriptor2::new(VkFormat::R8G8B8A8_UNORM, main_frame.get_extent(), interlude::ImageUsagePresets::AsColorTexture).device_resource();
	let edgebuffer_desc = interlude::ImageDescriptor2::new(VkFormat::R8G8_UNORM, main_frame.get_extent(), interlude::ImageUsagePresets::AsColorTexture).device_resource();
	let blend_weight_desc = interlude::ImageDescriptor2::new(VkFormat::R8G8B8A8_UNORM, main_frame.get_extent(), interlude::ImageUsagePresets::AsColorTexture).device_resource();
	let smaa_areatex_desc = interlude::ImageDescriptor2::new(VkFormat::BC5_UNORM_BLOCK, VkExtent2D(AREATEX_WIDTH, AREATEX_HEIGHT), VK_IMAGE_USAGE_SAMPLED_BIT);
	let smaa_searchtex_desc = interlude::ImageDescriptor2::new(VkFormat::BC4_UNORM_BLOCK, VkExtent2D(SEARCHTEX_WIDTH, SEARCHTEX_HEIGHT), VK_IMAGE_USAGE_SAMPLED_BIT);
	let imagebuffer_placement = interlude::ImagePreallocator::new().image_2d(vec![&gbuffer_desc, &edgebuffer_desc, &blend_weight_desc, &smaa_areatex_desc, &smaa_searchtex_desc]);
	let (backbuffers, stage_images) = try!(engine.create_double_image(&imagebuffer_placement));
	let (backbuffers, stage_images) = (backbuffers, stage_images.unwrap());
	let gbuffer_view = try!(engine.create_image_view_2d(backbuffers.dim2(0), VkFormat::R8G8B8A8_UNORM,
		interlude::ComponentMapping::straight(), interlude::ImageSubresourceRange::base_color()));
	let edgebuffer_view = try!(engine.create_image_view_2d(backbuffers.dim2(1), VkFormat::R8G8_UNORM,
		interlude::ComponentMapping::straight(), interlude::ImageSubresourceRange::base_color()));
	let blend_weight_view = try!(engine.create_image_view_2d(backbuffers.dim2(2), VkFormat::R8G8B8A8_UNORM,
		interlude::ComponentMapping::straight(), interlude::ImageSubresourceRange::base_color()));
	let smaa_areatex_view = try!(engine.create_image_view_2d(backbuffers.dim2(3), VkFormat::BC5_UNORM_BLOCK,
		interlude::ComponentMapping::double_swizzle_rep(interlude::ComponentSwizzle::R, interlude::ComponentSwizzle::G), interlude::ImageSubresourceRange::base_color()));
	let smaa_searchtex_view = try!(engine.create_image_view_2d(backbuffers.dim2(4), VkFormat::BC4_UNORM_BLOCK,
		interlude::ComponentMapping::single_swizzle(interlude::ComponentSwizzle::R), interlude::ImageSubresourceRange::base_color()));
	let gbuffer_sampler = try!(engine.create_sampler(&interlude::SamplerState::new()));
	try!(stage_images.map().map(|mapped|
	{
		let areatex_compressed = BC5::compress(&AREATEX_BYTES, (AREATEX_WIDTH as usize, AREATEX_HEIGHT as usize));
		mapped.map_mut::<[u8; AREATEX_SIZE as usize / 2]>(stage_images.image2d_offset(0) as usize).copy_from_slice(&areatex_compressed);
		// mapped.map_mut::<[u8; AREATEX_SIZE as usize]>(stage_images.image2d_offset(0) as usize).copy_from_slice(&AREATEX_BYTES);
		let searchtex_compressed = BC4::compress(&SEARCHTEX_BYTES, (SEARCHTEX_WIDTH as usize, SEARCHTEX_HEIGHT as usize));
		mapped.map_mut::<[u8; SEARCHTEX_SIZE as usize / 2]>(stage_images.image2d_offset(1) as usize).copy_from_slice(&searchtex_compressed);
		// mapped.map_mut::<[u8; SEARCHTEX_SIZE as usize]>(stage_images.image2d_offset(1) as usize).copy_from_slice(&SEARCHTEX_BYTES);
	}));

	let rp_attachment_descs =
	[
		interlude::AttachmentDesc
		{ // gbuffer
			format: VkFormat::R8G8B8A8_UNORM, clear_on_load: Some(true), preserve_stored_value: false,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
			.. Default::default()
		},
		interlude::AttachmentDesc
		{ // SMAA edgebuffer
			format: VkFormat::R8G8_UNORM, clear_on_load: Some(true), preserve_stored_value: false,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
			.. Default::default()
		},
		interlude::AttachmentDesc
		{ // SMAA blend weight buffer
			format: VkFormat::R8G8B8A8_UNORM, clear_on_load: Some(true), preserve_stored_value: false,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
			.. Default::default()
		},
		interlude::AttachmentDesc
		{ // swapchain buffer
			format: main_frame.get_format(), clear_on_load: Some(true), preserve_stored_value: true,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::PresentSrcKHR,
			.. Default::default()
		}
	];
	let render_passes =
	[
		interlude::PassDesc::single_fragment_output(0),
		interlude::PassDesc
		{
			color_attachment_indices: vec![interlude::AttachmentRef::color(1)],
			preserved_attachment_indices: vec![0],
			.. Default::default()
		},
		interlude::PassDesc
		{
			color_attachment_indices: vec![interlude::AttachmentRef::color(2)],
			preserved_attachment_indices: vec![0, 1],
			.. Default::default()
		},
		interlude::PassDesc { color_attachment_indices: vec![interlude::AttachmentRef::color(3)], .. Default::default() }
	];
	let pass_deps =
	[
		interlude::PassDependency
		{
			src: 0, dst: 1,
			src_stage_mask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, dst_stage_mask: VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
			src_access_mask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, dst_access_mask: VK_ACCESS_SHADER_READ_BIT,
			depend_by_region: false
		},
		interlude::PassDependency
		{
			src: 1, dst: 2,
			src_stage_mask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, dst_stage_mask: VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
			src_access_mask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, dst_access_mask: VK_ACCESS_SHADER_READ_BIT,
			depend_by_region: false
		},
		interlude::PassDependency
		{
			src: 0, dst: 3,
			src_stage_mask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, dst_stage_mask: VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
			src_access_mask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, dst_access_mask: VK_ACCESS_SHADER_READ_BIT,
			depend_by_region: false
		},
		interlude::PassDependency
		{
			src: 2, dst: 3,
			src_stage_mask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, dst_stage_mask: VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
			src_access_mask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, dst_access_mask: VK_ACCESS_SHADER_READ_BIT,
			depend_by_region: false
		}
	];
	let rp_framebuffer_form = try!(engine.create_render_pass(&rp_attachment_descs, &render_passes, &pass_deps));
	let framebuffers = try!(main_frame.get_back_images().iter()
		.map(|&x| engine.create_framebuffer(&rp_framebuffer_form, &[&gbuffer_view, &edgebuffer_view, &blend_weight_view, x], VkExtent3D(frame_width, frame_height, 1)))
		.collect::<Result<Vec<_>, _>>());

	// Resources //
	let application_buffer_prealloc = engine.buffer_preallocate(&[
		(std::mem::size_of::<[interlude::PosUV; 4]>(), interlude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::VertexMemoryForWireRender>(), interlude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::IndexMemory>(), interlude::BufferDataType::Index),
		(std::mem::size_of::<structures::InstanceMemory>(), interlude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::UniformMemory>(), interlude::BufferDataType::Uniform)
	]);
	let (application_data, appdata_stage) = try!(engine.create_double_buffer(&application_buffer_prealloc));

	// setup initial data //
	try!(appdata_stage.map().map(|mapped|
	{
		*mapped.map_mut::<[interlude::PosUV; 4]>(application_buffer_prealloc.offset(0)) = [
			interlude::PosUV(-1.0f32, -1.0f32, 0.0f32, 0.0f32), interlude::PosUV(1.0f32, -1.0f32, 1.0f32, 0.0f32),
			interlude::PosUV(-1.0f32, 1.0f32, 0.0f32, 1.0f32), interlude::PosUV(1.0f32, 1.0f32, 1.0f32, 1.0f32)
		];
		let vertices = mapped.map_mut::<structures::VertexMemoryForWireRender>(application_buffer_prealloc.offset(1));
		let indices = mapped.map_mut::<structures::IndexMemory>(application_buffer_prealloc.offset(2));
		vertices.unit_plane_source_vts = [
			Position(-1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, 0.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, 0.0f32, 1.0f32)
		];
		vertices.player_cube_vts = [
			Position(-1.0f32, -1.0f32, -1.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, -1.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, -1.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, -1.0f32, 1.0f32),
			Position(-1.0f32, -1.0f32,  1.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32,  1.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32,  1.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32,  1.0f32, 1.0f32)
		];
		indices.player_cube_ids = [
			0, 1, 1, 2, 2, 3, 3, 0,
			4, 5, 5, 6, 6, 7, 7, 4,
			0, 4, 1, 5, 2, 6, 3, 7
		];
		let uniforms = mapped.map_mut::<structures::UniformMemory>(application_buffer_prealloc.offset(4));
		logical_resources::projection_matrixes::setup_parameters(uniforms, main_frame.get_extent());
		uniforms.render_target_desc = [(frame_width as f32).recip(), (frame_height as f32).recip(), frame_width as f32, frame_height as f32];
	}));

	// Descriptor Set //
	let uniform_memory_info = interlude::BufferInfo(&application_data, application_buffer_prealloc.offset(4) .. application_buffer_prealloc.total_size());
	let gbuffer_info = interlude::ImageInfo(&gbuffer_sampler, &gbuffer_view, VkImageLayout::ShaderReadOnlyOptimal);
	let edgebuffer_info = interlude::ImageInfo(&gbuffer_sampler, &edgebuffer_view, VkImageLayout::ShaderReadOnlyOptimal);
	let blendweight_info = interlude::ImageInfo(&gbuffer_sampler, &blend_weight_view, VkImageLayout::ShaderReadOnlyOptimal);
	let areatex_info = interlude::ImageInfo(&gbuffer_sampler, &smaa_areatex_view, VkImageLayout::ShaderReadOnlyOptimal);
	let searchtex_info = interlude::ImageInfo(&gbuffer_sampler, &smaa_searchtex_view, VkImageLayout::ShaderReadOnlyOptimal);
	let dslayout_u1 = try!(engine.create_descriptor_set_layout(&[
		interlude::Descriptor::Uniform(1, vec![interlude::ShaderStage::Vertex, interlude::ShaderStage::Geometry, interlude::ShaderStage::Fragment])
	]));
	let dslayouts_smaa =
	[
		try!(engine.create_descriptor_set_layout(&[interlude::Descriptor::CombinedSampler(1, vec![interlude::ShaderStage::Fragment])])),
		try!(engine.create_descriptor_set_layout(&[interlude::Descriptor::CombinedSampler(3, vec![interlude::ShaderStage::Fragment])])),
		try!(engine.create_descriptor_set_layout(&[interlude::Descriptor::CombinedSampler(2, vec![interlude::ShaderStage::Fragment])]))
	];
	let all_descriptor_sets = try!(engine.preallocate_all_descriptor_sets(&[&dslayout_u1, &dslayouts_smaa[0], &dslayouts_smaa[1], &dslayouts_smaa[2]]));
	engine.update_descriptors(&[
		interlude::DescriptorSetWriteInfo::UniformBuffer(all_descriptor_sets[0], 0, vec![uniform_memory_info]),
		interlude::DescriptorSetWriteInfo::CombinedImageSampler(all_descriptor_sets[1], 0, vec![gbuffer_info.clone()]),
		interlude::DescriptorSetWriteInfo::CombinedImageSampler(all_descriptor_sets[2], 0, vec![edgebuffer_info, areatex_info, searchtex_info]),
		interlude::DescriptorSetWriteInfo::CombinedImageSampler(all_descriptor_sets[3], 0, vec![gbuffer_info, blendweight_info])
	]);

	// Shading Structures //
	let raw_output_vert = try!(engine.create_vertex_shader_from_asset("shaders.RawOutput", "main", &[
		interlude::VertexBinding::PerVertex(std::mem::size_of::<vertex_formats::Position>() as u32),
		interlude::VertexBinding::PerInstance(std::mem::size_of::<u32>() as u32)
	], &[interlude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0), interlude::VertexAttribute(1, VkFormat::R32_UINT, 0)]));
	let player_rotor_vert = try!(engine.create_vertex_shader_from_asset("shaders.PlayerRotor", "main", &[
		interlude::VertexBinding::PerVertex(std::mem::size_of::<vertex_formats::Position>() as u32),
		interlude::VertexBinding::PerInstance(std::mem::size_of::<structures::CVector4>() as u32)
	], &[interlude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0), interlude::VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)]));
	let smaa_edge_ppv = try!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.EdgeDetectionV", "main"));
	let smaa_bw_ppv = try!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.BlendWeightCalcV", "main"));
	let smaa_combine_ppv = try!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.CombineV", "main"));
	let backline_duplicator = try!(engine.create_geometry_shader_from_asset("shaders.BackLineDuplicator", "main"));
	let enemy_duplicator = try!(engine.create_geometry_shader_from_asset("shaders.EnemyDuplicator", "main"));
	let through_color_frag = try!(engine.create_fragment_shader_from_asset("shaders.ThroughColor", "main"));
	let smaa_edge_detection_frag = try!(engine.create_fragment_shader_from_asset("shaders.smaa.EdgeDetection", "main"));
	let smaa_blend_weight_frag = try!(engine.create_fragment_shader_from_asset("shaders.smaa.BlendWeightCalc", "main"));
	let smaa_combine_frag = try!(engine.create_fragment_shader_from_asset("shaders.smaa.Combine", "main"));

	let swapchain_viewport = VkViewport(0.0f32, 0.0f32, frame_width as f32, frame_height as f32, 0.0f32, 1.0f32);
	let wire_render_layout = try!(engine.create_pipeline_layout(&[&dslayout_u1], &[interlude::PushConstantDesc(VK_SHADER_STAGE_VERTEX_BIT, 0 .. 16)]));
	let smaa_layouts = 
	[
		try!(engine.create_pipeline_layout(&[&dslayout_u1, &dslayouts_smaa[0]], &[])),
		try!(engine.create_pipeline_layout(&[&dslayout_u1, &dslayouts_smaa[1]], &[])),
		try!(engine.create_pipeline_layout(&[&dslayout_u1, &dslayouts_smaa[2]], &[]))
	];
	let background_render_state = interlude::GraphicsPipelineBuilder::new(&wire_render_layout, &rp_framebuffer_form, 0)
		.vertex_shader(&raw_output_vert).geometry_shader(&backline_duplicator).fragment_shader(&through_color_frag)
		.primitive_topology(interlude::PrimitiveTopology::LineList(true))
		.viewport_scissors(&[interlude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
		.blend_state(&[interlude::AttachmentBlendState::PremultipliedAlphaBlend]);
	let enemy_render_state = interlude::GraphicsPipelineBuilder::inherit(&background_render_state)
	 	.geometry_shader(&enemy_duplicator)
		.blend_state(&[interlude::AttachmentBlendState::Disabled]);
	let player_render_state = interlude::GraphicsPipelineBuilder::new(&wire_render_layout, &rp_framebuffer_form, 0)
		.vertex_shader(&player_rotor_vert).fragment_shader(&through_color_frag)
		.primitive_topology(interlude::PrimitiveTopology::LineList(false))
		.viewport_scissors(&[interlude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
		.blend_state(&[interlude::AttachmentBlendState::Disabled]);
	// PostProcessing //
	let pp_smaa_edge_detection_state = interlude::GraphicsPipelineBuilder::new(&smaa_layouts[0], &rp_framebuffer_form, 1)
		.vertex_shader(&smaa_edge_ppv).fragment_shader(&smaa_edge_detection_frag)
		.primitive_topology(interlude::PrimitiveTopology::TriangleStrip(false))
		.viewport_scissors(&[interlude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
		.blend_state(&[interlude::AttachmentBlendState::Disabled]);
	let pp_smaa_blend_weight_state = interlude::GraphicsPipelineBuilder::new(&smaa_layouts[1], &rp_framebuffer_form, 2)
		.vertex_shader(&smaa_bw_ppv).fragment_shader(&smaa_blend_weight_frag)
		.primitive_topology(interlude::PrimitiveTopology::TriangleStrip(false))
		.viewport_scissors(&[interlude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
		.blend_state(&[interlude::AttachmentBlendState::Disabled]);
	let pp_smaa_combine_state = interlude::GraphicsPipelineBuilder::new(&smaa_layouts[2], &rp_framebuffer_form, 3)
		.vertex_shader(&smaa_combine_ppv).fragment_shader(&smaa_combine_frag)
		.primitive_topology(interlude::PrimitiveTopology::TriangleStrip(false))
		.viewport_scissors(&[interlude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
		.blend_state(&[interlude::AttachmentBlendState::Disabled]);
	let pipeline_states = try!(engine.create_graphics_pipelines(
		&[&background_render_state, &enemy_render_state, &player_render_state, &pp_smaa_edge_detection_state, &pp_smaa_blend_weight_state, &pp_smaa_combine_state]
	));
	let ref background_render = pipeline_states[0];
	let ref enemy_render = pipeline_states[1];
	let ref player_render = pipeline_states[2];
	let ref pp_smaa_edge_detection = pipeline_states[3];
	let ref pp_smaa_blend_weight_calc = pipeline_states[4];
	let ref pp_smaa_combine = pipeline_states[5];

	// Initial Data Transmission, Layouting for Swapchain Backbuffer Images //
	{
		let setup_commands = try!(engine.allocate_transient_transfer_command_buffers(1));

		let buffer_memory_barriers = [
			interlude::BufferMemoryBarrier::hold_ownership(&appdata_stage, 0 .. application_buffer_prealloc.total_size(),
				0, VK_ACCESS_TRANSFER_READ_BIT),
			interlude::BufferMemoryBarrier::hold_ownership(&application_data, 0 .. application_buffer_prealloc.total_size(),
				0, VK_ACCESS_TRANSFER_WRITE_BIT)
		];
		let buffer_memory_barriers_ret = [
			interlude::BufferMemoryBarrier::hold_ownership(&appdata_stage, 0 .. application_buffer_prealloc.total_size(),
				VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_MEMORY_READ_BIT),
			interlude::BufferMemoryBarrier::hold_ownership(&application_data, 0 .. application_buffer_prealloc.total_size(),
				VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT)
		];
		let blitted_image_templates =
		[
			interlude::ImageMemoryBarrier::template(&**backbuffers.dim2(3), interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(&**backbuffers.dim2(4), interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(stage_images.dim2(0), interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(stage_images.dim2(1), interlude::ImageSubresourceRange::base_color())
		];
		let image_memory_barriers = main_frame.get_back_images().iter()
			.map(|x| interlude::ImageMemoryBarrier::hold_ownership(*x, interlude::ImageSubresourceRange::base_color(),
			0, VK_ACCESS_MEMORY_READ_BIT, VkImageLayout::Undefined, VkImageLayout::PresentSrcKHR)).chain([
				interlude::ImageMemoryBarrier::hold_ownership(&**backbuffers.dim2(0), interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				interlude::ImageMemoryBarrier::hold_ownership(&**backbuffers.dim2(1), interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				interlude::ImageMemoryBarrier::hold_ownership(&**backbuffers.dim2(2), interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				blitted_image_templates[0].into_transfer_dst(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized),
				blitted_image_templates[1].into_transfer_dst(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized),
				blitted_image_templates[2].into_transfer_src(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized),
				blitted_image_templates[3].into_transfer_src(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized)
			].into_iter().map(|&x| x)).collect::<Vec<_>>();
		let image_memory_barriers_ret =
		[
			blitted_image_templates[0].from_transfer_dst(VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal),
			blitted_image_templates[1].from_transfer_dst(VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal)
		];

		try!(setup_commands.begin(0).and_then(|recorder|
			recorder.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false,
				&[], &buffer_memory_barriers, &image_memory_barriers)
			.copy_buffer(&appdata_stage, &application_data, &[interlude::BufferCopyRegion(0, 0, application_buffer_prealloc.total_size() as usize)])
			.copy_image(stage_images.dim2(0), &**backbuffers.dim2(3), VkImageLayout::TransferSrcOptimal, VkImageLayout::TransferDestOptimal,
				&[interlude::ImageCopyRegion(interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), VkExtent3D(AREATEX_WIDTH, AREATEX_HEIGHT, 1))])
			.copy_image(stage_images.dim2(1), &**backbuffers.dim2(4), VkImageLayout::TransferSrcOptimal, VkImageLayout::TransferDestOptimal,
				&[interlude::ImageCopyRegion(interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), VkExtent3D(SEARCHTEX_WIDTH, SEARCHTEX_HEIGHT, 1))])
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT, false, &[], &buffer_memory_barriers_ret, &image_memory_barriers_ret)
			.end()
		));

		try!(setup_commands.execute());
	}

	// Debug Information //
	let frame_time_ms = RefCell::new(0.0f64);
	let enemy_count = RefCell::new(0u32);
	let debug_info = try!(interlude::DebugInfo::new(&engine, &[
		interlude::DebugLine::Float("Frame Time".to_owned(), &frame_time_ms, Some("ms".to_string())),
		interlude::DebugLine::UnsignedInt("Enemy Count".to_owned(), &enemy_count, None)
	], &rp_framebuffer_form, 3, swapchain_viewport));

	// Rendering Commands //
	let combine_commands = try!(engine.allocate_bundled_command_buffers(2 * framebuffers.len() as u32));
	for (n, f) in framebuffers.iter().enumerate()
	{
		try!(combine_commands.begin(0 + 2 * n, &rp_framebuffer_form, 3, f).and_then(|recorder|
			recorder
				.bind_pipeline(pp_smaa_combine)
				.bind_descriptor_sets(&smaa_layouts[2], &[all_descriptor_sets[0], all_descriptor_sets[3]])
				.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(0))])
				.draw(4, 1)
				.end()
		));
		try!(combine_commands.begin(1 + 2 * n, &rp_framebuffer_form, 3, f).and_then(|recorder|
			recorder.inject_commands(|r| debug_info.inject_render_commands(r)).end()
		));
	}
	let framebuffer_commands = try!(engine.allocate_graphics_command_buffers(main_frame.get_back_images().len() as u32));
	try!(framebuffer_commands.begin_all().and_then(|iter| iter.map(|(i, recorder)|
	{
		let clear_values = [
			interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.015625f32, 1.0f32),
			interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.0f32, 0.0f32),
			interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.0f32, 0.0f32),
			interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.0f32, 0.0f32)
		];
		let color_output_barrier = interlude::ImageMemoryBarrier::template(main_frame.get_back_images()[i], interlude::ImageSubresourceRange::base_color())
			.hold_ownership(VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::PresentSrcKHR, VkImageLayout::ColorAttachmentOptimal);
		/*let ibar_gbuffer_end = interlude::ImageMemoryBarrier::template(gbuffer_obj, interlude::ImageSubresourceRange::base_color())
			.hold_ownership(VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ColorAttachmentOptimal, VkImageLayout::ShaderReadOnlyOptimal);
		let ibar_edgebuffer_end = interlude::ImageMemoryBarrier::template(edgebuffer_obj, interlude::ImageSubresourceRange::base_color())
			.hold_ownership(VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ColorAttachmentOptimal, VkImageLayout::ShaderReadOnlyOptimal);
		let ibar_blendweight_end = interlude::ImageMemoryBarrier::template(blendweight_obj, interlude::ImageSubresourceRange::base_color())
			.hold_ownership(VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ColorAttachmentOptimal, VkImageLayout::ShaderReadOnlyOptimal);*/

		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, false, &[], &[], &[color_output_barrier])
			.begin_render_pass(&framebuffers[i], &clear_values, false)
			// Pass 0 : Render to Buffer //
			.bind_descriptor_sets(&wire_render_layout, &all_descriptor_sets[0..1])
			.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(1))])
			.bind_pipeline(background_render)
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3) + structures::background_instance_offs())])
			.push_constants(&wire_render_layout, &[interlude::ShaderStage::Vertex],
				0 .. std::mem::size_of::<f32>() as u32 * 4, &[0.125f32, 0.5f32, 0.1875f32, 0.625f32])
			.draw(4, MAX_BK_COUNT as u32)
			.bind_pipeline(enemy_render)
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3))])
			.push_constants(&wire_render_layout, &[interlude::ShaderStage::Vertex],
				0 .. std::mem::size_of::<f32>() as u32 * 4, &[0.25f32, 0.9875f32, 1.5f32, 1.0f32])
			.draw(4, MAX_ENEMY_COUNT as u32)
			.bind_pipeline(player_render)
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3) + structures::player_instance_offs())])
			.bind_index_buffer(&application_data, application_buffer_prealloc.offset(2))
			.push_constants(&wire_render_layout, &[interlude::ShaderStage::Vertex],
				0 .. std::mem::size_of::<f32>() as u32 * 4, &[1.5f32, 1.25f32, 0.375f32, 1.0f32])
			.draw_indexed(24, 2, 4)
			// .pipeline_barrier(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, false, &[], &[], &[ibar_gbuffer_end])
			.next_subpass(false)
			// Pass 1 : Edge Detection(SMAA 1x) //
			.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(0))])
			.bind_pipeline(pp_smaa_edge_detection)
			.bind_descriptor_sets_partial(&smaa_layouts[0], 1, &all_descriptor_sets[1..2])
			.draw(4, 1)
			// .pipeline_barrier(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, false, &[], &[], &[ibar_edgebuffer_end])
			.next_subpass(false)
			// Pass 2 : Blend Weight Calculation(SMAA 1x) //
			.bind_pipeline(pp_smaa_blend_weight_calc)
			.bind_descriptor_sets_partial(&smaa_layouts[1], 1, &all_descriptor_sets[2..3])
			.draw(4, 1)
			// .pipeline_barrier(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, false, &[], &[], &[ibar_blendweight_end])
			.next_subpass(true)
			// Pass 3 : SMAA Combine and Debug Print //
			.execute_commands(&combine_commands[i * 2 .. i * 2 + 2])
			.end_render_pass()
		.end()
	}).collect::<Result<Vec<_>, _>>()));
	// Transfer Commands //
	let update_commands = try!(engine.allocate_transfer_command_buffers(1));
	try!(update_commands.begin(0).and_then(|recorder|
	{
		let uoffs = application_buffer_prealloc.offset(3);
		let buffer_barriers = [
			interlude::BufferMemoryBarrier::hold_ownership(&application_data, uoffs .. application_buffer_prealloc.total_size(),
				VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT, VK_ACCESS_TRANSFER_WRITE_BIT),
			interlude::BufferMemoryBarrier::hold_ownership(&appdata_stage, uoffs .. application_buffer_prealloc.total_size(),
				VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_TRANSFER_READ_BIT)
		];
		let buffer_barriers_ret = [
			interlude::BufferMemoryBarrier::hold_ownership(&application_data, uoffs .. application_buffer_prealloc.total_size(),
				VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT),
			interlude::BufferMemoryBarrier::hold_ownership(&appdata_stage, uoffs .. application_buffer_prealloc.total_size(),
				VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_MEMORY_READ_BIT)
		];

		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false, &[], &buffer_barriers, &[])
			.copy_buffer(&appdata_stage, &application_data, &[interlude::BufferCopyRegion(uoffs, uoffs, application_buffer_prealloc.total_size() as usize - uoffs)])
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, false, &[], &buffer_barriers_ret, &[])
		.end()
	}));

	let mut frame_index = try!(main_frame.execute_rendering(&engine, &framebuffer_commands, None, Some(&debug_info), &execute_next_signal));

	let mapped_range = try!(appdata_stage.map());
	let mapped_uniform_data = mapped_range.map_mut::<structures::UniformMemory>(application_buffer_prealloc.offset(4));
	let mapped_instance_data = mapped_range.map_mut::<structures::InstanceMemory>(application_buffer_prealloc.offset(3));
	let (_, uref_enemy, uref_bk, uref_player_center) = mapped_uniform_data.partial_borrow();
	let (iref_enemy, iref_bk, iref_player) = mapped_instance_data.partial_borrow();
	let mut background_datastore = logical_resources::BackgroundDatastore::new(uref_bk, iref_bk);
	let enemy_datastore = RefCell::new(logical_resources::EnemyDatastore::new(uref_enemy, iref_enemy));

	// double-buffered enemy entity list //
	let mut enemy_entities: LinkedList<Enemy> = LinkedList::new();
	let mut player = Player::new(uref_player_center, iref_player);

	let mut secs_from_last_fixed = 0.0f32;
	let mut input = try!(interlude::InputSystem::new())
		.add_input(LogicalInputTypes::Horizontal, InputType::Axis(InputAxis::X))
		.add_input(LogicalInputTypes::Horizontal, InputType::KeyAsAxis(InputKeys::Left, InputKeys::Right))
		.add_input(LogicalInputTypes::Vertical, InputType::Axis(InputAxis::Y))
		.add_input(LogicalInputTypes::Vertical, InputType::KeyAsAxis(InputKeys::Up, InputKeys::Down))
		.add_input(LogicalInputTypes::Shoot, InputType::Key(InputKeys::ButtonA))
		.add_input(LogicalInputTypes::Shoot, InputType::Key(InputKeys::Character('z')))
		.add_input(LogicalInputTypes::Slowdown, InputType::Axis(InputAxis::RZ))
		.add_input(LogicalInputTypes::Slowdown, InputType::Key(InputKeys::ButtonX))
		.add_input(LogicalInputTypes::Slowdown, InputType::Key(InputKeys::Character('x')))
		.add_input(LogicalInputTypes::Overdrive, InputType::Axis(InputAxis::Z));
	let mut randomizer = rand::thread_rng();
	let background_appear_rate = rand::distributions::Range::new(0, 6);
	let enemy_appear_rate = rand::distributions::Range::new(0, 40);
	let enemy_left_range = rand::distributions::Range::new(-25.0f32, 25.0f32);
	let mut background_next_appear = false;
	let mut enemy_next_appear = false;
	let mut prev_time = time::PreciseTime::now();
	while engine.process_messages()
	{
		// Render code...
		if execute_next_signal.get_status().is_ok()
		{
			let delta_time = prev_time.to(time::PreciseTime::now());
			*frame_time_ms.borrow_mut() = delta_time.num_microseconds().unwrap_or(-1) as f64 / 1000.0f64;
			frame_index = try!
			{
				execute_next_signal.clear().and_then(|()|
				main_frame.present(&engine, frame_index).and_then(|()|
				main_frame.execute_rendering(&engine, &framebuffer_commands, Some(&update_commands), Some(&debug_info), &execute_next_signal)))
			};

			// normal update
			input.update();
			let timescale = (1.0f32 + input[LogicalInputTypes::Slowdown] * 2.0f32) / (1.0f32 + input[LogicalInputTypes::Overdrive]);
			let delta_time_sec = (delta_time.num_milliseconds() as f32 / 1000.0f32) / timescale;
			secs_from_last_fixed += delta_time_sec;
			background_datastore.update(&mut randomizer, delta_time_sec, background_next_appear);

			if enemy_next_appear
			{
				if Enemy::new(&enemy_datastore, enemy_left_range.ind_sample(&mut randomizer)).map(|e| enemy_entities.push_back(e)) == None
				{
					warn!("Enemy Datastore is full!!");
				}
				else { *enemy_count.borrow_mut() += 1; }
				enemy_next_appear = false;
			}
			fn process_2<'a, F>(mut livings: LinkedList<Enemy<'a>>, mut purged_after: LinkedList<Enemy<'a>>,
				enemy_decrease_cb: F, delta_time_sec: f32) -> LinkedList<Enemy<'a>> where F: Fn()
			{
				if let Some(died_e) = purged_after.pop_front() { died_e.die(); }
				let mut purge_index: Option<usize> = None;
				for (idx, e) in purged_after.iter_mut().enumerate()
				{
					if e.update(delta_time_sec)
					{
						enemy_decrease_cb();
						purge_index = Some(idx);
						break;
					}
				}
				if let Some(purge_index) = purge_index
				{
					let mut purged_before = purged_after;
					let purged_after = purged_before.split_off(purge_index);
					livings.append(&mut purged_before);
					process_2(livings, purged_after, enemy_decrease_cb, delta_time_sec)
				}
				else
				{
					livings.append(&mut purged_after);
					livings
				}
			}
			let mut purge_index: Option<usize> = None;
			for (idx, e) in enemy_entities.iter_mut().enumerate()
			{
				if e.update(delta_time_sec)
				{
					*enemy_count.borrow_mut() -= 1;
					purge_index = Some(idx);
					break;
				}
			}
			if let Some(purge_index) = purge_index
			{
				let purged_after = enemy_entities.split_off(purge_index);
				enemy_entities = process_2(enemy_entities, purged_after, || { *enemy_count.borrow_mut() -= 1; }, delta_time_sec);
			}
			player.update(delta_time_sec, &input);

			background_next_appear = false;
			prev_time = time::PreciseTime::now();
		}

		if secs_from_last_fixed >= 1.0f32 / 60.0f32
		{
			// fixed update
			background_next_appear = background_appear_rate.ind_sample(&mut randomizer) == 0;
			enemy_next_appear = enemy_appear_rate.ind_sample(&mut randomizer) == 0;
			secs_from_last_fixed = 0.0f32;
		}
	}
	try!(engine.wait_device());

	Ok(())
}
