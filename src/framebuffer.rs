use interlude;
use interlude::ffi::*;

pub struct RenderPasses
{
	pub noaa: interlude::RenderPass, pub fullset: interlude::RenderPass,
}
impl RenderPasses
{
	pub fn new(engine: &interlude::Engine, sc_format: VkFormat) -> Self
	{
		// Attachment Descriptions //
		let gbuffer_desc = interlude::AttachmentDesc
		{
			format: VkFormat::R8G8B8A8_UNORM, clear_on_load: Some(true), preserve_stored_value: false,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
			.. Default::default()
		};
		let edgebuffer_desc = interlude::AttachmentDesc
		{
			format: VkFormat::R8G8_UNORM, clear_on_load: Some(true), preserve_stored_value: false,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
			.. Default::default()
		};
		let blendweight_buffer_desc = interlude::AttachmentDesc
		{
			format: VkFormat::R8G8B8A8_UNORM, clear_on_load: Some(true), preserve_stored_value: false,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::ColorAttachmentOptimal,
			.. Default::default()
		};
		let swapchain_buffer_desc = interlude::AttachmentDesc::swapchain_buffer(sc_format);

		// Render Pass Descriptiones //
		let content_render_pass = interlude::PassDesc::single_fragment_output(0);
		let aa_edgedetect_pass = interlude::PassDesc
		{
			color_attachment_indices: vec![interlude::AttachmentRef::color(1)], preserved_attachment_indices: vec![0],
			.. Default::default()
		};
		let aa_blendweight_pass = interlude::PassDesc
		{
			color_attachment_indices: vec![interlude::AttachmentRef::color(2)], preserved_attachment_indices: vec![0, 1],
			.. Default::default()
		};
		let aa_combine_pass = interlude::PassDesc::single_fragment_output(3);

		// Pass Dependencies //
		let deps_content_to_edge = interlude::PassDependency::fragment_referer(0, 1, false);
		let deps_content_to_combine = interlude::PassDependency::fragment_referer(0, 3, false);
		let deps_edge_to_blend = interlude::PassDependency::fragment_referer(1, 2, false);
		let deps_blend_to_combine = interlude::PassDependency::fragment_referer(2, 3, false);

		// Objects //
		let fullpass = Unrecoverable!(engine.create_render_pass(
			&[gbuffer_desc, edgebuffer_desc, blendweight_buffer_desc, swapchain_buffer_desc],
			&[content_render_pass.clone(), aa_edgedetect_pass, aa_blendweight_pass, aa_combine_pass],
			&[deps_content_to_edge, deps_content_to_combine, deps_edge_to_blend, deps_blend_to_combine]
		));
		let noaa_pass = Unrecoverable!(engine.create_render_pass(&[swapchain_buffer_desc], &[content_render_pass], &[]));
		
		RenderPasses
		{
			noaa: noaa_pass, fullset: fullpass
		}
	}
}
