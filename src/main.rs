
extern crate nalgebra;
extern crate rand;
extern crate time;
extern crate thread_scoped;
extern crate glob;
extern crate rayon;
#[macro_use] extern crate log;
extern crate itertools;
#[macro_use] extern crate lazy_static;
extern crate half;

#[macro_use] extern crate interlude;
extern crate postludium;
extern crate texture_compression;
extern crate psdloader;

use interlude::*;
use interlude::ffi::*;
use texture_compression::*;
use psdloader::*;

mod constants;
use constants::*;
mod structures;
use structures::*;
mod logical_resources;
use logical_resources::*;
mod utils;
use rand::distributions::*;
use half::f16;
use itertools::Itertools;

use postludium::*;

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

fn pack_color(canvas_size: VkExtent2D, red: DecompressedChannelImageData, green: DecompressedChannelImageData,
	blue: DecompressedChannelImageData, alpha: DecompressedChannelImageData) -> Vec<u8>
{
	let VkExtent2D(cwidth, cheight) = canvas_size;
	let mut color_pixels = vec![0u8; (cwidth * cheight) as usize * 4];
	for (x, y, px, py) in (0 .. red.height()).flat_map(|y| (0 .. red.width()).map(move |x| (x, y)))
		.map(|(x, y)| (x, y, x as isize + red.offset_x(), y as isize + red.offset_y()))
		.filter(|&(_, _, px, py)| (0 <= px && px < cwidth as isize) && (0 <= py && py < cheight as isize))
	{
		color_pixels[(px + py * cwidth as isize) as usize * 4 + 0] = red.fetch(x, y);
	}
	for (x, y, px, py) in (0 .. green.height()).flat_map(|y| (0 .. green.width()).map(move |x| (x, y)))
		.map(|(x, y)| (x, y, x as isize + green.offset_x(), y as isize + green.offset_y()))
		.filter(|&(_, _, px, py)| (0 <= px && px < cwidth as isize) && (0 <= py && py < cheight as isize))
	{
		color_pixels[(px + py * cwidth as isize) as usize * 4 + 1] = green.fetch(x, y);
	}
	for (x, y, px, py) in (0 .. blue.height()).flat_map(|y| (0 .. blue.width()).map(move |x| (x, y)))
		.map(|(x, y)| (x, y, x as isize + blue.offset_x(), y as isize + blue.offset_y()))
		.filter(|&(_, _, px, py)| (0 <= px && px < cwidth as isize) && (0 <= py && py < cheight as isize))
	{
		color_pixels[(px + py * cwidth as isize) as usize * 4 + 2] = blue.fetch(x, y);
	}
	for (x, y, px, py) in (0 .. alpha.height()).flat_map(|y| (0 .. alpha.width()).map(move |x| (x, y)))
		.map(|(x, y)| (x, y, x as isize + alpha.offset_x(), y as isize + alpha.offset_y()))
		.filter(|&(_, _, px, py)| (0 <= px && px < cwidth as isize) && (0 <= py && py < cheight as isize))
	{
		color_pixels[(px + py * cwidth as isize) as usize * 4 + 3] = alpha.fetch(x, y);
	}
	// premultiply
	for (x, y) in (0 .. cheight as usize).flat_map(|y| (0 .. cwidth as usize).map(move |x| (x, y)))
	{
		let alpha_p = color_pixels[(x + y * cwidth as usize) * 4 + 3] as f32 / 255.0;
		color_pixels[(x + y * cwidth as usize) * 4 + 0] = (color_pixels[(x + y * cwidth as usize) * 4 + 0] as f32 * alpha_p) as u8;
		color_pixels[(x + y * cwidth as usize) * 4 + 1] = (color_pixels[(x + y * cwidth as usize) * 4 + 1] as f32 * alpha_p) as u8;
		color_pixels[(x + y * cwidth as usize) * 4 + 2] = (color_pixels[(x + y * cwidth as usize) * 4 + 2] as f32 * alpha_p) as u8;
	}
	color_pixels
}

mod assets;
use assets::*;
mod framebuffer;
mod renderstate;
use framebuffer::*;
use renderstate::*;

