// RenderState(PipelineState)

use std::rc::Rc;
use std::mem::size_of;
use interlude::*;
use interlude::ffi::*;
use assets::*;
use framebuffer::*;
use std::ops::Deref;
use Viewports;

pub struct Layouts
{
	pub global_uniform_layout: DescriptorSetLayout,
	pub texture_layout: DescriptorSetLayout, pub texture_layout_geom: DescriptorSetLayout, pub texture_layout_vert: DescriptorSetLayout,
	pub ainput_layout: DescriptorSetLayout,
	pub ainput_require_layout: PipelineLayout,
	pub wire_pipeline_layout: Rc<PipelineLayout>, pub lineburst_particle_layout: PipelineLayout, pub sprite_layout: Rc<PipelineLayout>,
	pub bullet_layout: Rc<PipelineLayout>
}
impl Layouts
{
	fn new(engine: &GraphicsInterface) -> Self
	{
		let gu_layout = DescriptorSetLayout::new(engine, vec![
			Descriptor::Uniform(1, ShaderStage::Vertex | ShaderStage::Geometry), Descriptor::Storage(1, ShaderStage::Vertex)].into()).or_crash();
		let t_layout = DescriptorSetLayout::new(engine, vec![Descriptor::CombinedSampler(1, ShaderStage::Fragment)].into()).or_crash();
		let t_layout_g = DescriptorSetLayout::new(engine, vec![Descriptor::CombinedSampler(1, ShaderStage::Geometry)].into()).or_crash();
		let t_layout_v = DescriptorSetLayout::new(engine, vec![Descriptor::CombinedSampler(1, ShaderStage::Vertex)].into()).or_crash();
		let ainput_layout = DescriptorSetLayout::new(engine, vec![Descriptor::InputAttachment(1, ShaderStage::Fragment)].into()).or_crash();
		
		Layouts
		{
			ainput_require_layout: PipelineLayout::new(engine, &[&ainput_layout], &[]).or_crash(),
			wire_pipeline_layout: Rc::new(PipelineLayout::new(engine, &[&gu_layout],
				&[&PushConstantDesc(VK_SHADER_STAGE_VERTEX_BIT, 0 .. size_of::<CVector4>() as u32)]).or_crash()),
			lineburst_particle_layout: PipelineLayout::new(engine, &[&gu_layout, &t_layout_g], &[]).or_crash(),
			sprite_layout: Rc::new(PipelineLayout::new(engine, &[&gu_layout, &t_layout], &[]).or_crash()),
			bullet_layout: Rc::new(PipelineLayout::new(engine, &[&gu_layout, &t_layout, &t_layout_v], &[]).or_crash()),
			global_uniform_layout: gu_layout, texture_layout: t_layout, texture_layout_geom: t_layout_g, texture_layout_vert: t_layout_v,
			ainput_layout: ainput_layout
		}
	}
}
pub struct DescriptorSetBindings
{
	#[allow(dead_code)] sets: DescriptorSets,
	pub global_uniform: VkDescriptorSet,
	pub playerbullet_texture: VkDescriptorSet,
	pub bullet_texture: VkDescriptorSet,
	pub lineburst_particle_color: VkDescriptorSet,
	pub bullet_color: VkDescriptorSet,
	pub tonemap_input: VkDescriptorSet,
	pub smaa_edgedetect: VkDescriptorSet,
	pub smaa_blendweight: VkDescriptorSet,
	pub smaa_combine: VkDescriptorSet
}
pub struct PipelineStates
{
	#[allow(dead_code)] shaderstore: ShaderStore, layouts: Layouts,
	pub background: WireRender, pub enemy_body: WireRender, pub enemy_rezonator: WireRender, pub player: WireRender,
	pub playerbullet: SpriteRender, pub lineburst: GraphicsPipeline, pub bullet: SpriteRender,
	pub tonemapper: GraphicsPipeline, pub smaa: Option<SMAAPipelineStates>,
	pub descriptor_sets: DescriptorSetBindings
}
impl PipelineStates
{
	pub fn new<Engine: AssetProvider + Deref<Target = GraphicsInterface>>(engine: &Engine, use_smaa: bool, passes: &RenderPasses, viewports: &Viewports)
		-> Self
	{
		let shaderstore = ShaderStore::new(engine);
		let layouts = Layouts::new(engine);
		let normal_render_pass = PreciseRenderPass(&passes.normal_render, 0);

		let mut gps =
		{
			let background_ps = GraphicsPipelineBuilder::new(&layouts.wire_pipeline_layout, normal_render_pass.clone())
				.vertex_shader(PipelineShaderProgram::unspecialized(&shaderstore.geometry_preinstancing_vsh))
				.geometry_shader(PipelineShaderProgram::unspecialized(&shaderstore.background_duplication_gsh))
				.fragment_shader(PipelineShaderProgram::unspecialized(&shaderstore.solid_fsh))
				.primitive_topology(PrimitiveTopology::LineList(true))
				.viewport_scissors(&[ViewportWithScissorRect::default_scissor(&viewports.game)])
				.blend_state(&[AttachmentBlendState::PremultipliedAlphaBlend]);
			let enemy_ps = GraphicsPipelineBuilder::inherit(&background_ps)
				.geometry_shader(PipelineShaderProgram::unspecialized(&shaderstore.enemy_duplication_gsh))
				.blend_state(&[AttachmentBlendState::Disabled]);
			let enemy_rezonator_ps = GraphicsPipelineBuilder::inherit(&enemy_ps)
				.vertex_shader(PipelineShaderProgram::unspecialized(&shaderstore.erz_preinstancing_vsh))
				.geometry_shader(PipelineShaderProgram::unspecialized(&shaderstore.enemy_rezonator_duplication_gsh))
				.primitive_topology(PrimitiveTopology::TriangleList(false));
			let player_ps = GraphicsPipelineBuilder::new(&layouts.wire_pipeline_layout, normal_render_pass.clone())
				.vertex_shader(PipelineShaderProgram::unspecialized(&shaderstore.player_rotate_vsh))
				.fragment_shader(PipelineShaderProgram::unspecialized(&shaderstore.solid_fsh))
				.primitive_topology(PrimitiveTopology::LineList(false))
				.viewport_scissors(&[ViewportWithScissorRect::default_scissor(&viewports.game)])
				.blend_state(&[AttachmentBlendState::Disabled]);
			let playerbullet_ps = GraphicsPipelineBuilder::new(&layouts.sprite_layout, normal_render_pass.clone())
				.vertex_shader(PipelineShaderProgram(shaderstore.playerbullet_vsh.clone(), vec![(0, ConstantEntry::Float(0.75))]))
				.fragment_shader(PipelineShaderProgram::unspecialized(&shaderstore.sprite_fsh))
				.primitive_topology(PrimitiveTopology::TriangleStrip(false))
				.viewport_scissors(&[ViewportWithScissorRect::default_scissor(&viewports.game)])
				.blend_state(&[AttachmentBlendState::PremultipliedAlphaBlend]);
			let lineburst_ps = GraphicsPipelineBuilder::new(&layouts.lineburst_particle_layout, normal_render_pass.clone())
				.vertex_shader(PipelineShaderProgram::unspecialized(&shaderstore.lineburst_particle_vsh))
				.geometry_shader(PipelineShaderProgram::unspecialized(&shaderstore.lineburst_particle_instantiate_gsh))
				.fragment_shader(PipelineShaderProgram::unspecialized(&shaderstore.solid_fsh))
				.primitive_topology(PrimitiveTopology::Point)
				.viewport_scissors(&[ViewportWithScissorRect::default_scissor(&viewports.game)])
				.blend_state(&[AttachmentBlendState::PremultipliedAlphaBlend]);
			let tonemapper_ps = GraphicsPipelineBuilder::for_postprocess(engine, &layouts.ainput_require_layout, PreciseRenderPass(&passes.normal_render, 1),
				PipelineShaderProgram::unspecialized(&shaderstore.tonemap_fsh), &viewports.game).or_crash()
				.vertex_shader(PipelineShaderProgram::unspecialized(engine.postprocess_vsh(false).or_crash()));
			let bullet_ps = GraphicsPipelineBuilder::new(&layouts.bullet_layout, normal_render_pass.clone())
				.vertex_shader(PipelineShaderProgram(shaderstore.bullet_vsh.clone(), vec![(0, ConstantEntry::Float(0.6875))]))
				.fragment_shader(PipelineShaderProgram::unspecialized(&shaderstore.colored_sprite_fsh))
				.primitive_topology(PrimitiveTopology::TriangleStrip(false))
				.viewport_scissors(&[ViewportWithScissorRect::default_scissor(&viewports.game)])
				.blend_state(&[AttachmentBlendState::PremultipliedAlphaBlend]);
			GraphicsPipelines::new(engine, &[&background_ps, &enemy_ps, &enemy_rezonator_ps,
				&player_ps, &playerbullet_ps, &lineburst_ps, &tonemapper_ps, &bullet_ps]).or_crash()
		};
		let bullet_sr = SpriteRender::new(gps.pop().unwrap(), &layouts.bullet_layout);
		let tonemap_ps = gps.pop().unwrap();
		let lineburst_ps = gps.pop().unwrap();
		let playerbullet_sr = SpriteRender::new(gps.pop().unwrap(), &layouts.sprite_layout);
		let player_wr = WireRender::new(gps.pop().unwrap(), &layouts.wire_pipeline_layout);
		let enemy_rezonator_wr = WireRender::new(gps.pop().unwrap(), &layouts.wire_pipeline_layout);
		let enemy_wr = WireRender::new(gps.pop().unwrap(), &layouts.wire_pipeline_layout);
		let background_wr = WireRender::new(gps.pop().unwrap(), &layouts.wire_pipeline_layout);
		assert_eq!(gps.len(), 0);

		let (smaa, descriptor_sets) = if use_smaa
		{
			let ps = SMAAPipelineStates::new(engine, &passes, viewports);
			DescriptorSets::preallocate(engine, &[
				&layouts.global_uniform_layout, &layouts.texture_layout, &layouts.texture_layout,
				&layouts.texture_layout_geom, &layouts.texture_layout_vert, &layouts.ainput_layout,
				&ps.descriptor_sets[0], &ps.descriptor_sets[1], &ps.descriptor_sets[2]
			]).map(|dslist| (Some(ps), dslist))
		}
		else
		{
			DescriptorSets::preallocate(engine, &[
				&layouts.global_uniform_layout, &layouts.texture_layout, &layouts.texture_layout,
				&layouts.texture_layout_geom, &layouts.texture_layout_vert, &layouts.ainput_layout
			]).map(|dslist| (None, dslist))
		}.or_crash();

		PipelineStates
		{
			shaderstore: shaderstore, layouts: layouts,
			background: background_wr, enemy_body: enemy_wr, enemy_rezonator: enemy_rezonator_wr, player: player_wr, playerbullet: playerbullet_sr,
			lineburst: lineburst_ps, bullet: bullet_sr,
			tonemapper: tonemap_ps, smaa: smaa, descriptor_sets: DescriptorSetBindings
			{
				global_uniform: descriptor_sets[0],
				playerbullet_texture: descriptor_sets[1],
				bullet_texture: descriptor_sets[2],
				lineburst_particle_color: descriptor_sets[3],
				bullet_color: descriptor_sets[4],
				tonemap_input: descriptor_sets[5],
				smaa_edgedetect: descriptor_sets[6],
				smaa_blendweight: descriptor_sets[7],
				smaa_combine: descriptor_sets[8],
				sets: descriptor_sets
			}
		}
	}
	
