use interlude::*;
use interlude::ffi::*;

pub struct RenderPasses
{
	pub object: RenderPass,
	pub content_render_pass: u32,
	pub tonemap_pass: u32,
	pub smaa_edge_pass: u32,
	pub smaa_weight_pass: u32,
	pub smaa_combine_pass: u32,
	pub required_image_count: usize
}
impl RenderPasses
{
	pub fn new(engine: &Engine, sc_format: VkFormat) -> Self
	{
		// Attachment Descriptions //
		let attachments = [
			AttachmentDesc
			{
				format: VkFormat::R16G16B16A16_SFLOAT, clear_on_load: Some(true), preserve_stored_value: false,
				initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ShaderReadOnlyOptimal,
				.. Default::default()
			},
			AttachmentDesc
			{
				format: VkFormat::R8G8B8A8_UNORM, clear_on_load: None, preserve_stored_value: false,
				initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
				.. Default::default()
			},
			AttachmentDesc
			{
				format: VkFormat::R8G8_UNORM, clear_on_load: Some(true), preserve_stored_value: false,
				initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
				.. Default::default()
			},
			AttachmentDesc
			{
				format: VkFormat::R8G8B8A8_UNORM, clear_on_load: Some(true), preserve_stored_value: false,
				initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
				.. Default::default()
			},
			AttachmentDesc::swapchain_buffer(sc_format)
		];
		let ai_sfloat4 = 0;
		let ai_unorm4f = 1;
		let ai_unorm2 = 2;
		let ai_unorm4 = 3;
		let ai_final = 4;
		// Pass Descriptions //
		let passes =  [
			PassDesc::single_fragment_output(ai_sfloat4),
			PassDesc { input_attachment_indices: vec![AttachmentRef::input(ai_sfloat4)], color_attachment_indices: vec![AttachmentRef::color(ai_unorm4f)], .. Default::default() },
			PassDesc { color_attachment_indices: vec![AttachmentRef::color(ai_unorm2)], preserved_attachment_indices: vec![ai_unorm4], .. Default::default() },
			PassDesc { color_attachment_indices: vec![AttachmentRef::color(ai_unorm4)], preserved_attachment_indices: vec![ai_unorm4], .. Default::default() },
			PassDesc::single_fragment_output(ai_final)
		];
		let p_content = 0;
		let p_tonemap = 1;
		let p_smaa_edge = 2;
		let p_smaa_weight = 3;
		let p_smaa_combine = 4;
		// Pass Dependencies //
		let deps = [
			PassDependency::fragment_referer(p_content, p_tonemap, true),
			PassDependency::fragment_referer(p_tonemap, p_smaa_edge, false),
			PassDependency::fragment_referer(p_smaa_edge, p_smaa_weight, false),
			PassDependency::fragment_referer(p_tonemap, p_smaa_combine, false),
			PassDependency::fragment_referer(p_smaa_weight, p_smaa_combine, false)
		];

		// Objects //
		let fullpass = engine.create_render_pass(&attachments, &passes, &deps).or_crash();
		
		RenderPasses
		{
			object: fullpass,
			content_render_pass: p_content, tonemap_pass: p_tonemap, smaa_edge_pass: p_smaa_edge, smaa_weight_pass: p_smaa_weight, smaa_combine_pass: p_smaa_combine,
			required_image_count: attachments.len()
		}
	}
}
