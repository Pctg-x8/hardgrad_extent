// Prelude: Descriptor and its layouts in shader

use prelude::internals::*;
use std;
use vkffi::*;
use render_vk::wrap as vk;

#[derive(Clone, Copy)]
pub enum ShaderStage { Vertex, TessControl, TessEvaluate, Geometry, Fragment }
impl std::convert::Into<VkShaderStageFlags> for ShaderStage
{
	fn into(self) -> VkShaderStageFlags
	{
		match self
		{
			ShaderStage::Vertex => VK_SHADER_STAGE_VERTEX_BIT,
			ShaderStage::TessControl => VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT,
			ShaderStage::TessEvaluate => VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT,
			ShaderStage::Geometry => VK_SHADER_STAGE_GEOMETRY_BIT,
			ShaderStage::Fragment => VK_SHADER_STAGE_FRAGMENT_BIT
		}
	}
}

#[derive(Clone)]
pub enum Descriptor
{
	Uniform(u32, Vec<ShaderStage>),
	CombinedSampler(u32, Vec<ShaderStage>)
}
pub trait DescriptorInternals
{
	fn count(&self) -> u32;
	fn into_binding(&self, index: u32) -> VkDescriptorSetLayoutBinding;
	fn into_pool_size(&self) -> VkDescriptorPoolSize;
}
impl Descriptor
{
	fn native_type(&self) -> VkDescriptorType
	{
		match self
		{
			&Descriptor::Uniform(_, _) => VkDescriptorType::UniformBuffer,
			&Descriptor::CombinedSampler(_, _) => VkDescriptorType::CombinedImageSampler
		}
	}
	fn stage_mask(&self) -> VkShaderStageFlags
	{
		match self
		{
			&Descriptor::Uniform(_, ref s) => s,
			&Descriptor::CombinedSampler(_, ref s) => s
		}.iter().fold(0, |flag, f| flag | Into::<VkShaderStageFlags>::into(*f))
	}
}
impl DescriptorInternals for Descriptor
{
	fn count(&self) -> u32
	{
		match self
		{
			&Descriptor::Uniform(n, _) => n,
			&Descriptor::CombinedSampler(n, _) => n
		}
	}
	fn into_binding(&self, index: u32) -> VkDescriptorSetLayoutBinding
	{
		VkDescriptorSetLayoutBinding
		{
			binding: index, descriptorType: self.native_type(), descriptorCount: self.count(),
			stageFlags: self.stage_mask(), pImmutableSamplers: std::ptr::null()
		}
	}
	fn into_pool_size(&self) -> VkDescriptorPoolSize
	{
		VkDescriptorPoolSize(self.native_type(), self.count())
	}
}

pub struct DescriptorSetLayout
{
	internal: vk::DescriptorSetLayout,
	structure: Vec<Descriptor>
}
pub trait DescriptorSetLayoutInternals
{
	fn new(dsl: vk::DescriptorSetLayout, structure: &[Descriptor]) -> Self;
	fn descriptors(&self) -> &[Descriptor];
}
impl DescriptorSetLayoutInternals for DescriptorSetLayout
{
	fn new(dsl: vk::DescriptorSetLayout, structure: &[Descriptor]) -> Self
	{
		DescriptorSetLayout { internal: dsl, structure: Vec::from(structure) }
	}
	fn descriptors(&self) -> &[Descriptor] { &self.structure }
}
impl InternalExports<vk::DescriptorSetLayout> for DescriptorSetLayout
{
	fn get_internal(&self) -> &vk::DescriptorSetLayout { &self.internal }
}

pub struct DescriptorSets
{
	pool: vk::DescriptorPool, sets: Vec<VkDescriptorSet>
}
pub trait DescriptorSetsInternals
{
	fn new(pool: vk::DescriptorPool, sets: Vec<VkDescriptorSet>) -> Self;
}
impl DescriptorSetsInternals for DescriptorSets
{
	fn new(pool: vk::DescriptorPool, sets: Vec<VkDescriptorSet>) -> Self
	{
		DescriptorSets { pool: pool, sets: sets }
	}
}
