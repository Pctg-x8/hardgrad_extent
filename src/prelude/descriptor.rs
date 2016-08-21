// Prelude: Descriptor and its layout in shader

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
impl std::ops::Deref for DescriptorSets
{
	type Target = DescriptorSetArrayView;
	fn deref(&self) -> &Self::Target { &self.sets }
}
pub type DescriptorSetArrayView = [VkDescriptorSet];

pub struct BufferInfo<'a>(pub &'a BufferResource, pub std::ops::Range<usize>);
impl <'a> std::convert::Into<VkDescriptorBufferInfo> for &'a BufferInfo<'a>
{
	fn into(self) -> VkDescriptorBufferInfo
	{
		let &BufferInfo(res, ref range) = self;
		VkDescriptorBufferInfo(res.get_resource(), range.start as VkDeviceSize, (range.end - range.start) as VkDeviceSize)
	}
}

pub enum DescriptorSetWriteInfo<'a>
{
	UniformBuffer(VkDescriptorSet, u32, Vec<BufferInfo<'a>>)
}
pub struct IntoWriteDescriptorSetNativeStruct
{
	target: VkDescriptorSet, binding: u32,
	dtype: VkDescriptorType, buffers: Vec<VkDescriptorBufferInfo>
}
impl <'a> std::convert::Into<IntoWriteDescriptorSetNativeStruct> for &'a DescriptorSetWriteInfo<'a>
{
	fn into(self) -> IntoWriteDescriptorSetNativeStruct
	{
		match self
		{
			&DescriptorSetWriteInfo::UniformBuffer(target, binding, ref bufs) => IntoWriteDescriptorSetNativeStruct
			{
				target: target, binding: binding, buffers: bufs.iter().map(|x| x.into()).collect(),
				dtype: VkDescriptorType::UniformBuffer
			}
		}
	}
}
impl <'a> std::convert::Into<VkWriteDescriptorSet> for &'a IntoWriteDescriptorSetNativeStruct
{
	fn into(self) -> VkWriteDescriptorSet
	{
		VkWriteDescriptorSet
		{
			sType: VkStructureType::WriteDescriptorSet, pNext: std::ptr::null(),
			dstSet: self.target, dstBinding: self.binding, dstArrayElement: 0,
			descriptorType: self.dtype, descriptorCount: self.buffers.len() as u32,
			pBufferInfo: self.buffers.as_ptr(), pImageInfo: std::ptr::null(), pTexelBufferView: std::ptr::null()
		}
	}
}