	// readonly exporter
	pub fn layout_for_attachment_input(&self) -> &PipelineLayout { &self.layouts.ainput_require_layout }
	pub fn layout_for_wire_render(&self) -> &PipelineLayout { &self.layouts.wire_pipeline_layout }
	pub fn layout_for_lineburst_particle_render(&self) -> &PipelineLayout { &self.layouts.lineburst_particle_layout }
	pub fn layout_for_bullet(&self) -> &PipelineLayout { &self.layouts.bullet_layout }
}

// Wire Render Wrapper with moving pipeline state object
pub struct WireRender
{
	renderstate: GraphicsPipeline, layout_ref: Rc<PipelineLayout>
}
impl WireRender
{
	pub fn new(renderstate: GraphicsPipeline, layout: &Rc<PipelineLayout>) -> Self
	{
		WireRender { renderstate: renderstate, layout_ref: layout.clone() }
	}
	pub fn begin<RecorderT>(&self, comrec: RecorderT, wirecolor_r: f32, wirecolor_g: f32, wirecolor_b: f32, wirecolor_a: f32) -> RecorderT
		where RecorderT: DrawingCommandRecorder
	{
		comrec.bind_pipeline(&self.renderstate).push_constants(&self.layout_ref, ShaderStage::Vertex,
			0 .. size_of::<CVector4>() as u32, &[wirecolor_r, wirecolor_g, wirecolor_b, wirecolor_a])
	}
}
// Sprite Render with moving pipeline state object
pub struct SpriteRender
{
	renderstate: GraphicsPipeline, layout_ref: Rc<PipelineLayout>
}
impl SpriteRender
{
	pub fn new(renderstate: GraphicsPipeline, layout: &Rc<PipelineLayout>) -> Self
	{
		SpriteRender { renderstate: renderstate, layout_ref: layout.clone() }
	}
	pub fn begin<RecorderT>(&self, comrec: RecorderT, texture_ds: VkDescriptorSet) -> RecorderT
		where RecorderT: DrawingCommandRecorder
	{
		comrec.bind_pipeline(&self.renderstate).bind_descriptor_sets_partial(&self.layout_ref, 1, &[texture_ds])
	}
}

