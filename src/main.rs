
extern crate nalgebra;
extern crate rand;
extern crate time;
extern crate thread_scoped;
extern crate glob;
extern crate rayon;
#[macro_use] extern crate log;

extern crate interlude;
extern crate texture_compression;
extern crate psdloader;

use interlude::ffi::*;
use interlude::traits::*;
use interlude::{InputKeys, InputAxis, InputType};
use texture_compression::*;
use psdloader::*;

mod constants;
use constants::*;
mod traits;
mod vertex_formats;
use vertex_formats::*;
mod structures;
use structures::*;
mod logical_resources;
mod utils;
use nalgebra::*;
use rand::distributions::*;

mod smaa_extra_textures;
use smaa_extra_textures::*;

use rayon::prelude::*;

use std::cell::RefCell;

// For InputSystem
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum LogicalInputTypes
{
	Horizontal, Vertical, Shoot, Slowdown, Overdrive
}

fn store_quaternion(to: &mut CVector4, q: &Quaternion<f32>)
{
	*to = [q.i, q.j, q.k, q.w];
}

enum Enemy<'a>
{
	Free, Entity
	{
		block_index: u32, uniform_ref: &'a mut CharacterLocation, rezonator_iref: &'a mut CVector4,
		left: f32, living_secs: f32, rezonator_left: u32
	}, Garbage(u32)
}
unsafe impl <'a> std::marker::Send for Enemy<'a> {}
impl <'a> Enemy<'a>
{
	pub fn init(init_left: f32, block_index: u32, uref: &'a mut structures::CharacterLocation, iref_rez: &'a mut structures::CVector4) -> Self
	{
		uref.center_tf = [init_left, 0.0, 0.0, 0.0];
		store_quaternion(&mut uref.rotq[0], UnitQuaternion::new(Vector3::new(0.0, 0.0, 0.0)).quaternion());
		store_quaternion(&mut uref.rotq[1], UnitQuaternion::new(Vector3::new(0.0, 0.0, 0.0)).quaternion());
		*iref_rez = [3.0, 0.0, 0.0, 0.0];

		Enemy::Entity
		{
			block_index: block_index, uniform_ref: uref, rezonator_iref: iref_rez,
			left: init_left, living_secs: 0.0f32, rezonator_left: 3
		}
	}
	pub fn update(&mut self, delta_time: f32)
	{
		// update values
		let died_bi = match self
		{
			&mut Enemy::Entity { block_index, ref mut uniform_ref, ref mut rezonator_iref, left: _, ref mut living_secs, rezonator_left } =>
			{
				let current_y = if *living_secs < 0.875f32
				{
					15.0f32 * (1.0f32 - (1.0f32 - *living_secs / 0.875f32).powi(2)) - 3.0f32
				}
				else
				{
					15.0f32 + (*living_secs - 0.875f32) * 2.5f32 - 3.0f32
				};
				uniform_ref.center_tf[1] = current_y;
				store_quaternion(&mut uniform_ref.rotq[0], UnitQuaternion::new(Vector3::new(-1.0, 0.0, 0.75).normalize() * (260.0 * *living_secs).to_radians()).quaternion());
				store_quaternion(&mut uniform_ref.rotq[1], UnitQuaternion::new(Vector3::new(1.0, -1.0, 0.5).normalize() * (-260.0 * *living_secs + 13.0).to_radians()).quaternion());
				rezonator_iref[0] = rezonator_left as f32;
				rezonator_iref[1] -= 130.0f32.to_radians() * delta_time;
				rezonator_iref[2] += 220.0f32.to_radians() * delta_time;
				*living_secs += delta_time;

				if current_y >= 52.0 { rezonator_iref[0] = 0.0; Some(block_index) } else { None }
			},
			_ => None
		};

		// state change
		if let Some(bindex) = died_bi { *self = Enemy::Garbage(bindex); }
	}
	pub fn is_garbage(&self) -> bool
	{
		match self { &Enemy::Garbage(_) => true, _ => false }
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

pub struct WireRenderCommon<'a>
{
	renderstate_ref: &'a interlude::GraphicsPipeline, layout_ref: &'a interlude::PipelineLayout
}
impl <'a> WireRenderCommon<'a>
{
	pub fn new(renderstate: &'a interlude::GraphicsPipeline, layout: &'a interlude::PipelineLayout) -> Self
	{
		WireRenderCommon { renderstate_ref: renderstate, layout_ref: layout }
	}
	pub fn begin<Recorder: DrawingCommandRecorder>(&self, comrec: Recorder, wirecolor_r: f32, wirecolor_g: f32, wirecolor_b: f32, wirecolor_a: f32) -> Recorder
	{
		comrec.bind_pipeline(self.renderstate_ref).push_constants(self.layout_ref, &[interlude::ShaderStage::Vertex],
			0 .. std::mem::size_of::<structures::CVector4>() as u32, &[wirecolor_r, wirecolor_g, wirecolor_b, wirecolor_a])
	}
}

fn pack_color(src: DecompressedPSDImageData) -> Vec<u8>
{
	let mut color_pixels = Vec::new();
	for (x, y) in (0 .. src.height).flat_map(|y| (0 .. src.width).map(move |x| (x, y)))
	{
		for c in 0 .. src.channels
		{
			color_pixels.push(src.fetch(x, y, c));
		}
	}
	color_pixels
}

fn main() { if let Err(e) = app_main() { interlude::crash(e); } }
fn app_main() -> Result<(), interlude::EngineError>
{
	utils::memory_management_test();

	let engine = try!{
		interlude::Engine::new_with_features("HardGrad->Extent", 0x01, interlude::DeviceFeatures::new().enable_block_texture_compression())
			.map(|e| e.with_assets_in(std::env::current_dir().unwrap()))
	};
	let main_frame = try!(engine.create_render_window(VkExtent2D(640, 480), "HardGrad -> Extent"));
	let VkExtent2D(frame_width, frame_height) = main_frame.get_extent();
	let execute_next_signal = try!(engine.create_fence());

	let playerbullet_image = PhotoshopDocument::open(engine.parse_asset("graphs.playerbullet", "psd")).unwrap();
	let gbuffer_desc = interlude::ImageDescriptor2::new(VkFormat::R8G8B8A8_UNORM, main_frame.get_extent(), interlude::ImageUsagePresets::AsColorTexture).device_resource();
	let edgebuffer_desc = interlude::ImageDescriptor2::new(VkFormat::R8G8_UNORM, main_frame.get_extent(), interlude::ImageUsagePresets::AsColorTexture).device_resource();
	let blend_weight_desc = interlude::ImageDescriptor2::new(VkFormat::R8G8B8A8_UNORM, main_frame.get_extent(), interlude::ImageUsagePresets::AsColorTexture).device_resource();
	let smaa_areatex_desc = interlude::ImageDescriptor2::new(VkFormat::BC5_UNORM_BLOCK, VkExtent2D(AREATEX_WIDTH, AREATEX_HEIGHT), VK_IMAGE_USAGE_SAMPLED_BIT);
	let smaa_searchtex_desc = interlude::ImageDescriptor2::new(VkFormat::BC4_UNORM_BLOCK, VkExtent2D(SEARCHTEX_WIDTH, SEARCHTEX_HEIGHT), VK_IMAGE_USAGE_SAMPLED_BIT);
	let playerbullet_desc = interlude::ImageDescriptor2::new(VkFormat::R8G8B8A8_UNORM, VkExtent2D(playerbullet_image.width as u32, playerbullet_image.height as u32), interlude::ImageUsagePresets::AsColorTexture);
	let imagebuffer_placement = interlude::ImagePreallocator::new().image_2d(vec![&gbuffer_desc, &edgebuffer_desc, &blend_weight_desc, &smaa_areatex_desc, &smaa_searchtex_desc, &playerbullet_desc]);
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
	let playerbullet_view = try!(engine.create_image_view_2d(backbuffers.dim2(5), VkFormat::R8G8B8A8_UNORM,
		interlude::ComponentMapping::straight(), interlude::ImageSubresourceRange::base_color()));
	let gbuffer_sampler = try!(engine.create_sampler(&interlude::SamplerState::new()));
	try!(stage_images.map().map(|mapped|
	{
		let areatex_compressed = BC5::compress(&AREATEX_BYTES, (AREATEX_WIDTH as usize, AREATEX_HEIGHT as usize));
		mapped.map_mut::<[u8; AREATEX_SIZE as usize / 2]>(stage_images.image2d_offset(0) as usize).copy_from_slice(&areatex_compressed);
		// mapped.map_mut::<[u8; AREATEX_SIZE as usize]>(stage_images.image2d_offset(0) as usize).copy_from_slice(&AREATEX_BYTES);
		let searchtex_compressed = BC4::compress(&SEARCHTEX_BYTES, (SEARCHTEX_WIDTH as usize, SEARCHTEX_HEIGHT as usize));
		mapped.map_mut::<[u8; SEARCHTEX_SIZE as usize / 2]>(stage_images.image2d_offset(1) as usize).copy_from_slice(&searchtex_compressed);
		// mapped.map_mut::<[u8; SEARCHTEX_SIZE as usize]>(stage_images.image2d_offset(1) as usize).copy_from_slice(&SEARCHTEX_BYTES);
		let playerbullet_pixels = pack_color(playerbullet_image.combined_raw_image_data());
		mapped.range_mut::<u8>(stage_images.image2d_offset(2) as usize, playerbullet_image.width * playerbullet_image.height * 4).copy_from_slice(&playerbullet_pixels);
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
		vertices.enemy_rezonator_vts = [
			Position(0.0f32, 1.0f32, 0.0f32, 1.0f32),
			Position(-1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position(1.0f32, -1.0f32, 0.0f32, 1.0f32)
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
	let enemy_rezonator_dupv = try!(engine.create_vertex_shader_from_asset("shaders.EnemyRezonatorV", "main", &[
		interlude::VertexBinding::PerVertex(std::mem::size_of::<vertex_formats::Position>() as u32),
		interlude::VertexBinding::PerInstance(std::mem::size_of::<structures::CVector4>() as u32)
	], &[interlude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0), interlude::VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)]));
	let player_rotor_vert = try!(engine.create_vertex_shader_from_asset("shaders.PlayerRotor", "main", &[
		interlude::VertexBinding::PerVertex(std::mem::size_of::<vertex_formats::Position>() as u32),
		interlude::VertexBinding::PerInstance(std::mem::size_of::<structures::CVector4>() as u32)
	], &[interlude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0), interlude::VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)]));
	let backline_duplicator = try!(engine.create_geometry_shader_from_asset("shaders.BackLineDuplicator", "main"));
	let enemy_duplicator = try!(engine.create_geometry_shader_from_asset("shaders.EnemyDuplicator", "main"));
	let enemy_rezonator_duplicator = try!(engine.create_geometry_shader_from_asset("shaders.EnemyRezonatorDup", "main"));
	let through_color_frag = try!(engine.create_fragment_shader_from_asset("shaders.ThroughColor", "main"));
	let smaa_edge_ppv = try!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.EdgeDetectionV", "main"));
	let smaa_bw_ppv = try!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.BlendWeightCalcV", "main"));
	let smaa_combine_ppv = try!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.CombineV", "main"));
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
	let enemy_rezonators_render_state = interlude::GraphicsPipelineBuilder::inherit(&enemy_render_state)
		.vertex_shader(&enemy_rezonator_dupv).geometry_shader(&enemy_rezonator_duplicator)
		.primitive_topology(interlude::PrimitiveTopology::TriangleList(false));
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
	let pipeline_states = try!(engine.create_graphics_pipelines(&[
		&background_render_state, &enemy_render_state, &enemy_rezonators_render_state, &player_render_state,
		&pp_smaa_edge_detection_state, &pp_smaa_blend_weight_state, &pp_smaa_combine_state
	]));
	let background_render = WireRenderCommon::new(&pipeline_states[0], &wire_render_layout);
	let enemy_render = WireRenderCommon::new(&pipeline_states[1], &wire_render_layout);
	let enemy_rezonators_render = WireRenderCommon::new(&pipeline_states[2], &wire_render_layout);
	let player_render = WireRenderCommon::new(&pipeline_states[3], &wire_render_layout);
	let ref pp_smaa_edge_detection = pipeline_states[4];
	let ref pp_smaa_blend_weight_calc = pipeline_states[5];
	let ref pp_smaa_combine = pipeline_states[6];

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
			interlude::ImageMemoryBarrier::template(&**backbuffers.dim2(5), interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(stage_images.dim2(0), interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(stage_images.dim2(1), interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(stage_images.dim2(2), interlude::ImageSubresourceRange::base_color())
		];
		let image_memory_barriers = main_frame.get_back_images().iter()
			.map(|x| interlude::ImageMemoryBarrier::hold_ownership(*x, interlude::ImageSubresourceRange::base_color(),
			0, VK_ACCESS_MEMORY_READ_BIT, VkImageLayout::Undefined, VkImageLayout::PresentSrcKHR)).chain([
				interlude::ImageMemoryBarrier::hold_ownership(&**backbuffers.dim2(0), interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				interlude::ImageMemoryBarrier::hold_ownership(&**backbuffers.dim2(1), interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				interlude::ImageMemoryBarrier::hold_ownership(&**backbuffers.dim2(2), interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				blitted_image_templates[0].into_transfer_dst(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized),
				blitted_image_templates[1].into_transfer_dst(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized),
				blitted_image_templates[2].into_transfer_dst(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized),
				blitted_image_templates[3].into_transfer_src(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized),
				blitted_image_templates[4].into_transfer_src(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized),
				blitted_image_templates[5].into_transfer_src(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized)
			].into_iter().map(|&x| x)).collect::<Vec<_>>();
		let image_memory_barriers_ret =
		[
			blitted_image_templates[0].from_transfer_dst(VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal),
			blitted_image_templates[1].from_transfer_dst(VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal),
			blitted_image_templates[2].from_transfer_dst(VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal)
		];

		try!(setup_commands.begin(0).and_then(|recorder|
			recorder.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false,
				&[], &buffer_memory_barriers, &image_memory_barriers)
			.copy_buffer(&appdata_stage, &application_data, &[interlude::BufferCopyRegion(0, 0, application_buffer_prealloc.total_size() as usize)])
			.copy_image(stage_images.dim2(0), &**backbuffers.dim2(3), VkImageLayout::TransferSrcOptimal, VkImageLayout::TransferDestOptimal,
				&[interlude::ImageCopyRegion(interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), VkExtent3D(AREATEX_WIDTH, AREATEX_HEIGHT, 1))])
			.copy_image(stage_images.dim2(1), &**backbuffers.dim2(4), VkImageLayout::TransferSrcOptimal, VkImageLayout::TransferDestOptimal,
				&[interlude::ImageCopyRegion(interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), VkExtent3D(SEARCHTEX_WIDTH, SEARCHTEX_HEIGHT, 1))])
			.copy_image(stage_images.dim2(2), &**backbuffers.dim2(5), VkImageLayout::TransferSrcOptimal, VkImageLayout::TransferDestOptimal,
				&[interlude::ImageCopyRegion(interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), VkExtent3D(playerbullet_image.width as u32, playerbullet_image.height as u32, 1))])
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT, false, &[], &buffer_memory_barriers_ret, &image_memory_barriers_ret)
			.end()
		));

		try!(setup_commands.execute());
	}

	// Debug Information //
	let frame_time_ms = RefCell::new(0.0f64);
	let cputime_ms = RefCell::new(0.0f64);
	let enemy_count = RefCell::new(0u32);
	let debug_info = try!(interlude::DebugInfo::new(&engine, &[
		interlude::DebugLine::Float("Frame Time".to_owned(), &frame_time_ms, Some("ms".to_owned())),
		interlude::DebugLine::Float("CPU Time".to_owned(), &cputime_ms, Some("ms".to_owned())),
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
			.inject_commands(|r| background_render.begin(r, 0.125, 0.5, 0.1875, 0.625))
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::background_offs())])
			.draw(4, MAX_BK_COUNT as u32)
			.inject_commands(|r| enemy_render.begin(r, 0.25, 0.9875, 1.5, 1.0))
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3))])
			.draw(4, MAX_ENEMY_COUNT as u32)
			.inject_commands(|r| player_render.begin(r, 1.5, 1.25, 0.375, 1.0))
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::player_rot_offs())])
			.bind_index_buffer(&application_data, application_buffer_prealloc.offset(2))
			.draw_indexed(24, 2, 4)
			.inject_commands(|r| enemy_rezonators_render.begin(r, 1.25, 0.5, 0.625, 1.0))
			.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(1) + structures::VertexMemoryForWireRender::enemy_rezonator_offs()),
				(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::enemy_rez_offs())])
			.draw(3, MAX_ENEMY_COUNT as u32)
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
	let (uref_enemy, uref_bk, uref_player_center) =
	{
		let mapped = mapped_range.map_mut::<structures::UniformMemory>(application_buffer_prealloc.offset(4));
		(&mut mapped.enemy_instance_data, &mut mapped.background_instance_data, &mut mapped.player_center_tf)
	};
	let (iref_enemy, iref_bk, iref_player, iref_enemy_rez) =
	{
		let mapped = mapped_range.map_mut::<structures::InstanceMemory>(application_buffer_prealloc.offset(3));
		(&mut mapped.enemy_instance_mult, &mut mapped.background_instance_mult, &mut mapped.player_rotq, &mut mapped.enemy_rez_instance_data)
	};
	let mut background_datastore = logical_resources::BackgroundDatastore::new(uref_bk, iref_bk);
	let mut enemy_datastore = logical_resources::EnemyDatastore::new(iref_enemy);

	// double-buffered enemy entity list //
	let mut enemy_entities: [Enemy; MAX_ENEMY_COUNT] = unsafe { std::mem::uninitialized() };
	for n in 0 .. MAX_ENEMY_COUNT { enemy_entities[n] = Enemy::Free; }
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
			let cputime_start = time::PreciseTime::now();
			input.update();
			let timescale = (1.0f32 + input[LogicalInputTypes::Slowdown] * 2.0f32) / (1.0f32 + input[LogicalInputTypes::Overdrive]);
			let delta_time_sec = (delta_time.num_milliseconds() as f32 / 1000.0f32) / timescale;
			secs_from_last_fixed += delta_time_sec;
			background_datastore.update(&mut randomizer, delta_time_sec, background_next_appear);

			if enemy_next_appear
			{
				let block_index = enemy_datastore.allocate_block();
				if let Some(bindex) = block_index
				{
					let bindex = bindex as usize;
					enemy_entities[bindex] = unsafe
					{
						let uref_enemy_ptr = uref_enemy.as_mut_ptr();
						let iref_enemy_rez_ptr = iref_enemy_rez.as_mut_ptr();
						Enemy::init(enemy_left_range.ind_sample(&mut randomizer), bindex as u32,
							&mut *uref_enemy_ptr.offset(bindex as isize), &mut *iref_enemy_rez_ptr.offset(bindex as isize))
					};
					*enemy_count.borrow_mut() += 1;
				}
				else { warn!("Enemy Datastore is full!!"); }
				enemy_next_appear = false;
			}
			enemy_entities.par_iter_mut().for_each(|e| e.update(delta_time_sec));
			for e in enemy_entities.iter_mut().filter(|e| e.is_garbage())
			{
				match e { &mut Enemy::Garbage(bindex) => enemy_datastore.free_block(bindex), _ => unreachable!() };
				*e = Enemy::Free;
				*enemy_count.borrow_mut() -= 1;
			}
			player.update(delta_time_sec, &input);

			background_next_appear = false;
			prev_time = time::PreciseTime::now();
			*cputime_ms.borrow_mut() = cputime_start.to(time::PreciseTime::now()).num_microseconds().unwrap_or(0) as f64 / 1000.0f64;
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
