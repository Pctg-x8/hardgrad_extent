
extern crate nalgebra;
extern crate rand;
extern crate time;
extern crate thread_scoped;
extern crate glob;
extern crate rayon;
#[macro_use] extern crate log;
extern crate itertools;

#[macro_use] extern crate interlude;
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

mod postludium;
use postludium::*;

mod smaa_extra_textures;
use smaa_extra_textures::*;

use rayon::prelude::*;

use std::rc::Rc;
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
enum PlayerBullet<'a>
{
	Free, Entity { block_index: u32, offs_sincos_ref: &'a mut CVector4 }, Garbage(u32)
}
impl<'a> PlayerBullet<'a>
{
	pub fn init(init_left: f32, init_top: f32, init_angle: f32, block_index: u32, offs_sincos_ref: &'a mut CVector4) -> Self
	{
		offs_sincos_ref[0] = init_left;
		offs_sincos_ref[1] = init_top;
		let (s, c) = init_angle.to_radians().sin_cos();
		offs_sincos_ref[2] = s; offs_sincos_ref[3] = c;

		PlayerBullet::Entity { block_index: block_index, offs_sincos_ref: offs_sincos_ref }
	}
	pub fn update(&mut self, delta_time: f32)
	{
		let died_index = match self
		{
			&mut PlayerBullet::Entity { block_index: block, offs_sincos_ref: ref mut offs_sincos } =>
			{
				offs_sincos[0] += offs_sincos[2] * 8.0 * 14.0 * delta_time;
				offs_sincos[1] -= offs_sincos[3] * 8.0 * 14.0 * delta_time;
				if offs_sincos[0].abs() > 32.0 || !(0.0 <= offs_sincos[1] && offs_sincos[1] <= 50.0)
				{
					offs_sincos[0] = std::f32::MAX;
					offs_sincos[1] = std::f32::MAX;
					Some(block)
				}
				else { None }
			}, _ => None
		};
		
		if let Some(bindex) = died_index { *self = PlayerBullet::Garbage(bindex); }
	}
	pub fn is_garbage(&self) -> bool { match self { &PlayerBullet::Garbage(_) => true, _ => false } }
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

	pub fn left(&self) -> f32 { self.uniform_memory[0] }
	pub fn top(&self) -> f32 { self.uniform_memory[1] }
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
// Wire Render Wrapper with moving pipeline state object
pub struct WireRender
{
	renderstate: interlude::GraphicsPipeline, layout_ref: Rc<interlude::PipelineLayout>
}
impl WireRender
{
	pub fn new(renderstate: interlude::GraphicsPipeline, layout: &Rc<interlude::PipelineLayout>) -> Self
	{
		WireRender { renderstate: renderstate, layout_ref: layout.clone() }
	}
	pub fn begin<RecorderT>(&self, comrec: RecorderT, wirecolor_r: f32, wirecolor_g: f32, wirecolor_b: f32, wirecolor_a: f32) -> RecorderT
		where RecorderT: DrawingCommandRecorder
	{
		comrec.bind_pipeline(&self.renderstate).push_constants(&self.layout_ref, &[interlude::ShaderStage::Vertex],
			0 .. std::mem::size_of::<structures::CVector4>() as u32, &[wirecolor_r, wirecolor_g, wirecolor_b, wirecolor_a])
	}
}
// Sprite Render with moving pipeline state object
pub struct SpriteRender
{
	renderstate: interlude::GraphicsPipeline, layout_ref: Rc<interlude::PipelineLayout>
}
impl SpriteRender
{
	pub fn new(renderstate: interlude::GraphicsPipeline, layout: &Rc<interlude::PipelineLayout>) -> Self
	{
		SpriteRender { renderstate: renderstate, layout_ref: layout.clone() }
	}
	pub fn begin<RecorderT>(&self, comrec: RecorderT, texture_ds: VkDescriptorSet) -> RecorderT
		where RecorderT: DrawingCommandRecorder
	{
		comrec.bind_pipeline(&self.renderstate).bind_descriptor_sets_partial(&self.layout_ref, 1, &[texture_ds])
	}
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

mod framebuffer;
use framebuffer::*;

#[allow(dead_code)]
struct SMAAPipelineStates
{
	edgedetect_vshader: interlude::ShaderProgram, blendweight_vshader: interlude::ShaderProgram, combine_vshader: interlude::ShaderProgram,
	edgedetect_shader: interlude::ShaderProgram, blendweight_shader: interlude::ShaderProgram, combine_shader: interlude::ShaderProgram,
	descriptor_sets: [interlude::DescriptorSetLayout; 3],
	pub edgedetect_layout: interlude::PipelineLayout, pub blendweight_layout: interlude::PipelineLayout, pub combine_layout: interlude::PipelineLayout,
	pub edgedetect: interlude::GraphicsPipeline, pub blendweight_calc: interlude::GraphicsPipeline, pub combine: interlude::GraphicsPipeline
}
impl SMAAPipelineStates
{
	pub fn new(engine: &interlude::Engine, render_pass: &interlude::RenderPass, base_subpass: usize, processing_viewport: VkViewport) -> Self
	{
		let VkViewport(_, _, vw, vh, _, _) = processing_viewport;

		let evsh = Unrecoverable!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.EdgeDetectionV", "main"));
		let bwvsh = Unrecoverable!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.BlendWeightCalcV", "main"));
		let cvsh = Unrecoverable!(engine.create_postprocess_vertex_shader_from_asset("shaders.smaa.CombineV", "main"));
		let esh = Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.smaa.EdgeDetection", "main"));
		let bwsh = Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.smaa.BlendWeightCalc", "main"));
		let csh = Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.smaa.Combine", "main"));

		let dss = [
			Unrecoverable!(engine.create_descriptor_set_layout(&[interlude::Descriptor::CombinedSampler(1, vec![interlude::ShaderStage::Fragment])])),
			Unrecoverable!(engine.create_descriptor_set_layout(&[interlude::Descriptor::CombinedSampler(3, vec![interlude::ShaderStage::Fragment])])),
			Unrecoverable!(engine.create_descriptor_set_layout(&[interlude::Descriptor::CombinedSampler(2, vec![interlude::ShaderStage::Fragment])]))
		];
		let epl = Unrecoverable!(engine.create_pipeline_layout(&[&dss[0]], &[]));
		let bwpl = Unrecoverable!(engine.create_pipeline_layout(&[&dss[1]], &[]));
		let cpl = Unrecoverable!(engine.create_pipeline_layout(&[&dss[2]], &[]));

		let scons_rt_metrics = vec![
			(0, interlude::ConstantEntry::Float(vw)),
			(1, interlude::ConstantEntry::Float(vh)),
			(2, interlude::ConstantEntry::Float(vw.recip())),
			(3, interlude::ConstantEntry::Float(vh.recip()))
		];
		let mut gps =
		{
			let eps = interlude::GraphicsPipelineBuilder::for_postprocess(engine, &epl, render_pass, base_subpass as u32 + 0,
				interlude::PipelineShaderProgram::unspecialized(&esh), processing_viewport)
				.vertex_shader(interlude::PipelineShaderProgram(&evsh, scons_rt_metrics.clone()));
			let bwps = interlude::GraphicsPipelineBuilder::for_postprocess(engine, &bwpl, render_pass, base_subpass as u32 + 1,
				interlude::PipelineShaderProgram(&bwsh, scons_rt_metrics.clone()), processing_viewport)
				.vertex_shader(interlude::PipelineShaderProgram(&bwvsh, scons_rt_metrics.clone()));
			let cps = interlude::GraphicsPipelineBuilder::for_postprocess(engine, &cpl, render_pass, base_subpass as u32 + 2,
				interlude::PipelineShaderProgram(&csh, scons_rt_metrics.clone()), processing_viewport)
				.vertex_shader(interlude::PipelineShaderProgram(&cvsh, scons_rt_metrics));
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
#[allow(dead_code)]
struct PipelineStates
{
	geometry_preinstancing_vsh: interlude::ShaderProgram, erz_preinstancing_vsh: interlude::ShaderProgram, player_rotate_vsh: interlude::ShaderProgram,
	solid_fsh: interlude::ShaderProgram, playerbullet_vsh: interlude::ShaderProgram, sprite_fsh: interlude::ShaderProgram,
	enemy_duplication_gsh: interlude::ShaderProgram, background_duplication_gsh: interlude::ShaderProgram, enemy_rezonator_duplication_gsh: interlude::ShaderProgram,
	global_uniform_layout: interlude::DescriptorSetLayout, sprite_texture_layout: interlude::DescriptorSetLayout,
	pub wire_layout: Rc<interlude::PipelineLayout>, pub sprite_layout: Rc<interlude::PipelineLayout>,
	pub background: WireRender, pub enemy_body: WireRender, pub enemy_rezonator: WireRender, pub player: WireRender, pub playerbullet: SpriteRender,
	pub smaa: Option<SMAAPipelineStates>,
	descriptor_sets: interlude::DescriptorSets
}
impl PipelineStates
{
	pub fn new(engine: &interlude::Engine, use_smaa: bool, render_pass: &interlude::RenderPass, swapchain_viewport: VkViewport) -> Self
	{
		let geometry_preinstancing_vsh = Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.GeometryPreinstancing", "main", &[
			interlude::VertexBinding::PerVertex(std::mem::size_of::<CVector4>() as u32),
			interlude::VertexBinding::PerInstance(std::mem::size_of::<u32>() as u32)
		], &[
			interlude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
			interlude::VertexAttribute(1, VkFormat::R32_UINT, 0)
		]));
		let enemy_rezonator_preinstancing_vsh = Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.EnemyRezonatorV", "main", &[
			interlude::VertexBinding::PerVertex(std::mem::size_of::<CVector4>() as u32),
			interlude::VertexBinding::PerInstance(std::mem::size_of::<CVector4>() as u32)
		], &[
			interlude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
			interlude::VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
		]));
		let player_rotate_vsh = Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.PlayerRotor", "main", &[
			interlude::VertexBinding::PerVertex(std::mem::size_of::<CVector4>() as u32),
			interlude::VertexBinding::PerInstance(std::mem::size_of::<CVector4>() as u32)
		], &[
			interlude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
			interlude::VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
		]));
		let playerbullet_vsh = Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.PlayerBullet", "main", &[
			interlude::VertexBinding::PerVertex(std::mem::size_of::<CVector4>() as u32),
			interlude::VertexBinding::PerInstance(std::mem::size_of::<CVector4>() as u32)
		], &[
			interlude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
			interlude::VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
		]));
		let solid_fsh = Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.ThroughColor", "main"));
		let sprite_fsh = Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.SpriteFrag", "main"));
		let enemy_duplication_gsh = Unrecoverable!(engine.create_geometry_shader_from_asset("shaders.EnemyDuplicator", "main"));
		let enemy_rezonator_duplication_gsh = Unrecoverable!(engine.create_geometry_shader_from_asset("shaders.EnemyRezonatorDup", "main"));
		let background_duplication_gsh = Unrecoverable!(engine.create_geometry_shader_from_asset("shaders.BackLineDuplicator", "main"));

		let gu_layout = Unrecoverable!(engine.create_descriptor_set_layout(&[
			interlude::Descriptor::Uniform(1, vec![interlude::ShaderStage::Vertex, interlude::ShaderStage::Geometry])
		]));
		let st_layout = Unrecoverable!(engine.create_descriptor_set_layout(&[
			interlude::Descriptor::CombinedSampler(1, vec![interlude::ShaderStage::Fragment])
		]));
		let wire_pl = Rc::new(Unrecoverable!(engine.create_pipeline_layout(&[&gu_layout],
			&[interlude::PushConstantDesc(VK_SHADER_STAGE_VERTEX_BIT, 0 .. std::mem::size_of::<CVector4>() as u32)])));
		let sprite_pl = Rc::new(Unrecoverable!(engine.create_pipeline_layout(&[&gu_layout, &st_layout], &[])));

		let mut gps =
		{
			let background_ps = interlude::GraphicsPipelineBuilder::new(&wire_pl, render_pass, 0)
				.vertex_shader(interlude::PipelineShaderProgram::unspecialized(&geometry_preinstancing_vsh))
				.geometry_shader(interlude::PipelineShaderProgram::unspecialized(&background_duplication_gsh))
				.fragment_shader(interlude::PipelineShaderProgram::unspecialized(&solid_fsh))
				.primitive_topology(interlude::PrimitiveTopology::LineList(true))
				.viewport_scissors(&[interlude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
				.blend_state(&[interlude::AttachmentBlendState::PremultipliedAlphaBlend]);
			let enemy_ps = interlude::GraphicsPipelineBuilder::inherit(&background_ps)
				.geometry_shader(interlude::PipelineShaderProgram::unspecialized(&enemy_duplication_gsh))
				.blend_state(&[interlude::AttachmentBlendState::Disabled]);
			let enemy_rezonator_ps = interlude::GraphicsPipelineBuilder::inherit(&enemy_ps)
				.vertex_shader(interlude::PipelineShaderProgram::unspecialized(&enemy_rezonator_preinstancing_vsh))
				.geometry_shader(interlude::PipelineShaderProgram::unspecialized(&enemy_rezonator_duplication_gsh))
				.primitive_topology(interlude::PrimitiveTopology::TriangleList(false));
			let player_ps = interlude::GraphicsPipelineBuilder::new(&wire_pl, render_pass, 0)
				.vertex_shader(interlude::PipelineShaderProgram::unspecialized(&player_rotate_vsh))
				.fragment_shader(interlude::PipelineShaderProgram::unspecialized(&solid_fsh))
				.primitive_topology(interlude::PrimitiveTopology::LineList(false))
				.viewport_scissors(&[interlude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
				.blend_state(&[interlude::AttachmentBlendState::Disabled]);
			let playerbullet_ps = interlude::GraphicsPipelineBuilder::new(&sprite_pl, render_pass, 0)
				.vertex_shader(interlude::PipelineShaderProgram(&playerbullet_vsh, vec![(0, interlude::ConstantEntry::Float(0.75))]))
				.fragment_shader(interlude::PipelineShaderProgram::unspecialized(&sprite_fsh))
				.primitive_topology(interlude::PrimitiveTopology::TriangleStrip(false))
				.viewport_scissors(&[interlude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
				.blend_state(&[interlude::AttachmentBlendState::PremultipliedAlphaBlend]);
			Unrecoverable!(engine.create_graphics_pipelines(&[&background_ps, &enemy_ps, &enemy_rezonator_ps, &player_ps, &playerbullet_ps]))
		};
		let playerbullet_sr = SpriteRender::new(gps.pop().unwrap(), &sprite_pl);
		let player_wr = WireRender::new(gps.pop().unwrap(), &wire_pl);
		let enemy_rezonator_wr = WireRender::new(gps.pop().unwrap(), &wire_pl);
		let enemy_wr = WireRender::new(gps.pop().unwrap(), &wire_pl);
		let background_wr = WireRender::new(gps.pop().unwrap(), &wire_pl);
		assert_eq!(gps.len(), 0);

		let (smaa, descriptor_sets) = if use_smaa
		{
			let ps = SMAAPipelineStates::new(engine, render_pass, 1, swapchain_viewport);
			let dslist = Unrecoverable!(engine.preallocate_all_descriptor_sets(&[&gu_layout, &st_layout, &ps.descriptor_sets[0], &ps.descriptor_sets[1], &ps.descriptor_sets[2]]));
			(Some(ps), dslist)
		}
		else
		{
			let dslist = Unrecoverable!(engine.preallocate_all_descriptor_sets(&[&gu_layout, &st_layout]));
			(None, dslist)
		};

		PipelineStates
		{
			geometry_preinstancing_vsh: geometry_preinstancing_vsh, erz_preinstancing_vsh: enemy_rezonator_preinstancing_vsh, player_rotate_vsh: player_rotate_vsh,
			solid_fsh: solid_fsh, enemy_duplication_gsh: enemy_duplication_gsh, enemy_rezonator_duplication_gsh: enemy_rezonator_duplication_gsh,
			background_duplication_gsh: background_duplication_gsh, playerbullet_vsh: playerbullet_vsh, sprite_fsh: sprite_fsh,
			global_uniform_layout: gu_layout, sprite_texture_layout: st_layout,
			wire_layout: wire_pl, sprite_layout: sprite_pl,
			background: background_wr, enemy_body: enemy_wr, enemy_rezonator: enemy_rezonator_wr, player: player_wr, playerbullet: playerbullet_sr,
			smaa: smaa, descriptor_sets: descriptor_sets
		}
	}
	
	pub fn get_descriptor_set_for_uniform_buffer(&self) -> VkDescriptorSet { self.descriptor_sets[0] }
	pub fn get_descriptor_set_for_playerbullet_texture(&self) -> VkDescriptorSet { self.descriptor_sets[1] }
	pub fn get_descriptor_set_for_smaa_edgedetect(&self) -> VkDescriptorSet { self.descriptor_sets[2] }
	pub fn get_descriptor_set_for_smaa_blendweight(&self) -> VkDescriptorSet { self.descriptor_sets[3] }
	pub fn get_descriptor_set_for_smaa_combine(&self) -> VkDescriptorSet { self.descriptor_sets[4] }
}

enum ApplicationEvent
{
	Update, Exit
}

fn main() { if let Err(e) = app_main() { interlude::crash(e); } }
fn app_main() -> Result<(), interlude::EngineError>
{
	utils::memory_management_test();

	let engine = try!{
		interlude::Engine::new("hardgrad_extend", 0x01, Some(std::env::current_dir().unwrap()), interlude::DeviceFeatures::new().enable_block_texture_compression())
	};
	let window_system = engine.window_system_ref().clone();
	let main_frame = try!(engine.create_render_window(VkExtent2D(640, 480), "HardGrad -> Extend"));
	let VkExtent2D(frame_width, frame_height) = main_frame.get_extent();

	// Resources //
	let images = DevConfImages::from_file(&engine, engine.parse_asset("devconf.images", "pdc"), main_frame.get_extent(), VkFormat::R8G8B8A8_UNORM);
	// Reference Bindings //
	let (ref gbuffer, ref gbuffer_view) = images.images_2d()[0];
	let (ref edgebuffer, ref edgebuffer_view) = images.images_2d()[1];
	let (ref blend_weight, ref blend_weight_view) = images.images_2d()[2];
	let (ref smaa_areatex, ref smaa_areatex_view) = images.images_2d()[3];
	let (ref smaa_searchtex, ref smaa_searchtex_view) = images.images_2d()[4];
	let (ref playerbullet_tex, ref playerbullet_view) = images.images_2d()[5];
	let (_, ref lineburst_particle_gradient_view) = images.images_1d()[0];
	let ref gbuffer_sampler = images.samplers()[0];
	let ref smaa_areatex_stg = images.staging_images().unwrap()[0];
	let ref smaa_searchtex_stg = images.staging_images().unwrap()[1];
	let ref playerbullet_tex_stg = images.staging_images().unwrap()[2];

	let playerbullet_image = PhotoshopDocument::open(engine.parse_asset("graphs.playerbullet", "psd")).unwrap();
	if let Some(mapped) = images.map_staging_images_memory()
	{
		let offsets = images.staging_offsets().unwrap();
		let areatex_compressed = BC5::compress(&AREATEX_BYTES, (AREATEX_WIDTH as usize, AREATEX_HEIGHT as usize));
		mapped.map_mut::<[u8; AREATEX_SIZE as usize / 2]>(offsets[0] as usize).copy_from_slice(&areatex_compressed);
		let searchtex_compressed = BC4::compress(&SEARCHTEX_BYTES, (SEARCHTEX_WIDTH as usize, SEARCHTEX_HEIGHT as usize));
		mapped.map_mut::<[u8; SEARCHTEX_SIZE as usize / 2]>(offsets[1] as usize).copy_from_slice(&searchtex_compressed);

		let playerbullet_pixels = pack_color(
			VkExtent2D(playerbullet_image.width as u32, playerbullet_image.height as u32),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Red),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Green),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Blue),
			playerbullet_image.layer_raw_channel_image_data(0, PSDChannelIndices::Alpha)
		);
		mapped.range_mut::<u8>(offsets[2] as usize, 16 * 16 * 4).copy_from_slice(&playerbullet_pixels);
	};
	let application_buffer_prealloc = engine.buffer_preallocate(&[
		(std::mem::size_of::<[interlude::PosUV; 4]>(), interlude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::VertexMemoryForWireRender>(), interlude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::IndexMemory>(), interlude::BufferDataType::Index),
		(std::mem::size_of::<structures::InstanceMemory>(), interlude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::UniformMemory>(), interlude::BufferDataType::Uniform)
	]);
	let (application_data, appdata_stage) = try!(engine.create_double_buffer(&application_buffer_prealloc));

	// Rendering Switches //
	let use_post_smaa = true;

	let render_passes = RenderPasses::new(&engine, main_frame.get_format());
	let enabled_pass = if use_post_smaa { &render_passes.fullset } else { &render_passes.noaa };
	let framebuffers = main_frame.get_back_images().iter().map(|&x| if use_post_smaa
	{
		Unrecoverable!(engine.create_framebuffer(&render_passes.fullset, &[gbuffer_view, edgebuffer_view, blend_weight_view, x], VkExtent3D(frame_width, frame_height, 1)))
	}
	else
	{
		Unrecoverable!(engine.create_framebuffer(&render_passes.noaa, &[x], VkExtent3D(frame_width, frame_height, 1)))
	}).collect::<Vec<_>>();

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
		vertices.sprite_plane_vts = [
			Position(-1.0, -1.0, 0.0, 1.0),
			Position( 1.0, -1.0, 0.0, 1.0),
			Position(-1.0,  1.0, 0.0, 1.0),
			Position( 1.0,  1.0, 0.0, 1.0)
		];
		indices.player_cube_ids = [
			0, 1, 1, 2, 2, 3, 3, 0,
			4, 5, 5, 6, 6, 7, 7, 4,
			0, 4, 1, 5, 2, 6, 3, 7
		];
		let uniforms = mapped.map_mut::<structures::UniformMemory>(application_buffer_prealloc.offset(4));
		logical_resources::projection_matrixes::setup_parameters(uniforms, main_frame.get_extent());
	}));

	// Pipelines //
	let sc_viewport = VkViewport(0.0f32, 0.0f32, frame_width as f32, frame_height as f32, 0.0f32, 1.0f32);
	let pipelines = PipelineStates::new(&engine, use_post_smaa, enabled_pass, sc_viewport);

	// Descriptor Set //
	let uniform_memory_info = interlude::BufferInfo(&application_data, application_buffer_prealloc.offset(4) .. application_buffer_prealloc.total_size());
	let gbuffer_info = interlude::ImageInfo(gbuffer_sampler, gbuffer_view, VkImageLayout::ShaderReadOnlyOptimal);
	let edgebuffer_info = interlude::ImageInfo(gbuffer_sampler, edgebuffer_view, VkImageLayout::ShaderReadOnlyOptimal);
	let blendweight_info = interlude::ImageInfo(gbuffer_sampler, blend_weight_view, VkImageLayout::ShaderReadOnlyOptimal);
	let areatex_info = interlude::ImageInfo(gbuffer_sampler, smaa_areatex_view, VkImageLayout::ShaderReadOnlyOptimal);
	let searchtex_info = interlude::ImageInfo(gbuffer_sampler, smaa_searchtex_view, VkImageLayout::ShaderReadOnlyOptimal);
	let playerbullet_info = interlude::ImageInfo(gbuffer_sampler, playerbullet_view, VkImageLayout::ShaderReadOnlyOptimal);
	if use_post_smaa
	{
		engine.update_descriptors(&[
			interlude::DescriptorSetWriteInfo::UniformBuffer(pipelines.get_descriptor_set_for_uniform_buffer(), 0, vec![uniform_memory_info]),
			interlude::DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_smaa_edgedetect(), 0, vec![gbuffer_info.clone()]),
			interlude::DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_smaa_blendweight(), 0, vec![edgebuffer_info, areatex_info, searchtex_info]),
			interlude::DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_smaa_combine(), 0, vec![gbuffer_info, blendweight_info]),
			interlude::DescriptorSetWriteInfo::CombinedImageSampler(pipelines.get_descriptor_set_for_playerbullet_texture(), 0, vec![playerbullet_info])
		]);
	}
	else
	{
		engine.update_descriptors(&[
			interlude::DescriptorSetWriteInfo::UniformBuffer(pipelines.get_descriptor_set_for_uniform_buffer(), 0, vec![uniform_memory_info])
		]);
	}

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
			interlude::ImageMemoryBarrier::template(&**smaa_areatex, interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(&**smaa_searchtex, interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(&**playerbullet_tex, interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(smaa_areatex_stg, interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(smaa_searchtex_stg, interlude::ImageSubresourceRange::base_color()),
			interlude::ImageMemoryBarrier::template(playerbullet_tex_stg, interlude::ImageSubresourceRange::base_color())
		];
		let image_memory_barriers = main_frame.get_back_images().iter()
			.map(|x| interlude::ImageMemoryBarrier::hold_ownership(*x, interlude::ImageSubresourceRange::base_color(),
			0, VK_ACCESS_MEMORY_READ_BIT, VkImageLayout::Undefined, VkImageLayout::PresentSrcKHR)).chain([
				interlude::ImageMemoryBarrier::hold_ownership(&**gbuffer, interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				interlude::ImageMemoryBarrier::hold_ownership(&**edgebuffer, interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
				interlude::ImageMemoryBarrier::hold_ownership(&**blend_weight, interlude::ImageSubresourceRange::base_color(), VK_ACCESS_HOST_WRITE_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::Preinitialized, VkImageLayout::ColorAttachmentOptimal),
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
			.copy_image(smaa_areatex_stg, &**smaa_areatex, VkImageLayout::TransferSrcOptimal, VkImageLayout::TransferDestOptimal,
				&[interlude::ImageCopyRegion(interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), VkExtent3D(AREATEX_WIDTH, AREATEX_HEIGHT, 1))])
			.copy_image(smaa_searchtex_stg, &**smaa_searchtex, VkImageLayout::TransferSrcOptimal, VkImageLayout::TransferDestOptimal,
				&[interlude::ImageCopyRegion(interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), interlude::ImageSubresourceLayers::base_color(), VkOffset3D(0, 0, 0), VkExtent3D(SEARCHTEX_WIDTH, SEARCHTEX_HEIGHT, 1))])
			.copy_image(playerbullet_tex_stg, &**playerbullet_tex, VkImageLayout::TransferSrcOptimal, VkImageLayout::TransferDestOptimal,
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
	], enabled_pass, if use_post_smaa { 3 } else { 0 }, sc_viewport));

	info!("Recording Rendering Commands...");
	// Rendering Commands //
	let combine_commands = if use_post_smaa
	{
		let smaa_combine_descriptor_sets = [pipelines.get_descriptor_set_for_smaa_combine()];
		let smaa_combine_vertex_buffers = [(&application_data as &interlude::traits::BufferResource, application_buffer_prealloc.offset(0))];
		let combine_commands = try!(engine.allocate_bundled_command_buffers(2 * framebuffers.len() as u32));
		for (n, f) in framebuffers.iter().enumerate()
		{
			try!(combine_commands.begin(0 + 2 * n, enabled_pass, 3, f).and_then(|recorder|
				recorder
					.bind_pipeline(&pipelines.smaa.as_ref().unwrap().combine)
					.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().combine_layout, &smaa_combine_descriptor_sets)
					.bind_vertex_buffers(&smaa_combine_vertex_buffers)
					.draw(4, 1)
				.end()
			));
			try!(combine_commands.begin(1 + 2 * n, enabled_pass, 3, f).and_then(|recorder|
				/*recorder.inject_commands(|r| debug_info.inject_render_commands(r)).end()*/recorder.end()
			));
		}
		Some(combine_commands)
	}
	else { None };
	let framebuffer_commands = try!(engine.allocate_graphics_command_buffers(main_frame.get_back_images().len() as u32));
	try!(framebuffer_commands.begin_all().map(|iter| iter.map(|(i, recorder)|
	{
		if use_post_smaa
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
				.bind_descriptor_sets(&pipelines.wire_layout, &[pipelines.get_descriptor_set_for_uniform_buffer()])
				.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(1))])
				.inject_commands(|r| pipelines.background.begin(r, 0.125, 0.5, 0.1875, 0.625))
				.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::background_offs())])
				.draw(4, MAX_BK_COUNT as u32)
				.inject_commands(|r| pipelines.enemy_body.begin(r, 0.25, 0.9875, 1.5, 1.0))
				.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3))])
				.draw(4, MAX_ENEMY_COUNT as u32)
				.inject_commands(|r| pipelines.player.begin(r, 1.5, 1.25, 0.375, 1.0))
				.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::player_rot_offs())])
				.bind_index_buffer(&application_data, application_buffer_prealloc.offset(2))
				.draw_indexed(24, 2, 4)
				.inject_commands(|r| pipelines.enemy_rezonator.begin(r, 1.25, 0.5, 0.625, 1.0))
				.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(1) + structures::VertexMemoryForWireRender::enemy_rezonator_offs()),
					(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::enemy_rez_offs())])
				.draw(3, MAX_ENEMY_COUNT as u32)
				.inject_commands(|r| pipelines.playerbullet.begin(r, pipelines.get_descriptor_set_for_playerbullet_texture()))
				.bind_vertex_buffers(&[
					(&application_data, application_buffer_prealloc.offset(1) + structures::VertexMemoryForWireRender::sprite_plane_offs()),
					(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::player_bullet_offs())
				])
				.draw(4, MAX_PLAYER_BULLET_COUNT as u32)
				.next_subpass(false)
				// Pass 1 : Edge Detection(SMAA 1x) //
				.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(0))])
				.bind_pipeline(&pipelines.smaa.as_ref().unwrap().edgedetect)
				.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().edgedetect_layout, &[pipelines.get_descriptor_set_for_smaa_edgedetect()])
				.draw(4, 1)
				// .pipeline_barrier(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, false, &[], &[], &[ibar_edgebuffer_end])
				.next_subpass(false)
				// Pass 2 : Blend Weight Calculation(SMAA 1x) //
				.bind_pipeline(&pipelines.smaa.as_ref().unwrap().blendweight_calc)
				.bind_descriptor_sets(&pipelines.smaa.as_ref().unwrap().blendweight_layout, &[pipelines.get_descriptor_set_for_smaa_blendweight()])
				.draw(4, 1)
				// .pipeline_barrier(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, false, &[], &[], &[ibar_blendweight_end])
				.next_subpass(true)
				// Pass 3 : SMAA Combine and Debug Print //
				.execute_commands(&combine_commands.as_ref().unwrap()[i * 2 .. i * 2 + 1])
				.end_render_pass()
			.end().unwrap()
		}
		else
		{
			let clear_values = [
				interlude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.015625f32, 1.0f32)
			];
			let color_output_barrier = interlude::ImageMemoryBarrier::template(main_frame.get_back_images()[i], interlude::ImageSubresourceRange::base_color())
				.hold_ownership(VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, VkImageLayout::PresentSrcKHR, VkImageLayout::ColorAttachmentOptimal);

			recorder
				.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, false, &[], &[], &[color_output_barrier])
				.begin_render_pass(&framebuffers[i], &clear_values, false)
				// Pass 0 : Render to Buffer //
				.bind_descriptor_sets(&pipelines.wire_layout, &[pipelines.get_descriptor_set_for_uniform_buffer()])
				.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(1))])
				.inject_commands(|r| pipelines.background.begin(r, 0.125, 0.5, 0.1875, 0.625))
				.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::background_offs())])
				.draw(4, MAX_BK_COUNT as u32)
				.inject_commands(|r| pipelines.enemy_body.begin(r, 0.25, 0.9875, 1.5, 1.0))
				.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3))])
				.draw(4, MAX_ENEMY_COUNT as u32)
				.inject_commands(|r| pipelines.player.begin(r, 1.5, 1.25, 0.375, 1.0))
				.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::player_rot_offs())])
				.bind_index_buffer(&application_data, application_buffer_prealloc.offset(2))
				.draw_indexed(24, 2, 4)
				.inject_commands(|r| pipelines.enemy_rezonator.begin(r, 1.25, 0.5, 0.625, 1.0))
				.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(1) + structures::VertexMemoryForWireRender::enemy_rezonator_offs()),
					(&application_data, application_buffer_prealloc.offset(3) + structures::InstanceMemory::enemy_rez_offs())])
				.draw(3, MAX_ENEMY_COUNT as u32)
				.inject_commands(|r| debug_info.inject_render_commands(r))
				.end_render_pass()
			.end().unwrap()
		}
	}).collect::<Vec<_>>()));
	info!("Recording Transfer Commands...");
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

	info!("Preparing for Render Loop...");

	let _/*engine*/ = {
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
				main_frame.acquire_next_backbuffer_index(&rendering_order_sem).and_then(|findex|
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
					main_frame.present(engine.graphics_queue_ref(), frame_index).and_then(|()|
					main_frame.acquire_next_backbuffer_index(&rendering_order_sem).and_then(|findex|
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

		let mapped_range = try!(appdata_stage.map());
		let (uref_enemy, uref_bk, uref_player_center) =
		{
			let mapped = mapped_range.map_mut::<structures::UniformMemory>(application_buffer_prealloc.offset(4));
			(&mut mapped.enemy_instance_data, &mut mapped.background_instance_data, &mut mapped.player_center_tf)
		};
		let (iref_enemy, iref_bk, iref_player, iref_enemy_rez, iref_player_bullet, iref_lineburst_particle_groups) =
		{
			let mapped = mapped_range.map_mut::<structures::InstanceMemory>(application_buffer_prealloc.offset(3));
			(&mut mapped.enemy_instance_mult, &mut mapped.background_instance_mult, &mut mapped.player_rotq,
				&mut mapped.enemy_rez_instance_data, &mut mapped.player_bullet_offset_sincos, &mut mapped.lineburst_particle_groups)
		};
		let mut background_datastore = logical_resources::BackgroundDatastore::new(uref_bk, iref_bk);
		let mut enemy_datastore = logical_resources::EnemyDatastore::new(iref_enemy);
		let mut pb_memory_manager = utils::MemoryBlockManager::new(MAX_PLAYER_BULLET_COUNT as u32);
		let mut lineburst_particles = logical_resources::LineBurstParticles::new(iref_lineburst_particle_groups);

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
		loop
		{
			match event_receiver.recv().unwrap()
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
