
extern crate nalgebra;
extern crate rand;
extern crate time;
extern crate thread_scoped;
extern crate glob;
extern crate rayon;
#[macro_use] extern crate log;
extern crate itertools;
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
use logical_resources::enemy::*;
mod utils;
use rand::distributions::*;
use half::f16;
use itertools::Itertools;

use postludium::*;

mod smaa_extra_textures;
use smaa_extra_textures::*;

use rayon::prelude::*;

use std::ops::Deref;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::*;

// For InputSystem
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum LogicalInputTypes
{
	Horizontal, Vertical, Shoot, Slowdown, Overdrive
}

/*
fn pack_color(canvas_size: &Size2, red: DecompressedChannelImageData, green: DecompressedChannelImageData,
	blue: DecompressedChannelImageData, alpha: DecompressedChannelImageData) -> Vec<u8>
{
	let &Size2(cwidth, cheight) = canvas_size;
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
*/
fn single(canvas_size: &Size2, v: DecompressedChannelImageData) -> Vec<u8>
{
	let &Size2(cwidth, cheight) = canvas_size;
	let mut color_pixels = vec![0u8; (cwidth * cheight) as usize];
	for (x, y, px, py) in (0 .. v.height()).flat_map(|y| (0 .. v.width()).map(move |x| (x, y)))
		.map(|(x, y)| (x, y, x as isize + v.offset_x(), y as isize + v.offset_y()))
		.filter(|&(_, _, px, py)| (0 <= px && px < cwidth as isize) && (0 <= py && py < cheight as isize))
	{
		color_pixels[(px + py * cwidth as isize) as usize] = v.fetch(x, y);
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
	pub fn new<Engine: AssetProvider + Deref<Target = GraphicsInterface>>(engine: &Engine, render_passes: &RenderPasses, processing_viewport: &Viewport) -> Self
	{
		let &Viewport(_, _, vw, vh, _, _) = processing_viewport;

		let evsh = ShaderProgram::new_postprocess_vertex_from_asset(engine, "shaders.smaa.EdgeDetectionV", "main").or_crash();
		let bwvsh = ShaderProgram::new_postprocess_vertex_from_asset(engine, "shaders.smaa.BlendWeightCalcV", "main").or_crash();
		let cvsh = ShaderProgram::new_postprocess_vertex_from_asset(engine, "shaders.smaa.CombineV", "main").or_crash();
		let esh = ShaderProgram::new_fragment_from_asset(engine, "shaders.smaa.EdgeDetection", "main").or_crash();
		let bwsh = ShaderProgram::new_fragment_from_asset(engine, "shaders.smaa.BlendWeightCalc", "main").or_crash();
		let csh = ShaderProgram::new_fragment_from_asset(engine, "shaders.smaa.Combine", "main").or_crash();

		let dss = [
			DescriptorSetLayout::new(engine, vec![Descriptor::CombinedSampler(1, ShaderStage::Fragment)].into()).or_crash(),
			DescriptorSetLayout::new(engine, vec![Descriptor::CombinedSampler(3, ShaderStage::Fragment)].into()).or_crash(),
			DescriptorSetLayout::new(engine, vec![Descriptor::CombinedSampler(2, ShaderStage::Fragment)].into()).or_crash()
		];
		let epl = PipelineLayout::new(engine, &[&dss[0]], &[]).or_crash();
		let bwpl = PipelineLayout::new(engine, &[&dss[1]], &[]).or_crash();
		let cpl = PipelineLayout::new(engine, &[&dss[2]], &[]).or_crash();

		let scons_rt_metrics = vec![
			(0, ConstantEntry::Float(vw)),
			(1, ConstantEntry::Float(vh)),
			(2, ConstantEntry::Float(vw.recip())),
			(3, ConstantEntry::Float(vh.recip()))
		];
		let mut gps =
		{
			let eps = GraphicsPipelineBuilder::for_postprocess(engine, &epl, PreciseRenderPass(&render_passes.smaa_edgedetect, 0),
				PipelineShaderProgram::unspecialized(&esh), processing_viewport).or_crash()
				.vertex_shader(PipelineShaderProgram(&evsh, scons_rt_metrics.clone()));
			let bwps = GraphicsPipelineBuilder::for_postprocess(engine, &bwpl, PreciseRenderPass(&render_passes.smaa_blendweight, 0),
				PipelineShaderProgram(&bwsh, scons_rt_metrics.clone()), processing_viewport).or_crash()
				.vertex_shader(PipelineShaderProgram(&bwvsh, scons_rt_metrics.clone()));
			let cps = GraphicsPipelineBuilder::for_postprocess(engine, &cpl, PreciseRenderPass(&render_passes.smaa_combine, 0),
				PipelineShaderProgram(&csh, scons_rt_metrics.clone()), processing_viewport).or_crash()
				.vertex_shader(PipelineShaderProgram(&cvsh, scons_rt_metrics));
			GraphicsPipelines::new(engine, &[&eps, &bwps, &cps]).or_crash()
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

fn gen_bullet_gradient(to: &mut [[u8; 4]; 16])
{
	fn interpolate(a: u8, b: u8, v: f32) -> u8 { (b as f32 * v + a as f32 * (1.0 - v)) as u8 }

	for n in 0 .. 16
	{
		to[n] = if n == 0 { [255, 255, 255, 255] }
			else if n < 6 { [interpolate(64, 255, n as f32 / 5.0), 255, interpolate(255, 64, n as f32 / 5.0), 255] }
			else if n < 12 { [255, interpolate(255, 192, (n as f32 - 6.0) / 5.0), 64, 255] }
			else { [255, interpolate(192, 128, (n as f32 - 12.0) / 3.0), interpolate(64, 128, (n as f32 - 12.0) / 3.0), 255] }
	}
}

struct StagingPair1<'d> { stage: &'d LinearImage2D, device: &'d ImageView1D }
struct StagingPair2<'d> { stage: &'d LinearImage2D, device: &'d ImageView2D }
impl<'d> StagingPair1<'d>
{
	fn copy_entire_region<'a: 'd>(&self, recorder: TransferCommandRecorder<'a>) -> TransferCommandRecorder<'a>
	{
		recorder.copy_image(self.stage, self.device.deref(), &[ImageCopyRegion::entire_colorbits(VkExtent3D(*self.device.size(), 1, 1))])
	}
}
impl<'d> StagingPair2<'d>
{
	fn copy_entire_region<'a: 'd>(&self, recorder: TransferCommandRecorder<'a>) -> TransferCommandRecorder<'a>
	{
		recorder.copy_image(self.stage, self.device.deref(), &[ImageCopyRegion::entire_colorbits(VkExtent3D(self.device.size().0, self.device.size().1, 1))])
	}
}
struct DevConfImageBindings<'d>
{
	bb_sfloat4: &'d ImageView2D, bb_unorm4f: &'d ImageView2D, bb_unorm2: &'d ImageView2D, bb_unorm4: &'d ImageView2D,
	smaa_areatex: StagingPair2<'d>, smaa_searchtex: StagingPair2<'d>, playerbullet_tex: StagingPair2<'d>, circle16_tex: StagingPair2<'d>,
	lineparticle_colortex: StagingPair1<'d>, bullet_colortex: StagingPair1<'d>,
	linear_sampler: &'d Sampler
}

/// in real secs
pub type GameTime = f32;
/// An updating util sets incl. Immutable Reference to Random Generator
pub struct GameUpdateArgs<'a>
{
	/// A global random generator
	pub randomizer: &'a mut rand::Rng,
	/// The delta time from previous frame, in seconds
	pub delta_time: GameTime
}
unsafe impl<'a> Sync for GameUpdateArgs<'a> {}

fn main()
{
	let engine = EngineBuilder::new("hardgrad_extend".into(), (0, 0, 1), "HardGrad -> Extend".into(), &Size2(640, 480))
		.asset_base(std::env::current_dir().unwrap().into()).device_feature_block_texture_compression().launch().or_crash();
	let target_format = engine.render_window().format();

	// Resources //
	let images = DevConfImages::from_file(&engine, "devconf.images", engine.render_window().size(), target_format).ensure_has_staging();
	// Reference Bindings //
	let image_names = DevConfImageBindings
	{
		bb_sfloat4: &images.images_2d()[0], bb_unorm4f: &images.images_2d()[1], bb_unorm2: &images.images_2d()[2], bb_unorm4: &images.images_2d()[3],
		smaa_areatex: StagingPair2 { stage: &images.staging_images()[2], device: &images.images_2d()[4] },
		smaa_searchtex: StagingPair2 { stage: &images.staging_images()[3], device: &images.images_2d()[5] },
		playerbullet_tex: StagingPair2 { stage: &images.staging_images()[4], device: &images.images_2d()[6] },
		circle16_tex: StagingPair2 { stage: &images.staging_images()[5], device: &images.images_2d()[7] },
		lineparticle_colortex: StagingPair1 { stage: &images.staging_images()[0], device: &images.images_1d()[0] },
		bullet_colortex: StagingPair1 { stage: &images.staging_images()[1], device: &images.images_1d()[1] },
		linear_sampler: &images.samplers()[0]
	};

	let playerbullet_image = PhotoshopDocument::open(engine.parse_asset("graphs.playerbullet", "psd")).unwrap();
	let circle16_image = PhotoshopDocument::open(engine.parse_asset("graphs.circle16", "psd")).unwrap();
	{
		let mapped = images.map_staging_images_memory();
		let offsets = images.staging_offsets();
		let areatex_compressed = BC5::compress(&AREATEX_BYTES, (AREATEX_WIDTH, AREATEX_HEIGHT));
		mapped.map_mut::<[u8; AREATEX_SIZE / 2]>(offsets[2] as usize).copy_from_slice(&areatex_compressed);
		let searchtex_compressed = BC4::compress(&SEARCHTEX_BYTES, (SEARCHTEX_WIDTH, SEARCHTEX_HEIGHT));
		mapped.map_mut::<[u8; SEARCHTEX_SIZE / 2]>(offsets[3] as usize).copy_from_slice(&searchtex_compressed);

		let playerbullet_pixels = BC4::compress(&single(&Size2(playerbullet_image.width as u32, playerbullet_image.height as u32),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Alpha)
		), (playerbullet_image.width, playerbullet_image.height));
		let circle16_pixels = BC4::compress(&single(&Size2(circle16_image.width as u32, circle16_image.height as u32),
			circle16_image.layer_raw_channel_image_data(0, PSDChannelIndices::Alpha)
		), (circle16_image.width, circle16_image.height));
		mapped.range_mut::<u8>(offsets[4] as usize, 16 * 16 / 2).copy_from_slice(&playerbullet_pixels);
		mapped.range_mut::<u8>(offsets[5] as usize, 16 * 16 / 2).copy_from_slice(&circle16_pixels);
		mapped.map_mut::<[[f16; 4]; 4]>(offsets[0] as usize).copy_from_slice(&[
			[f16::from_f64(2.0), f16::from_f64(1.5), f16::from_f64(1.0), f16::from_f64(1.0)],
			[f16::from_f64(1.5), f16::from_f64(1.0), f16::from_f64(0.25), f16::from_f64(1.0)],
			[f16::from_f64(1.0), f16::from_f64(0.1875), f16::from_f64(0.125), f16::from_f64(0.875)],
			[f16::from_f64(0.125), f16::from_f64(0.125), f16::from_f64(0.125), f16::from_f64(0.375)]
		]);
		gen_bullet_gradient(mapped.map_mut(offsets[1] as usize));
	}
	let appdata = ApplicationBufferData::new(&engine, engine.render_window().size());

	let render_pass = RenderPasses::new(&engine, target_format);
	let framebuffers = Framebuffers::new(&engine, &render_pass, image_names.bb_sfloat4, image_names.bb_unorm4f, image_names.bb_unorm2, image_names.bb_unorm4,
		engine.render_window().render_targets(), engine.render_window().size());

	// Pipelines //
	let sc_viewport = Viewport::from(engine.render_window().size().clone());
	let pipelines = PipelineStates::new(&engine, true, &render_pass, &sc_viewport);

	// Descriptor Set //
	initialize_descriptor_sets(&engine, &appdata, &pipelines.descriptor_sets, &image_names);

	// Initial Data Transmission, Layouting for Swapchain Backbuffer Images //
	TransientTransferCommandBuffers::allocate(&engine, 1).and_then(|setup_commands|
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
			ImageMemoryBarrier::template(&**image_names.smaa_areatex.device, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(&**image_names.smaa_searchtex.device, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(&**image_names.playerbullet_tex.device, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(&**image_names.circle16_tex.device, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(&**image_names.lineparticle_colortex.device, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(&**image_names.bullet_colortex.device, ImageSubresourceRange::base_color())
		];
		let blitted_image_templates_stg = vec![
			ImageMemoryBarrier::template(image_names.smaa_areatex.stage, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(image_names.smaa_searchtex.stage, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(image_names.playerbullet_tex.stage, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(image_names.circle16_tex.stage, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(image_names.lineparticle_colortex.stage, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(image_names.bullet_colortex.stage, ImageSubresourceRange::base_color())
		];
		let image_memory_barriers = engine.render_window().render_targets().iter()
			.map(|x| ImageMemoryBarrier::hold_ownership(x, ImageSubresourceRange::base_color(),
				0, VK_ACCESS_MEMORY_READ_BIT, VkImageLayout::Undefined, VkImageLayout::PresentSrcKHR))
			.chain(vec![
				ImageMemoryBarrier::initialize(&**image_names.bb_sfloat4, ImageSubresourceRange::base_color(), VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::ColorAttachmentOptimal),
				ImageMemoryBarrier::initialize(&**image_names.bb_unorm4f, ImageSubresourceRange::base_color(), VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal),
				ImageMemoryBarrier::initialize(&**image_names.bb_unorm2, ImageSubresourceRange::base_color(), VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal),
				ImageMemoryBarrier::initialize(&**image_names.bb_unorm4, ImageSubresourceRange::base_color(), VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal)
			]).chain(blitted_image_templates_dev.iter().map(|t| t.into_transfer_dst(0, VkImageLayout::Preinitialized)))
			.chain(blitted_image_templates_stg.into_iter().map(|t| t.into_transfer_src(0, VkImageLayout::Preinitialized))).collect_vec();
		let image_memory_barriers_ret = blitted_image_templates_dev.into_iter()
			.map(|t| t.from_transfer_dst(VK_ACCESS_SHADER_READ_BIT, VkImageLayout::ShaderReadOnlyOptimal)).collect_vec();

		try!(setup_commands.begin(0).and_then(|recorder| recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false,
				&[], &buffer_memory_barriers, &image_memory_barriers)
			.copy_buffer(&appdata.stg, &appdata.dev, &[BufferCopyRegion(0, 0, appdata.size())])
			.inject_commands(|r| image_names.smaa_areatex.copy_entire_region(r))
			.inject_commands(|r| image_names.smaa_searchtex.copy_entire_region(r))
			.inject_commands(|r| image_names.playerbullet_tex.copy_entire_region(r))
			.inject_commands(|r| image_names.circle16_tex.copy_entire_region(r))
			.inject_commands(|r| image_names.lineparticle_colortex.copy_entire_region(r))
			.inject_commands(|r| image_names.bullet_colortex.copy_entire_region(r))
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT, false,
				&[], &buffer_memory_barriers_ret, &image_memory_barriers_ret)
			.end()
		));
		setup_commands.execute()
	}).or_crash();

	// Debug Information //
	let frames_per_second = RefCell::new(0u32);
	let frame_time_ms = RefCell::new(0.0f64);
	let cputime_ms = RefCell::new(0.0f64);
	let enemy_count = RefCell::new(0u32);
	let player_bithash = RefCell::new(0u32);
	let debug_info = DebugInfo::new(&engine, &[
		DebugLine::UnsignedInt("FPS".to_owned(), &frames_per_second, None),
		DebugLine::Float("Frame Time".to_owned(), &frame_time_ms, Some("ms".to_owned())),
		DebugLine::Float("CPU Time".to_owned(), &cputime_ms, Some("ms".to_owned())),
		DebugLine::UnsignedInt("Enemy Count".to_owned(), &enemy_count, None),
		DebugLine::UnsignedInt("Player Bithash".to_owned(), &player_bithash, None)
	], &render_pass.smaa_combine, 0, &sc_viewport).or_crash();

	info!("Recording Rendering Commands...");
	// Rendering Commands //
	let combine_commands = GraphicsCommandBuffers::allocate(&engine, engine.render_window().render_targets().len()).or_crash();
	for (n, f) in framebuffers.final_output.iter().enumerate()
	{
		let presenter_output_barriers = [
			ImageMemoryBarrier::hold_ownership(&engine.render_window().render_targets()[n], ImageSubresourceRange::base_color(),
				VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
				VkImageLayout::PresentSrcKHR, VkImageLayout::ColorAttachmentOptimal)
		];

		combine_commands.begin(n).and_then(|recorder|
			recorder
				.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, false, &[], &[], &presenter_output_barriers)
				.begin_render_pass(f, &[], false)
				.bind_pipeline(&pipelines.smaa.as_ref().unwrap().combine)
				.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().combine_layout, &[pipelines.descriptor_sets.smaa_combine])
				.bind_vertex_buffers(&[(&appdata.dev, appdata.offset_ppvbuf())])
				.draw(4, 1)
				.inject_commands(|r| debug_info.inject_render_commands(r))
				.end_render_pass()
			.end()
		).or_crash();
	}
	let gcommands = GraphicsCommandBuffers::allocate(&engine, 1).or_crash();
	gcommands.begin(0).and_then(|recorder|
	{
		let rr_clear_value = AttachmentClearValue::Color(0.0f32, 0.0f32, 0.015625f32, 1.0f32);
		let pure_clear_value = AttachmentClearValue::Color(0.0f32, 0.0f32, 0.0f32, 0.0f32);
		let color_output_barriers: Vec<_> = [
			ImageMemoryBarrier::template(&**image_names.bb_unorm4f, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(&**image_names.bb_unorm2, ImageSubresourceRange::base_color()),
			ImageMemoryBarrier::template(&**image_names.bb_unorm4, ImageSubresourceRange::base_color())
		].into_iter().map(|x| x.hold_ownership(VK_ACCESS_SHADER_READ_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
			VkImageLayout::ShaderReadOnlyOptimal, VkImageLayout::ColorAttachmentOptimal)).collect();
		
		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, false, &[], &[], &color_output_barriers)
			.begin_render_pass(&framebuffers.normal_render, &[rr_clear_value], false)
			.inject_commands(|r| populate_normal_render_commands(r, &pipelines, &appdata))
			.next_subpass(false)
			// Tonemapping //
			.bind_vertex_buffers(&[(&appdata.dev, appdata.offset_ppvbuf())])
			.bind_pipeline(&pipelines.tonemapper)
			.bind_descriptor_sets(&pipelines.layout_for_attachment_input(), &[pipelines.descriptor_sets.tonemap_input])
			.draw(4, 1)
			.end_render_pass().begin_render_pass(&framebuffers.smaa_edgedetect, &[pure_clear_value.clone()], false)
			// Edge Detection(SMAA 1x) //
			.bind_pipeline(&pipelines.smaa.as_ref().unwrap().edgedetect)
			.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().edgedetect_layout, &[pipelines.descriptor_sets.smaa_edgedetect])
			.draw(4, 1)
			.end_render_pass().begin_render_pass(&framebuffers.smaa_blendweight, &[pure_clear_value], false)
			// Blend Weight Calculation(SMAA 1x) //
			.bind_pipeline(&pipelines.smaa.as_ref().unwrap().blendweight_calc)
			.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().blendweight_layout, &[pipelines.descriptor_sets.smaa_blendweight])
			.draw(4, 1)
			.end_render_pass()
		.end()
	}).or_crash();
	info!("Recording Transfer Commands...");
	// Transfer Commands //
	let update_commands = TransferCommandBuffers::allocate(&engine, 1).or_crash();
	update_commands.begin(0).and_then(|recorder|
	{
		let buffer_barriers = [
			BufferMemoryBarrier::hold_ownership(&appdata.dev, appdata.range_need_to_update(),
				VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT, VK_ACCESS_TRANSFER_WRITE_BIT),
			BufferMemoryBarrier::hold_ownership(&appdata.stg, appdata.range_need_to_update(), VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_TRANSFER_READ_BIT)
		];
		let buffer_barriers_ret = [
			BufferMemoryBarrier::hold_ownership(&appdata.dev, appdata.range_need_to_update(),
				VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT),
			BufferMemoryBarrier::hold_ownership(&appdata.stg, appdata.range_need_to_update(), VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_MEMORY_READ_BIT)
		];

		let r = appdata.range_need_to_update();
		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false, &[], &buffer_barriers, &[])
			.copy_buffer(&appdata.stg, &appdata.dev, &[interlude::BufferCopyRegion(r.start, r.start, r.len())])
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, false, &[], &buffer_barriers_ret, &[])
		.end()
	}).or_crash();

	info!("Preparing for Render Loop...");

	let _/*engine*/ = {
		let window_system = engine.render_window().clone();
		let input_system = engine.input_system_ref().clone();
		let exit_flag = Arc::new(AtomicBool::new(false));
		let exit_flag_uo = exit_flag.clone();
		let execute_next_signal = Fence::new(&engine).or_crash();
		let copy_completion_sig = Fence::new(&engine).or_crash();
		let dbg_copy_completion_sig = Fence::new(&engine).or_crash();
		let rendering_order_sem = QueueFence::new(&engine).or_crash();
		let debug_transfer_commands = debug_info.get_transfer_commands();
		let update_event = Event::new("Update Event").or_crash();
		let srv_update = update_event.clone();
		let update_observer = unsafe { thread_scoped::scoped(move ||
		{
			let final_commands = combine_commands;
			let update_commands = update_commands;
			let mut frame_index = engine.render_window().acquire_next_target_index(&rendering_order_sem).and_then(|findex|
				engine.submit_graphics_commands(&[gcommands[0], final_commands[findex as usize]],
					&[(&rendering_order_sem, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)],
					None, Some(&execute_next_signal)).map(|()| findex)
				).or_crash();
			while !exit_flag_uo.load(Ordering::Acquire)
			{
				execute_next_signal.wait().and_then(|()| execute_next_signal.clear()).or_crash();
				Unrecoverable!(engine.submit_transfer_commands(&debug_transfer_commands[..], &[], None, Some(&dbg_copy_completion_sig)));
				Unrecoverable!(engine.submit_transfer_commands(&update_commands[..], &[], None, Some(&copy_completion_sig)));
				copy_completion_sig.wait().and_then(|()| copy_completion_sig.clear()).or_crash();
				dbg_copy_completion_sig.wait().and_then(|()| dbg_copy_completion_sig.clear()).or_crash();
				srv_update.set();
				frame_index = engine.render_window().present(&engine, frame_index, None).and_then(|()|
					engine.render_window().acquire_next_target_index(&rendering_order_sem).and_then(|findex|
					{
						engine.submit_graphics_commands(&[gcommands[0], final_commands[findex as usize]],
							&[(&rendering_order_sem, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)],
							None, Some(&execute_next_signal)).map(|()| findex)
					})).or_crash();
			}

			engine.wait_device().or_crash();
			engine
		}) };

		let mapped_range = appdata.stg.map().or_crash();
		let (uref_enemy, uref_bk, uref_player_center, uref_gametime, uref_particle_infos) =
		{
			let mapped = mapped_range.map_mut::<UniformMemory>(appdata.offset_uniform());
			(&mut mapped.enemy_instance_data, &mut mapped.background_instance_data, &mut mapped.player_center_tf,
				&mut mapped.gametime, &mut mapped.lineburst_particles)
		};
		let ref mut uref_bullet = mapped_range.map_mut::<BulletTranslations>(appdata.offset_bullet_translations()).0;
		let (iref_enemy, iref_bk, iref_player, iref_enemy_rez, iref_player_bullet, iref_lineburst_particle_groups, iref_bullet) =
		{
			let mapped = mapped_range.map_mut::<InstanceMemory>(appdata.offset_instance());
			(&mut mapped.enemy_instance_mult, &mut mapped.background_instance_mult, &mut mapped.player_rotq,
				&mut mapped.enemy_rez_instance_data, &mut mapped.player_bullet_offset_sincos, &mut mapped.lineburst_particle_groups,
				&mut mapped.bullet_instances)
		};
		let mut background_datastore = BackgroundDatastore::new(uref_bk, iref_bk);
		let mut enemy_datastore = EnemyDatastore::new(iref_enemy);
		let mut pb_memory_manager = utils::MemoryBlockManager::new(MAX_PLAYER_BULLET_COUNT as u32);
		let mut lineburst_particles = LineBurstParticles::new(iref_lineburst_particle_groups, uref_particle_infos);
		let mut bullet_datastore = BulletDatastore::new(iref_bullet);

		// double-buffered enemy entity list //
		let mut enemy_entities: [Enemy; MAX_ENEMY_COUNT] = unsafe { std::mem::uninitialized() };
		for n in 0 .. MAX_ENEMY_COUNT { enemy_entities[n] = Enemy::Free; }
		let mut player = Player::new(uref_player_center, iref_player);
		let mut player_bullets: [PlayerBullet; MAX_PLAYER_BULLET_COUNT] = unsafe { std::mem::uninitialized() };
		for n in 0 .. MAX_PLAYER_BULLET_COUNT { player_bullets[n] = PlayerBullet::Free; }
		let mut bullets: [Bullet; MAX_BULLETS] = unsafe { std::mem::uninitialized() };
		for n in 0 .. MAX_BULLETS { bullets[n] = Bullet::Free; }

		let mut mgr = EnemyManager::new();
		// let mut eg = EnemyGroup::new(spawn_group::strategies::RandomFall(0.1, 10));

		let mut frequest_queue = Vec::<FireRequest>::new();
		let mut secs_from_last_fixed = 0.0f32;
		input_system.write().and_then(|mut isw|
		{
			isw.add_input(LogicalInputTypes::Horizontal, InputType::Axis(InputAxis::X));
			isw.add_input(LogicalInputTypes::Horizontal, InputType::KeyAsAxis(InputKeys::Left, InputKeys::Right));
			isw.add_input(LogicalInputTypes::Vertical, InputType::Axis(InputAxis::Y));
			isw.add_input(LogicalInputTypes::Vertical, InputType::KeyAsAxis(InputKeys::Up, InputKeys::Down));
			isw.add_input(LogicalInputTypes::Shoot, InputType::Key(InputKeys::ButtonA));
			isw.add_input(LogicalInputTypes::Shoot, InputType::Key(InputKeys::Character('z')));
			isw.add_input(LogicalInputTypes::Slowdown, InputType::Axis(InputAxis::RZ));
			isw.add_input(LogicalInputTypes::Slowdown, InputType::Key(InputKeys::ButtonX));
			isw.add_input(LogicalInputTypes::Slowdown, InputType::Key(InputKeys::Character('x')));
			isw.add_input(LogicalInputTypes::Overdrive, InputType::Axis(InputAxis::Z));
			Ok(())
		}).unwrap();
		let mut randomizer = rand::thread_rng();
		let background_appear_rate = rand::distributions::Range::new(0, 6);
		let mut background_next_appear = false;
		let mut prev_time = time::PreciseTime::now();
		let mut shooting = false;
		let mut secs_from_last_trigger = 0.0;
		let mut game_secs = 0.0;
		let mut prev_fps_period = time::PreciseTime::now();
		let mut fpscount = 0;
		let mut next_shoot = false;
		let mut next_particle_spawn = Vec::new();
		let particle_spawn_count = rand::distributions::Range::new(1, 8);
		loop
		{
			let msg = window_system.process_events_and_messages(&[&update_event]);
			match msg
			{
				ApplicationState::Exited => break,
				ApplicationState::EventArrived(0) =>
				{
					// update
					update_event.reset();
					let delta_time = prev_time.to(time::PreciseTime::now());
					prev_time = time::PreciseTime::now();
					*frame_time_ms.borrow_mut() = delta_time.num_microseconds().unwrap_or(-1) as f64 / 1000.0f64;
					fpscount += 1;

					// normal update
					let cputime_start = time::PreciseTime::now();
					input_system.write().unwrap().update();
					let inputs = input_system.read().unwrap();
					let timescale = (1.0f32 + inputs[LogicalInputTypes::Slowdown] * 2.0f32) / (1.0f32 + inputs[LogicalInputTypes::Overdrive]);
					let movescale = 1.0f32 + inputs[LogicalInputTypes::Slowdown] * 0.25f32;
					let mut update_args = GameUpdateArgs
					{
						delta_time: (delta_time.num_microseconds().unwrap() as f32 / 1_000_000.0) / timescale,
						randomizer: &mut randomizer
					};
					secs_from_last_fixed += update_args.delta_time;
					secs_from_last_trigger += update_args.delta_time;
					game_secs += update_args.delta_time;
					uref_gametime[0] = game_secs;
					background_datastore.update(&mut update_args, background_next_appear);

					let new_shooting = input_system.read().unwrap()[LogicalInputTypes::Shoot] > 0.0;
					next_shoot = if !shooting && new_shooting
					{
						// start timer
						secs_from_last_trigger = update_args.delta_time;
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
					player_bullets.par_iter_mut().for_each(|e| e.update(&update_args));
					for e in player_bullets.iter_mut().filter(|e| e.is_garbage())
					{
						match e { &mut PlayerBullet::Garbage(bindex) => pb_memory_manager.free(bindex), _ => unreachable!() };
						*e = PlayerBullet::Free;
					}

					mgr.update(&mut update_args, |x, y, lref, manage_index| if let Some(bindex) = enemy_datastore.allocate_block()
					{
						let bindexi = bindex as isize;
						enemy_entities[bindex as usize] = unsafe
						{
							let ref mut uref_enemy_ptr = *uref_enemy.as_mut_ptr().offset(bindexi);
							let ref mut iref_enemy_rez_ptr = *iref_enemy_rez.as_mut_ptr().offset(bindexi);
							Enemy::init(x, bindex, lref, manage_index, uref_enemy_ptr, iref_enemy_rez_ptr)
						};
						*enemy_count.borrow_mut() += 1;
						Some(bindex)
					} else { None });
					frequest_queue.clear();
					for e in enemy_entities.iter_mut()
					{
						let op = e.update(&update_args, &mut frequest_queue);
						if let Some((new_left, new_top)) = op
						{
							for pb in player_bullets.iter_mut()
							{
								if let Some((psx, psy)) = pb.crash(new_left, new_top)
								{
									next_particle_spawn.push((particle_spawn_count.ind_sample(&mut update_args.randomizer), psx, psy));
								}
							}
						}
					}
					for e in enemy_entities.iter_mut().filter(|e| e.is_garbage())
					{
						match e { &mut Enemy::Garbage(bindex) => enemy_datastore.free_block(bindex), _ => unreachable!() };
						*e = Enemy::Free;
						*enemy_count.borrow_mut() -= 1;
					}
					*player_bithash.borrow_mut() = player.update(&update_args, &*inputs, movescale);
					// println!("PlayerBitHashBin: {:08b}", *player_bithash.borrow());
					bullets.par_iter_mut().for_each(|e| e.update(&update_args));
					bullet_datastore.increase_all_lifetime(update_args.delta_time);
					for e in bullets.iter_mut().filter(|e| e.is_garbage())
					{
						match e { &mut Bullet::Garbage(i) => bullet_datastore.free(i), _ => unreachable!() };
						*e = Bullet::Free;
					}
					for f in &frequest_queue
					{
						match f
						{
							&FireRequest::Linears(ref vinfo) => for &(from, angle, speed) in vinfo
							{
								if let Some(bindex) = bullet_datastore.allocate()
								{
									bullets[bindex as usize] = unsafe
									{
										let uref_bullet_ref_ptr = uref_bullet.as_mut_ptr();
										bullet_datastore.init_lifetime(bindex);
										Bullet::init_linear(bindex, &mut *uref_bullet_ref_ptr.offset(bindex as isize), &from, angle, speed)
									};
								}
								else { warn!("Bullet Datastore is full!!"); }
							}
						}
					}

					if !next_particle_spawn.is_empty()
					{
						for &(count, x, y) in next_particle_spawn.iter()
						{
							lineburst_particles.spawn(count, x, y, game_secs);
						}
						next_particle_spawn.clear();
					}
					lineburst_particles = lineburst_particles.garbage_collect(game_secs);

					background_next_appear = false;
					*cputime_ms.borrow_mut() = cputime_start.to(time::PreciseTime::now()).num_microseconds().unwrap_or(0) as f64 / 1000.0f64;
					debug_info.update();
				},
				_ => ()
			}

			if secs_from_last_fixed >= 1.0 / 60.0
			{
				// fixed update
				background_next_appear = background_appear_rate.ind_sample(&mut randomizer) == 0;
				secs_from_last_fixed -= 1.0 / 60.0;
			}
			if shooting && secs_from_last_trigger >= 0.0375
			{
				next_shoot = true;
				secs_from_last_trigger -= 0.0375;
			}
			if prev_fps_period.to(time::PreciseTime::now()) >= time::Duration::seconds(1)
			{
				*frames_per_second.borrow_mut() = fpscount;
				fpscount = 0;
				prev_fps_period = time::PreciseTime::now();
			}
		}

		exit_flag.store(true, Ordering::Release);
		update_observer.join()
	};
}

/// Initializes content in all Descriptor Sets
fn initialize_descriptor_sets(engine: &GraphicsInterface, appdata: &ApplicationBufferData, descriptor_sets: &DescriptorSetBindings, images: &DevConfImageBindings)
{
	let uniform_memory_bt_info	= BufferInfo(&appdata.dev, appdata.offset_bullet_translations() .. appdata.size_bullet_translations());
	let uniform_memory_info		= BufferInfo(&appdata.dev, appdata.offset_uniform() .. appdata.offset_uniform() + appdata.size_uniform());
	let backbuffer_sfloat4_info = ImageInfo(images.linear_sampler, images.bb_sfloat4, VkImageLayout::ShaderReadOnlyOptimal);
	let backbuffer_unorm4f_info = ImageInfo(images.linear_sampler, images.bb_unorm4f, VkImageLayout::ShaderReadOnlyOptimal);
	let backbuffer_unorm2_info	= ImageInfo(images.linear_sampler, images.bb_unorm2, VkImageLayout::ShaderReadOnlyOptimal);
	let backbuffer_unorm4_info	= ImageInfo(images.linear_sampler, images.bb_unorm4, VkImageLayout::ShaderReadOnlyOptimal);
	let areatex_info			= ImageInfo(images.linear_sampler, images.smaa_areatex.device, VkImageLayout::ShaderReadOnlyOptimal);
	let searchtex_info			= ImageInfo(images.linear_sampler, images.smaa_searchtex.device, VkImageLayout::ShaderReadOnlyOptimal);
	let playerbullet_info		= ImageInfo(images.linear_sampler, images.playerbullet_tex.device, VkImageLayout::ShaderReadOnlyOptimal);
	let circle16_info			= ImageInfo(images.linear_sampler, images.circle16_tex.device, VkImageLayout::ShaderReadOnlyOptimal);
	let lineburst_particle_gradient_tex_info = ImageInfo(images.linear_sampler, images.lineparticle_colortex.device, VkImageLayout::ShaderReadOnlyOptimal);
	let bullet_colramp_tex_info	= ImageInfo(images.linear_sampler, images.bullet_colortex.device, VkImageLayout::ShaderReadOnlyOptimal);
	engine.update_descriptors(&[
		DescriptorSetWriteInfo::UniformBuffer(descriptor_sets.global_uniform, 0, vec![uniform_memory_info]),
		DescriptorSetWriteInfo::StorageBuffer(descriptor_sets.global_uniform, 1, vec![uniform_memory_bt_info]),
		DescriptorSetWriteInfo::InputAttachment(descriptor_sets.tonemap_input, 0, vec![backbuffer_sfloat4_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(descriptor_sets.smaa_edgedetect, 0, vec![backbuffer_unorm4f_info.clone()]),
		DescriptorSetWriteInfo::CombinedImageSampler(descriptor_sets.smaa_blendweight, 0, vec![backbuffer_unorm2_info, areatex_info, searchtex_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(descriptor_sets.smaa_combine, 0, vec![backbuffer_unorm4f_info, backbuffer_unorm4_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(descriptor_sets.playerbullet_texture, 0, vec![playerbullet_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(descriptor_sets.lineburst_particle_color, 0, vec![lineburst_particle_gradient_tex_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(descriptor_sets.bullet_texture, 0, vec![circle16_info]),
		DescriptorSetWriteInfo::CombinedImageSampler(descriptor_sets.bullet_color, 0, vec![bullet_colramp_tex_info])
	]);
}
/// Records some commands for NormalRender
pub fn populate_normal_render_commands<'a>(recorder: GraphicsCommandRecorder<'a>, pipelines: &PipelineStates, appdata: &ApplicationBufferData)
	-> GraphicsCommandRecorder<'a>
{
	recorder
		.bind_descriptor_sets(pipelines.layout_for_wire_render(), &[pipelines.descriptor_sets.global_uniform])
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
		.inject_commands(|r| pipelines.bullet.begin(r, pipelines.descriptor_sets.bullet_texture))
		.bind_descriptor_sets_partial(pipelines.layout_for_bullet(), 2, &[pipelines.descriptor_sets.bullet_color])
		.bind_vertex_buffers(&[
			(&appdata.dev, appdata.offset_vbuf() + VertexMemoryForWireRender::sprite_plane_offs()),
			(&appdata.dev, appdata.offset_instance() + InstanceMemory::bullet_instances_offs())
		])
		.draw(4, MAX_BULLETS as u32)
		.inject_commands(|r| pipelines.playerbullet.begin(r, pipelines.descriptor_sets.playerbullet_texture))
		.bind_vertex_buffers_partial(1, &[
			(&appdata.dev, appdata.offset_instance() + InstanceMemory::player_bullet_offs())
		])
		.draw(4, MAX_PLAYER_BULLET_COUNT as u32)
		.bind_pipeline(&pipelines.lineburst)
		.bind_descriptor_sets_partial(&pipelines.layout_for_lineburst_particle_render(), 1, &[pipelines.descriptor_sets.lineburst_particle_color])
		.bind_vertex_buffers(&[(&appdata.dev, appdata.offset_instance() + structures::InstanceMemory::lbparticle_groups_offs())])
		.draw(MAX_LBPARTICLE_GROUPS as u32, 1)
}
