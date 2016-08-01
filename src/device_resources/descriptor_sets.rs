use std;
use vkffi::*;
use render_vk::wrap as vk;

pub struct DescriptorSets<'d>
{
	#[allow(dead_code)] pool: vk::DescriptorPool<'d>,
	pub set_layout_uniform_vg: vk::DescriptorSetLayout<'d>,
	pub set_layout_s1: vk::DescriptorSetLayout<'d>,
	pub sets: vk::DescriptorSets<'d>
}
impl <'d> DescriptorSets<'d>
{
	pub fn new(device: &'d vk::Device) -> Self
	{
		let pool = device.create_descriptor_pool(4, &[
			VkDescriptorPoolSize(VkDescriptorType::UniformBuffer, 4), VkDescriptorPoolSize(VkDescriptorType::CombinedImageSampler, 1)
		]).unwrap();
		let uniform_vg_set_layout_bindings = VkDescriptorSetLayoutBinding
		{
			binding: 0, descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			stageFlags: VK_SHADER_STAGE_VERTEX_BIT | VK_SHADER_STAGE_GEOMETRY_BIT, pImmutableSamplers: std::ptr::null()
		};
		let s1_set_layout_bindings = VkDescriptorSetLayoutBinding
		{
			binding: 0, descriptorType: VkDescriptorType::CombinedImageSampler, descriptorCount: 1,
			stageFlags: VK_SHADER_STAGE_FRAGMENT_BIT, pImmutableSamplers: std::ptr::null()
		};
		let layout_uniform_vg = device.create_descriptor_set_layout(&[uniform_vg_set_layout_bindings]).unwrap();
		let layout_s1 = device.create_descriptor_set_layout(&[s1_set_layout_bindings]).unwrap();
		let sets = pool.allocate_sets(&[*layout_uniform_vg, *layout_s1]).unwrap();

		DescriptorSets
		{
			pool: pool,
			set_layout_uniform_vg: layout_uniform_vg, set_layout_s1: layout_s1,
			sets: sets
		}
	}
}