#[allow(dead_code)]
pub struct SMAAPipelineStates
{
	edgedetect_vshader: ShaderProgram, blendweight_vshader: ShaderProgram, combine_vshader: ShaderProgram,
	edgedetect_shader: ShaderProgram, blendweight_shader: ShaderProgram, combine_shader: ShaderProgram,
	descriptor_sets: [DescriptorSetLayout; 3],
	pub edgedetect_layout: PipelineLayout, pub blendweight_layout: PipelineLayout, pub combine_layout: PipelineLayout,
	pub edgedetect: GraphicsPipeline, pub blendweight_calc: GraphicsPipeline, pub combine: GraphicsPipeline
}
impl SMAAPipelineStates
{
	pub fn new(engine: &Engine, render_pass: &RenderPass, base_subpass: u32, processing_viewport: VkViewport) -> Self
	{
		let VkViewport(_, _, vw, vh, _, _) = processing_viewport;

		let evsh = Unrecoverable!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.EdgeDetectionV", "main"));
		let bwvsh = Unrecoverable!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.BlendWeightCalcV", "main"));
		let cvsh = Unrecoverable!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.CombineV", "main"));
		let esh = Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.smaa.EdgeDetection", "main"));
		let bwsh = Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.smaa.BlendWeightCalc", "main"));
		let csh = Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.smaa.Combine", "main"));

		let dss = [
			Unrecoverable!(engine.create_descriptor_set_layout(&[Descriptor::CombinedSampler(1, vec![ShaderStage::Fragment])])),
			Unrecoverable!(engine.create_descriptor_set_layout(&[Descriptor::CombinedSampler(3, vec![ShaderStage::Fragment])])),
			Unrecoverable!(engine.create_descriptor_set_layout(&[Descriptor::CombinedSampler(2, vec![ShaderStage::Fragment])]))
		];
		let epl = Unrecoverable!(engine.create_pipeline_layout(&[&dss[0]], &[]));
		let bwpl = Unrecoverable!(engine.create_pipeline_layout(&[&dss[1]], &[]));
		let cpl = Unrecoverable!(engine.create_pipeline_layout(&[&dss[2]], &[]));

		let scons_rt_metrics = vec![
			(0, ConstantEntry::Float(vw)),
			(1, ConstantEntry::Float(vh)),
			(2, ConstantEntry::Float(vw.recip())),
			(3, ConstantEntry::Float(vh.recip()))
		];
		let mut gps =
		{
			let eps = GraphicsPipelineBuilder::for_postprocess(engine, &epl, render_pass, base_subpass + 0,
				PipelineShaderProgram::unspecialized(&esh), processing_viewport)
				.vertex_shader(PipelineShaderProgram(&evsh, scons_rt_metrics.clone()));
			let bwps = GraphicsPipelineBuilder::for_postprocess(engine, &bwpl, render_pass, base_subpass + 1,
				PipelineShaderProgram(&bwsh, scons_rt_metrics.clone()), processing_viewport)
				.vertex_shader(PipelineShaderProgram(&bwvsh, scons_rt_metrics.clone()));
			let cps = GraphicsPipelineBuilder::for_postprocess(engine, &cpl, render_pass, base_subpass + 2,
				PipelineShaderProgram(&csh, scons_rt_metrics.clone()), processing_viewport)
				.vertex_shader(PipelineShaderProgram(&cvsh, scons_rt_metrics));
			Unrecoverable!(engine.create_graphics_pipelines(&[&eps, &bwps, &cps]))
		};
		let cpso = gps.pop().unwrap();
		let bwpso = gps.pop().unwrap();
		let epso = gps.pop().unwrap();
		assert_eq!(gps.len(), 0);

		SMAAPipelineStates
		{
			edgedetect_vshader: evsh, blendweight_vshader: bwvsh, combine_vshader: cvsh,
			edgedetect_shader: esh, blendweight_shader: bwsh, combine_shader: csh,
			descriptor_sets: dss, edgedetect_layout: epl, blendweight_layout: bwpl, combine_layout: cpl,
			edgedetect: epso, blendweight_calc: bwpso, combine: cpso
		}
	}
}

enum ApplicationEvent { Update, Exit }

