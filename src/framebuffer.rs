use interlude::*;
use interlude::ffi::*;

pub struct RenderPasses
{
	pub normal_render: RenderPass, pub smaa_edgedetect: RenderPass, pub smaa_blendweight: RenderPass, pub smaa_combine: RenderPass
}
impl RenderPasses
{
	pub fn new<Engine: EngineCore>(engine: &Engine, sc_format: VkFormat) -> Self
	{
		// Attachment Descriptions //
		let a_render = AttachmentDesc
		{
			format: VkFormat::R16G16B16A16_SFLOAT, clear_on_load: Some(true), preserve_stored_value: false,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
			.. Default::default()
		};
		let a_tonemap_out = AttachmentDesc
		{
			format: VkFormat::R8G8B8A8_UNORM, clear_on_load: None, preserve_stored_value: true,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ShaderReadOnlyOptimal,
			.. Default::default()
		};
		let a_smaa_edgedetect_out = AttachmentDesc
		{
			format: VkFormat::R8G8_UNORM, clear_on_load: Some(true), preserve_stored_value: true,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ShaderReadOnlyOptimal,
			.. Default::default()
		};
		let a_smaa_blendweight_out = AttachmentDesc
		{
			format: VkFormat::R8G8B8A8_UNORM, clear_on_load: Some(true), preserve_stored_value: true,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ShaderReadOnlyOptimal,
			.. Default::default()
		};
		let a_swapchain = AttachmentDesc
		{
			format: sc_format, clear_on_load: None, preserve_stored_value: true,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::PresentSrcKHR,
			.. Default::default()
		};

		// Pass Descriptions //
		let normal_render_pass = PassDesc::single_fragment_output(0);
		let tonemap_pass = PassDesc { input_attachment_indices: vec![AttachmentRef::input(0)], color_attachment_indices: vec![AttachmentRef::color(1)], .. Default::default() };
		let smaa_pass = PassDesc::single_fragment_output(0);

		// Pass Dependencies //
		let rr_tonemap_dep = PassDependency
		{
			src: 0, dst: 1,
			src_stage_mask: VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, dst_stage_mask: VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
			src_access_mask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT, dst_access_mask: VK_ACCESS_SHADER_READ_BIT,
			depend_by_region: true
		};

		// objects //
		RenderPasses
		{
			normal_render: engine.create_render_pass(&[&a_render, &a_tonemap_out], &[&normal_render_pass, &tonemap_pass], &[&rr_tonemap_dep]).or_crash(),
			smaa_edgedetect: engine.create_render_pass(&[&a_smaa_edgedetect_out], &[&smaa_pass], &[]).or_crash(),
			smaa_blendweight: engine.create_render_pass(&[&a_smaa_blendweight_out], &[&smaa_pass], &[]).or_crash(),
			smaa_combine: engine.create_render_pass(&[&a_swapchain], &[&smaa_pass], &[]).or_crash()
		}
	}
}

pub struct Framebuffers
{
	pub normal_render: Framebuffer, pub smaa_edgedetect: Framebuffer, pub smaa_blendweight: Framebuffer, pub final_output: Vec<Framebuffer>
}
impl Framebuffers
{
	pub fn new<Engine: EngineCore>(engine: &Engine, molds: &RenderPasses, nr_view: &ImageView2D, tonemap_out_view: &ImageView2D,
		smaa_edgedetect_out_view: &ImageView2D, smaa_blendweight_out_view: &ImageView2D, swapchain_views: &[&WindowRenderTarget], size: &Size2) -> Self
	{
		let &Size2(w, h) = size;
		let fsz = Size3(w, h, 1);

		Framebuffers
		{
			normal_render: engine.create_framebuffer(&molds.normal_render, &[nr_view, tonemap_out_view], &fsz).or_crash(),
			smaa_edgedetect: engine.create_framebuffer(&molds.smaa_edgedetect, &[smaa_edgedetect_out_view], &fsz).or_crash(),
			smaa_blendweight: engine.create_framebuffer(&molds.smaa_blendweight, &[smaa_blendweight_out_view], &fsz).or_crash(),
			final_output: swapchain_views.into_iter().map(|&v| engine.create_framebuffer(&molds.smaa_combine, &[v], &fsz))
				.collect::<Result<_, _>>().or_crash()
		}
	}
}
