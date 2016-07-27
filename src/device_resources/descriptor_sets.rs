use std;
use vkffi::*;
use render_vk::wrap as vk;

pub struct DescriptorSets<'d>
{
	#[allow(dead_code)] pool: vk::DescriptorPool<'d>,
	pub set_layout_ub1_vg: vk::DescriptorSetLayout<'d>,
	pub set_layout_ub1: vk::DescriptorSetLayout<'d>,
	pub set_layout_s1: vk::DescriptorSetLayout<'d>,
	pub set_layout_ub1_g: vk::DescriptorSetLayout<'d>,
	pub sets: vk::DescriptorSets<'d>
}
impl <'d> DescriptorSets<'d>
{
	pub fn new(device: &'d vk::Device) -> Self
	{
		let pool = device.create_descriptor_pool(4, &[
			VkDescriptorPoolSize(VkDescriptorType::UniformBuffer, 4), VkDescriptorPoolSize(VkDescriptorType::CombinedImageSampler, 1)
		]).unwrap();
		let ub1_vg_set_layout_bindings = VkDescriptorSetLayoutBinding
		{
			binding: 0, descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			stageFlags: VK_SHADER_STAGE_VERTEX_BIT | VK_SHADER_STAGE_GEOMETRY_BIT, pImmutableSamplers: std::ptr::null()
		};
		let ub1_set_layout_bindings = VkDescriptorSetLayoutBinding
		{
			binding: 0, descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			stageFlags: VK_SHADER_STAGE_VERTEX_BIT, pImmutableSamplers: std::ptr::null()
		};
		let s1_set_layout_bindings = VkDescriptorSetLayoutBinding
		{
			binding: 0, descriptorType: VkDescriptorType::CombinedImageSampler, descriptorCount: 1,
			stageFlags: VK_SHADER_STAGE_FRAGMENT_BIT, pImmutableSamplers: std::ptr::null()
		};
		let ub1_g_set_layout_bindings = VkDescriptorSetLayoutBinding
		{
			binding: 0, descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
			stageFlags: VK_SHADER_STAGE_GEOMETRY_BIT, pImmutableSamplers: std::ptr::null()
		};
		let layout_ub1_vg = device.create_descriptor_set_layout(&[ub1_vg_set_layout_bindings]).unwrap();
		let layout = device.create_descriptor_set_layout(&[ub1_set_layout_bindings]).unwrap();
		let layout_s1 = device.create_descriptor_set_layout(&[s1_set_layout_bindings]).unwrap();
		let layout_ub1_g = device.create_descriptor_set_layout(&[ub1_g_set_layout_bindings]).unwrap();
		let sets = pool.allocate_sets(&[*layout_ub1_vg, *layout_ub1_g, *layout_ub1_g, *layout_s1]).unwrap();

		DescriptorSets
		{
			pool: pool,
			set_layout_ub1_vg: layout_ub1_vg, set_layout_ub1: layout,
			set_layout_s1: layout_s1, set_layout_ub1_g: layout_ub1_g,
			sets: sets
		}
	}
}
