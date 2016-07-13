use std;
use vkffi::*;
use render_vk::wrap as vk;
use render_vk::traits::*;
use traits::*;

pub struct DescriptorSets<'d>
{
	pool: vk::DescriptorPool<'d>, pub set_layout_ub1: vk::DescriptorSetLayout<'d>,
	pub sets: vk::DescriptorSets<'d>
}
impl <'d> DescriptorSets<'d>
{
	pub fn new(device: &'d vk::Device) -> Self
	{
		let pool = device.create_descriptor_pool(2, &[VkDescriptorPoolSize(VkDescriptorType::UniformBuffer, 1)]).unwrap();
		let layout_bindings = [
			VkDescriptorSetLayoutBinding
			{
				binding: 0, descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
				stageFlags: VK_SHADER_STAGE_VERTEX_BIT, pImmutableSamplers: std::ptr::null()
			}
		];
		let layout = device.create_descriptor_set_layout(&layout_bindings).unwrap();
		let sets = pool.allocate_sets(&[layout.get(), layout.get()]).unwrap();

		DescriptorSets
		{
			pool: pool, set_layout_ub1: layout, sets: sets
		}
	}
}