pub struct SMAAPipelineStates
{
	#[allow(dead_code)] shaders: (Rc<VertexShader>, Rc<VertexShader>, Rc<VertexShader>, Rc<FragmentShader>, Rc<FragmentShader>, Rc<FragmentShader>),
	descriptor_sets: [DescriptorSetLayout; 3],
	pub edgedetect_layout: PipelineLayout, pub blendweight_layout: PipelineLayout, pub combine_layout: PipelineLayout,
	pub edgedetect: GraphicsPipeline, pub blendweight_calc: GraphicsPipeline, pub combine: GraphicsPipeline
}
impl SMAAPipelineStates
{
	pub fn new<Engine: AssetProvider + Deref<Target = GraphicsInterface>>(engine: &Engine, render_passes: &RenderPasses, viewports: &Viewports) -> Self
	{
		let Viewport(_, _, vw, vh, _, _) = viewports.game;

		let evsh = VertexShader::from_asset_for_postprocessing(engine, "shaders.smaa.EdgeDetectionV", "main").or_crash();
		let bwvsh = VertexShader::from_asset_for_postprocessing(engine, "shaders.smaa.BlendWeightCalcV", "main").or_crash();
		let cvsh = VertexShader::from_asset_for_postprocessing(engine, "shaders.smaa.CombineV", "main").or_crash();
		let esh = FragmentShader::from_asset(engine, "shaders.smaa.EdgeDetection", "main").or_crash();
		let bwsh = FragmentShader::from_asset(engine, "shaders.smaa.BlendWeightCalc", "main").or_crash();
		let csh = FragmentShader::from_asset(engine, "shaders.smaa.Combine", "main").or_crash();

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
				PipelineShaderProgram::unspecialized(&esh), &viewports.game).or_crash()
				.vertex_shader(PipelineShaderProgram(evsh.clone(), scons_rt_metrics.clone()));
			let bwps = GraphicsPipelineBuilder::for_postprocess(engine, &bwpl, PreciseRenderPass(&render_passes.smaa_blendweight, 0),
				PipelineShaderProgram(bwsh.clone(), scons_rt_metrics.clone()), &viewports.game).or_crash()
				.vertex_shader(PipelineShaderProgram(bwvsh.clone(), scons_rt_metrics.clone()));
			let cps = GraphicsPipelineBuilder::for_postprocess(engine, &cpl, PreciseRenderPass(&render_passes.smaa_combine, 0),
				PipelineShaderProgram(csh.clone(), scons_rt_metrics.clone()), &viewports.game).or_crash()
				.vertex_shader(PipelineShaderProgram(cvsh.clone(), scons_rt_metrics));
			GraphicsPipelines::new(engine, &[&eps, &bwps, &cps]).or_crash()
		};
		let cpso = gps.pop().unwrap();
		let bwpso = gps.pop().unwrap();
		let epso = gps.pop().unwrap();
		assert_eq!(gps.len(), 0);

		SMAAPipelineStates
		{
			shaders: (evsh, bwvsh, cvsh, esh, bwsh, csh),
			descriptor_sets: dss, edgedetect_layout: epl, blendweight_layout: bwpl, combine_layout: cpl,
			edgedetect: epso, blendweight_calc: bwpso, combine: cpso
		}
	}
}
