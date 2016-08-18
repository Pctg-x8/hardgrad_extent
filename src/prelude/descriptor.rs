// Prelude: Descriptor and its layouts in shader

use prelude::internals::*;
use vkffi::*;
use render_vk::wrap as vk;

pub struct DescriptorSetLayout { internal: vk::DescriptorSetLayout }
pub trait DescriptorSetLayoutInternals
{
	fn new(dsl: vk::DescriptorSetLayout) -> Self;
}
impl DescriptorSetLayoutInternals for DescriptorSetLayout
{
	fn new(dsl: vk::DescriptorSetLayout) -> Self
	{
		DescriptorSetLayout { internal: dsl }
	}
}
impl InternalExports<vk::DescriptorSetLayout> for DescriptorSetLayout
{
	fn get_internal(&self) -> &vk::DescriptorSetLayout { &self.internal }
}
