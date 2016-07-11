use render_vk::wrap as vk;
use vkffi::*;
use std;

pub struct UniformBufferDescriptorPool<'d>
{
	pub pool: vk::DescriptorPool<'d>, pub layout: vk::DescriptorSetLayout<'d>
}
impl <'d> UniformBufferDescriptorPool<'d>
{
	pub fn new(device: &'d vk::Device, max_sets: u32) -> Self
	{
		let pool = device.create_descriptor_pool(max_sets, &[VkDescriptorPoolSize(VkDescriptorType::UniformBuffer, 1)]).unwrap();
		let layout_bindings = [
			VkDescriptorSetLayoutBinding
			{
				binding: 0, descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
				stageFlags: VK_SHADER_STAGE_VERTEX_BIT, pImmutableSamplers: std::ptr::null()
			}
		];
		let layout = device.create_descriptor_set_layout(&layout_bindings).unwrap();

		UniformBufferDescriptorPool { pool: pool, layout: layout }
	}
}