fn main() { if let Err(e) = app_main() { interlude::crash(e); } }
fn app_main() -> Result<(), EngineError>
{
	let engine = try!(Engine::new("hardgrad_extend", 0x01, Some(std::env::current_dir().unwrap()), DeviceFeatures::new().enable_block_texture_compression()));
	let main_frame = try!(engine.create_render_window(VkExtent2D(640, 480), "HardGrad -> Extend"));
	let extent = main_frame.get_extent();
	game_main(*engine, main_frame, extent)
}
fn game_main(engine: Engine, target: Box<RenderWindow>, target_extent: VkExtent2D) -> Result<(), EngineError>
{
	// Resources //
	let images = DevConfImages::from_file(&engine, "devconf.images", target_extent, target.get_format()).ensure_has_staging();
	// Reference Bindings //
	let ref backbuffer_sfloat4_set = images.images_2d()[0];
	let ref backbuffer_unorm4f_set = images.images_2d()[1];
	let ref backbuffer_unorm2_set = images.images_2d()[2];
	let ref backbuffer_unorm4_set = images.images_2d()[3];
	let ref smaa_areatex_set = images.images_2d()[4];
	let ref smaa_searchtex_set = images.images_2d()[5];
	let ref playerbullet_tex_set = images.images_2d()[6];
	let ref lineburst_particle_gradient_tex_set = images.images_1d()[0];
	let ref gbuffer_sampler = images.samplers()[0];
	let ref lineburst_particle_gradient_tex_stg = images.staging_images()[0];
	let ref smaa_areatex_stg = images.staging_images()[1];
	let ref smaa_searchtex_stg = images.staging_images()[2];
	let ref playerbullet_tex_stg = images.staging_images()[3];

	let playerbullet_image = PhotoshopDocument::open(engine.parse_asset("graphs.playerbullet", "psd")).unwrap();
	{
		let mapped = images.map_staging_images_memory();
		let offsets = images.staging_offsets();
		let areatex_compressed = BC5::compress(&AREATEX_BYTES, (AREATEX_WIDTH, AREATEX_HEIGHT));
		mapped.map_mut::<[u8; AREATEX_SIZE / 2]>(offsets[1] as usize).copy_from_slice(&areatex_compressed);
		let searchtex_compressed = BC4::compress(&SEARCHTEX_BYTES, (SEARCHTEX_WIDTH, SEARCHTEX_HEIGHT));
		mapped.map_mut::<[u8; SEARCHTEX_SIZE / 2]>(offsets[2] as usize).copy_from_slice(&searchtex_compressed);

		let playerbullet_pixels = pack_color(
			VkExtent2D(playerbullet_image.width as u32, playerbullet_image.height as u32),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Red),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Green),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Blue),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Alpha)
		);
		mapped.range_mut::<u8>(offsets[3] as usize, 16 * 16 * 4).copy_from_slice(&playerbullet_pixels);
		mapped.map_mut::<[[f16; 4]; 4]>(offsets[0] as usize).copy_from_slice(&[
			[f16::from_f64(2.0), f16::from_f64(1.5), f16::from_f64(1.0), f16::from_f64(1.0)],
			[f16::from_f64(1.5), f16::from_f64(1.0), f16::from_f64(0.25), f16::from_f64(1.0)],
			[f16::from_f64(1.0), f16::from_f64(0.1875), f16::from_f64(0.125), f16::from_f64(0.875)],
			[f16::from_f64(0.25), f16::from_f64(0.25), f16::from_f64(0.25), f16::from_f64(0.375)]
		]);
	}
	let appdata = ApplicationBufferData::new(&engine, target_extent);

	let render_pass = RenderPasses::new(&engine, target.get_format());
	let framebuffers = target.get_back_images().iter().map(|&finalbuffer| 
		engine.create_framebuffer(&render_pass.object, &[backbuffer_sfloat4_set, backbuffer_unorm4f_set, backbuffer_unorm2_set, backbuffer_unorm4_set, finalbuffer], VkExtent3D::from(target_extent))
	).collect::<Result<Vec<_>, _>>().or_crash();

	// Pipelines //
	let sc_viewport = VkViewport::from(target_extent);
	let pipelines = PipelineStates::new(&engine, true, &render_pass, sc_viewport);

	// Descriptor Set //
	let uniform_memory_info = BufferInfo(&appdata.dev, appdata.offset_uniform() .. appdata.size());
	let backbuffer_unorm4f_info = ImageInfo(gbuffer_sampler, backbuffer_unorm4f_set, VkImageLayout::ShaderReadOnlyOptimal);
	let backbuffer_unorm2_info = ImageInfo(gbuffer_sampler, backbuffer_unorm2_set, VkImageLayout::ShaderReadOnlyOptimal);
	let backbuffer_unorm4_info = ImageInfo(gbuffer_sampler, backbuffer_unorm4_set, VkImageLayout::ShaderReadOnlyOptimal);
	let areatex_info = ImageInfo(gbuffer_sampler, smaa_areatex_set, VkImageLayout::ShaderReadOnlyOptimal);
	let searchtex_info = ImageInfo(gbuffer_sampler, smaa_searchtex_set, VkImageLayout::ShaderReadOnlyOptimal);
	let playerbullet_info = ImageInfo(gbuffer_sampler, playerbullet_tex_set, VkImageLayout::ShaderReadOnlyOptimal);
	let lineburst_particle_gradient_tex_info = ImageInfo(gbuffer_sampler, lineburst_particle_gradient_tex_set, VkImageLayout::ShaderReadOnlyOptimal);
	engine.update_descriptors(&[
		DescriptorSetWriteInfo::UniformBuffer(pipelines.get_descriptor_set_for_uniform_buffer(), 0, vec![uniform_memory_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_smaa_edgedetect(), 0, vec![backbuffer_unorm4f_info.clone()]),
		DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_smaa_blendweight(), 0, vec![backbuffer_unorm2_info, areatex_info, searchtex_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_smaa_combine(), 0, vec![backbuffer_unorm4f_info, backbuffer_unorm4_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_playerbullet_texture(), 0, vec![playerbullet_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_lineburst_particle_color(), 0, vec![lineburst_particle_gradient_tex_info])
	]);

	// Initial Data Transmission, Layouting for Swapchain Backbuffer Images //
	engine.allocate_transient_transfer_command_buffers(1).and_then(|setup_commands|
	{
		let buffer_memory_barriers = [
			BufferMemoryBarrier::hold_ownership(&appdata.stg, 0 .. appdata.size(), 0, VK_ACCESS_TRANSFER_READ_BIT),
			BufferMemoryBarrier::hold_ownership(&appdata.dev, 0 .. appdata.size(), 0, VK_ACCESS_TRANSFER_WRITE_BIT)
		];
		let buffer_memory_barriers_ret = [
			BufferMemoryBarrier::hold_ownership(&appdata.stg, 0 .. appdata.size(), VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_MEMORY_READ_BIT),
			BufferMemoryBarrier::hold_ownership(&appdata.dev, 0 .. appdata.size(),
				VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT)
		];
		let blitted_image_templates_dev = vec![
			ImageMemoryBarrier::template(smaa_areatex_set, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(smaa_searchtex_set, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(playerbullet_tex_set, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(lineburst_particle_gradient_tex_set, ImageSubresourceRange::base_color())
		];
		let blitted_image_templates_stg = vec![
			ImageMemoryBarrier::template(smaa_areatex_stg, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(smaa_searchtex_stg, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(playerbullet_tex_stg, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(lineburst_particle_gradient_tex_stg, ImageSubresourceRange::base_color())
		];
		let image_memory_barriers = target.get_back_images().iter()
			.map(|x| ImageMemoryBarrier::hold_ownership(*x, ImageSubresourceRange::base_color(),
				0, VK_ACCESS_MEMORY_READ_BIT, VkImageLayout::Undefined, VkImageLayout::PresentSrcKHR))
			.chain(vec![
				ImageMemoryBarrier::hold_ownership(backbuffer_sfloat4_set, ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				ImageMemoryBarrier::hold_ownership(backbuffer_unorm4f_set, ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				ImageMemoryBarrier::hold_ownership(backbuffer_unorm2_set, ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				ImageMemoryBarrier::hold_ownership(backbuffer_unorm4_set, ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal)
			]).chain(blitted_image_templates_dev.iter().map(|t| t.into_transfer_dst(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized)))
			.chain(blitted_image_templates_stg.into_iter().map(|t| t.into_transfer_src(VK_ACCESS_HOST_WRITE_BIT, VkImageLayout::Preinitialized))).collect_vec();
		let image_memory_barriers_ret = blitted_image_templates_dev.into_iter()
			.map(|t| t.from_transfer_dst(VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal)).collect_vec();

		try!(setup_commands.begin(0).and_then(|recorder| recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false,
				&[], &buffer_memory_barriers, &image_memory_barriers)
			.copy_buffer(&appdata.stg, &appdata.dev, &[BufferCopyRegion(0, 0, appdata.size())])
			.copy_image(smaa_areatex_stg, smaa_areatex_set, &[ImageCopyRegion::entire_colorbits(VkExtent3D(AREATEX_WIDTH as u32, AREATEX_HEIGHT as u32, 1))])
			.copy_image(smaa_searchtex_stg, smaa_searchtex_set, &[ImageCopyRegion::entire_colorbits(VkExtent3D(SEARCHTEX_WIDTH as u32, SEARCHTEX_HEIGHT as u32, 1))])
			.copy_image(playerbullet_tex_stg, playerbullet_tex_set,
				&[ImageCopyRegion::entire_colorbits(VkExtent3D(playerbullet_image.width as u32, playerbullet_image.height as u32, 1))])
			.copy_image(lineburst_particle_gradient_tex_stg, lineburst_particle_gradient_tex_set, &[ImageCopyRegion::entire_colorbits(VkExtent3D(4, 1, 1))])
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT, false,
				&[], &buffer_memory_barriers_ret, &image_memory_barriers_ret)
			.end()
		));
		setup_commands.execute()
	}).or_crash();

	// Debug Information //
	let frame_time_ms = RefCell::new(0.0f64);
	let cputime_ms = RefCell::new(0.0f64);
	let enemy_count = RefCell::new(0u32);
	let debug_info = try!(interlude::DebugInfo::new(&engine, &[
		interlude::DebugLine::Float("Frame Time".to_owned(), &frame_time_ms, Some("ms".to_owned())),
		interlude::DebugLine::Float("CPU Time".to_owned(), &cputime_ms, Some("ms".to_owned())),
		interlude::DebugLine::UnsignedInt("Enemy Count".to_owned(), &enemy_count, None)
	], &render_pass.object, render_pass.smaa_combine_pass, sc_viewport));

	info!("Recording Rendering Commands...");
	// Rendering Commands //
	let combine_commands = 
	{
		let smaa_combine_descriptor_sets = [pipelines.get_descriptor_set_for_smaa_combine()];
		let smaa_combine_vertex_buffers = [(&appdata.dev as &BufferResource, appdata.offset_ppvbuf())];
		let combine_commands = try!(engine.allocate_bundled_command_buffers(2 * framebuffers.len() as u32));
		for (n, f) in framebuffers.iter().enumerate()
		{
			try!(combine_commands.begin(0 + 2 * n, &render_pass.object, 3, f).and_then(|recorder|
				recorder
					.bind_pipeline(&pipelines.smaa.as_ref().unwrap().combine)
					.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().combine_layout, &smaa_combine_descriptor_sets)
					.bind_vertex_buffers(&smaa_combine_vertex_buffers)
					.draw(4, 1)
				.end()
			));
			/*try!(combine_commands.begin(1 + 2 * n, &render_pass.object, 3, f).and_then(|recorder|
				recorder.inject_commands(|r| debug_info.inject_render_commands(r)).end()
			));*/
		}
		Some(combine_commands)
	};
	let framebuffer_commands = try!(engine.allocate_graphics_command_buffers(target.get_back_images().len() as u32));
	try!(framebuffer_commands.begin_all().map(|iter| iter.map(|(i, recorder)|
	{
		let clear_values = [
			interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.015625f32, 1.0f32),
			interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.0f32, 0.0f32),
			interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.0f32, 0.0f32),
			interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.0f32, 0.0f32)
		];
		let color_output_barrier = interlude::ImageMemoryBarrier::template(target.get_back_images()[i], interlude::ImageSubresourceRange::base_color())
			.hold_ownership(VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::PresentSrcKHR, VkImageLayout::ColorAttachmentOptimal);

		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, false, &[], &[], &[color_output_barrier])
			.begin_render_pass(&framebuffers[i], &clear_values, false)
			// Pass 0 : Render to Buffer //
			.bind_descriptor_sets(pipelines.layout_for_wire_render(), &[pipelines.get_descriptor_set_for_uniform_buffer()])
			.bind_vertex_buffers(&[(&appdata.dev, appdata.offset_vbuf())])
			.inject_commands(|r| pipelines.background.begin(r, 0.125, 0.5, 0.1875, 0.625))
			.bind_vertex_buffers_partial(1, &[(&appdata.dev, appdata.offset_instance() + InstanceMemory::background_offs())])
			.draw(4, MAX_BK_COUNT as u32)
			.inject_commands(|r| pipelines.enemy_body.begin(r, 0.25, 0.9875, 1.5, 1.0))
			.bind_vertex_buffers_partial(1, &[(&appdata.dev, appdata.offset_instance())])
			.draw(4, MAX_ENEMY_COUNT as u32)
			.inject_commands(|r| pipelines.player.begin(r, 1.5, 1.25, 0.375, 1.0))
			.bind_vertex_buffers_partial(1, &[(&appdata.dev, appdata.offset_instance() + InstanceMemory::player_rot_offs())])
			.bind_index_buffer(&appdata.dev, appdata.offset_ibuf())
			.draw_indexed(24, 2, 4)
			.inject_commands(|r| pipelines.enemy_rezonator.begin(r, 1.25, 0.5, 0.625, 1.0))
			.bind_vertex_buffers(&[(&appdata.dev, appdata.offset_vbuf() + VertexMemoryForWireRender::enemy_rezonator_offs()),
				(&appdata.dev, appdata.offset_instance() + InstanceMemory::enemy_rez_offs())])
			.draw(3, MAX_ENEMY_COUNT as u32)
			.inject_commands(|r| pipelines.playerbullet.begin(r, pipelines.get_descriptor_set_for_playerbullet_texture()))
			.bind_vertex_buffers(&[
				(&appdata.dev, appdata.offset_vbuf() + VertexMemoryForWireRender::sprite_plane_offs()),
				(&appdata.dev, appdata.offset_instance() + InstanceMemory::player_bullet_offs())
			])
			.draw(4, MAX_PLAYER_BULLET_COUNT as u32)
			.bind_pipeline(&pipelines.lineburst)
			.bind_descriptor_sets_partial(&pipelines.layout_for_lineburst_particle_render(), 1, &[pipelines.get_descriptor_set_for_lineburst_particle_color()])
			.bind_vertex_buffers(&[(&appdata.dev, appdata.offset_instance() + structures::InstanceMemory::lbparticle_groups_offs())])
			.draw(MAX_LBPARTICLE_GROUPS as u32, 1)
			.next_subpass(false)
			// Tonemapping //
			.bind_vertex_buffers(&[(&appdata.dev, appdata.offset_ppvbuf())])
			.bind_pipeline(&pipelines.tonemapper)
			.draw(4, 1)
			.next_subpass(false)
			// Edge Detection(SMAA 1x) //
			.bind_vertex_buffers(&[(&appdata.dev, appdata.offset_ppvbuf())])
			.bind_pipeline(&pipelines.smaa.as_ref().unwrap().edgedetect)
			.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().edgedetect_layout, &[pipelines.get_descriptor_set_for_smaa_edgedetect()])
			.draw(4, 1)
			// .pipeline_barrier(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, false, &[], &[], &[ibar_edgebuffer_end])
			.next_subpass(false)
			// Blend Weight Calculation(SMAA 1x) //
			.bind_pipeline(&pipelines.smaa.as_ref().unwrap().blendweight_calc)
			.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().blendweight_layout, &[pipelines.get_descriptor_set_for_smaa_blendweight()])
			.draw(4, 1)
			// .pipeline_barrier(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, false, &[], &[], &[ibar_blendweight_end])
			.next_subpass(true)
			// SMAA Combine and Debug Print //
			.execute_commands(&combine_commands.as_ref().unwrap()[i * 2 .. i * 2 + 1])
			.end_render_pass()
		.end().or_crash()
	}).collect::<Vec<_>>()));
	info!("Recording Transfer Commands...");
	// Transfer Commands //
	let update_commands = try!(engine.allocate_transfer_command_buffers(1));
	try!(update_commands.begin(0).and_then(|recorder|
	{
		let uoffs = appdata.offset_instance();
		let buffer_barriers = [
			interlude::BufferMemoryBarrier::hold_ownership(&appdata.dev, uoffs .. appdata.size(),
				VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT, VK_ACCESS_TRANSFER_WRITE_BIT),
			interlude::BufferMemoryBarrier::hold_ownership(&appdata.stg, uoffs .. appdata.size(),
				VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_TRANSFER_READ_BIT)
		];
		let buffer_barriers_ret = [
			interlude::BufferMemoryBarrier::hold_ownership(&appdata.dev, uoffs .. appdata.size(),
				VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT),
			interlude::BufferMemoryBarrier::hold_ownership(&appdata.stg, uoffs .. appdata.size(),
				VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_MEMORY_READ_BIT)
		];

		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false, &[], &buffer_barriers, &[])
			.copy_buffer(&appdata.stg, &appdata.dev, &[interlude::BufferCopyRegion(uoffs, uoffs, appdata.size() - uoffs)])
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, false, &[], &buffer_barriers_ret, &[])
		.end()
	}));

	info!("Preparing for Render Loop...");

	let _/*engine*/ = {
		let window_system = engine.window_system_ref().clone();
		let exit_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
		let exit_flag_uo = exit_flag.clone();
		let execute_next_signal = Unrecoverable!(engine.create_fence());
		let copy_completion_sig = Unrecoverable!(engine.create_fence());
		let dbg_copy_completion_sig = Unrecoverable!(engine.create_fence());
		let rendering_order_sem = Unrecoverable!(engine.create_queue_fence());
		let debug_transfer_commands = debug_info.get_transfer_commands();
		let (event_sender, event_receiver) = std::sync::mpsc::channel();
		let event_sender2 = event_sender.clone();
		let update_observer = unsafe { thread_scoped::scoped(move ||
		{
			let framebuffer_commands = framebuffer_commands;
			let update_commands = update_commands;
			let mut frame_index = Unrecoverable!(
				target.acquire_next_backbuffer_index(&rendering_order_sem).and_then(|findex|
					engine.submit_graphics_commands(&framebuffer_commands[findex as usize .. findex as usize + 1],
						&[(&rendering_order_sem, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)],
						None, Some(&execute_next_signal)).map(|()| findex)
				)
			);
			while !exit_flag_uo.load(std::sync::atomic::Ordering::Acquire)
			{
				Unrecoverable!(execute_next_signal.wait());
				Unrecoverable!(execute_next_signal.clear());
				Unrecoverable!(engine.submit_transfer_commands(&debug_transfer_commands, &[], None, Some(&dbg_copy_completion_sig)));
				Unrecoverable!(engine.submit_transfer_commands(&update_commands, &[], None, Some(&copy_completion_sig)));
				Unrecoverable!(copy_completion_sig.wait());  Unrecoverable!(dbg_copy_completion_sig.wait());
				Unrecoverable!(copy_completion_sig.clear()); Unrecoverable!(dbg_copy_completion_sig.clear());
				event_sender.send(ApplicationEvent::Update).unwrap();
				frame_index = Unrecoverable!(
					target.present(engine.graphics_queue_ref(), frame_index).and_then(|()|
					target.acquire_next_backbuffer_index(&rendering_order_sem).and_then(|findex|
					{
						engine.submit_graphics_commands(&framebuffer_commands[findex as usize .. findex as usize + 1],
							&[(&rendering_order_sem, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)],
							None, Some(&execute_next_signal)).map(|()| findex)
					}))
				);
			}

			Unrecoverable!(engine.wait_device());
			engine
		}) };
		let ws_event_observer = unsafe { thread_scoped::scoped(move ||
		{
			window_system.process_all_events();
			event_sender2.send(ApplicationEvent::Exit).unwrap();
		}) };

		let mapped_range = try!(appdata.stg.map());
		let (uref_enemy, uref_bk, uref_player_center, uref_gametime, uref_particle_infos) =
		{
			let mapped = mapped_range.map_mut::<UniformMemory>(appdata.offset_uniform());
			(&mut mapped.enemy_instance_data, &mut mapped.background_instance_data, &mut mapped.player_center_tf,
				&mut mapped.gametime, &mut mapped.lineburst_particles)
		};
		let (iref_enemy, iref_bk, iref_player, iref_enemy_rez, iref_player_bullet, iref_lineburst_particle_groups) =
		{
			let mapped = mapped_range.map_mut::<InstanceMemory>(appdata.offset_instance());
			(&mut mapped.enemy_instance_mult, &mut mapped.background_instance_mult, &mut mapped.player_rotq,
				&mut mapped.enemy_rez_instance_data, &mut mapped.player_bullet_offset_sincos, &mut mapped.lineburst_particle_groups)
		};
		let mut background_datastore = logical_resources::BackgroundDatastore::new(uref_bk, iref_bk);
		let mut enemy_datastore = EnemyDatastore::new(iref_enemy);
		let mut pb_memory_manager = utils::MemoryBlockManager::new(MAX_PLAYER_BULLET_COUNT as u32);
		let mut lineburst_particles = LineBurstParticles::new(iref_lineburst_particle_groups, uref_particle_infos);

		// double-buffered enemy entity list //
		let mut enemy_entities: [Enemy; MAX_ENEMY_COUNT] = unsafe { std::mem::uninitialized() };
		for n in 0 .. MAX_ENEMY_COUNT { enemy_entities[n] = Enemy::Free; }
		let mut player = Player::new(uref_player_center, iref_player);
		let mut player_bullets: [PlayerBullet; MAX_PLAYER_BULLET_COUNT] = unsafe { std::mem::uninitialized() };
		for n in 0 .. MAX_PLAYER_BULLET_COUNT { player_bullets[n] = PlayerBullet::Free; }

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
		let mut shooting = false;
		let mut secs_from_last_trigger = 0.0;
		let mut game_secs = 0.0;
		let mut next_shoot = false;
		let mut next_particle_spawn = None;
		let particle_spawn_rate = rand::distributions::Range::new(0, 30);
		let particle_spawn_count = rand::distributions::Range::new(1, 8);
		let particle_spawn_wrange = rand::distributions::Range::new(-30.0, 30.0);
		let particle_spawn_hrange = rand::distributions::Range::new(0.0, 50.0);
		while let Ok(event) = event_receiver.recv()
		{
			match event
			{
				ApplicationEvent::Exit => break,
				ApplicationEvent::Update =>
				{
					let delta_time = prev_time.to(time::PreciseTime::now());
					*frame_time_ms.borrow_mut() = delta_time.num_microseconds().unwrap_or(-1) as f64 / 1000.0f64;

					// normal update
					let cputime_start = time::PreciseTime::now();
					input.update();
					let timescale = (1.0f32 + input[LogicalInputTypes::Slowdown] * 2.0f32) / (1.0f32 + input[LogicalInputTypes::Overdrive]);
					let delta_time_sec = (delta_time.num_microseconds().unwrap() as f32 / 1_000_000.0f32) / timescale;
					secs_from_last_fixed += delta_time_sec;
					secs_from_last_trigger += delta_time_sec;
					game_secs += delta_time_sec;
					uref_gametime[0] = game_secs;
					background_datastore.update(&mut randomizer, delta_time_sec, background_next_appear);

					let new_shooting = input[LogicalInputTypes::Shoot] > 0.0;
					next_shoot = if !shooting && new_shooting
					{
						// start timer
						secs_from_last_trigger = delta_time_sec;
						shooting = true;
						true
					} else if shooting && !new_shooting
					{
						// stop timer
						shooting = false;
						false
					} else { next_shoot };
					if next_shoot
					{
						let winder_angle_abs = (game_secs * std::f32::consts::PI).sin() * 25.0;
						for a in -1 .. 2
						{
							let block_index = pb_memory_manager.allocate();
							if let Some(bindex) = block_index
							{
								let bindex = bindex as usize;
								player_bullets[bindex] = unsafe
								{
									let iref_player_bullet_ref_ptr = iref_player_bullet.as_mut_ptr();
									PlayerBullet::init(player.left(), player.top(), winder_angle_abs * a as f32, bindex as u32,
										&mut *iref_player_bullet_ref_ptr.offset(bindex as isize))
								};
							}
							else { warn!("Player Bullet Datastore is full!!"); }
						}
						next_shoot = false;
					}
					player_bullets.par_iter_mut().for_each(|e| e.update(delta_time_sec));
					for e in player_bullets.iter_mut().filter(|e| e.is_garbage())
					{
						match e { &mut PlayerBullet::Garbage(bindex) => pb_memory_manager.free(bindex), _ => unreachable!() };
						*e = PlayerBullet::Free;
					}

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

					if let Some((count, x, y)) = next_particle_spawn
					{
						lineburst_particles.spawn(count, x, y, game_secs);
						next_particle_spawn = None;
					}
					lineburst_particles = lineburst_particles.garbage_collect(game_secs);

					background_next_appear = false;
					prev_time = time::PreciseTime::now();
					*cputime_ms.borrow_mut() = cputime_start.to(time::PreciseTime::now()).num_microseconds().unwrap_or(0) as f64 / 1000.0f64;
					debug_info.update();
				}
			}

			if secs_from_last_fixed >= 1.0 / 60.0
			{
				// fixed update
				background_next_appear = background_appear_rate.ind_sample(&mut randomizer) == 0;
				enemy_next_appear = enemy_appear_rate.ind_sample(&mut randomizer) == 0;
				if particle_spawn_rate.ind_sample(&mut randomizer) == 0
				{
					next_particle_spawn = Some((particle_spawn_count.ind_sample(&mut randomizer),
						particle_spawn_wrange.ind_sample(&mut randomizer), particle_spawn_hrange.ind_sample(&mut randomizer)));
				}
				secs_from_last_fixed -= 1.0 / 60.0;
			}
			if shooting && secs_from_last_trigger >= 0.0375
			{
				next_shoot = true;
				secs_from_last_trigger -= 0.0375;
			}
		}

		ws_event_observer.join();
		exit_flag.store(true, std::sync::atomic::Ordering::Release);
		update_observer.join()
	};

	Ok(())
}
