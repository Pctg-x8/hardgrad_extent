use vkffi::*;
use render_vk::wrap as vk;
use std;
use traits::*;
use device_resources;

const MAX_BK_COUNT: VkDeviceSize = 64;

pub struct BackgroundDatastore
{
    #[allow(dead_code)] descriptor_set_index: usize,
    pub uniform_offset: VkDeviceSize, pub index_multipliers_offset: VkDeviceSize,
    descriptor_buffer_info: VkDescriptorBufferInfo
}
impl BackgroundDatastore
{
    pub fn new<'d>(buffer: &vk::Buffer<'d>, offset: VkDeviceSize, descriptor_set_index: usize) -> BackgroundDatastore
    {
        BackgroundDatastore
        {
            descriptor_set_index: descriptor_set_index,
            uniform_offset: offset,
            index_multipliers_offset: offset + std::mem::size_of::<[f32; 4]>() as VkDeviceSize * MAX_BK_COUNT,
            descriptor_buffer_info: VkDescriptorBufferInfo(**buffer, offset, Self::required_sizes()[0])
        }
    }
}
impl DeviceStore for BackgroundDatastore
{
    fn required_sizes() -> Vec<VkDeviceSize>
    {
        vec![std::mem::size_of::<[f32; 4]>() as VkDeviceSize * MAX_BK_COUNT, std::mem::size_of::<u32>() as VkDeviceSize * MAX_BK_COUNT as VkDeviceSize]
    }
    fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
    {
        let index_multipliers_range = mapped_range.range_mut::<u32>(self.index_multipliers_offset, MAX_BK_COUNT as usize);

        index_multipliers_range.copy_from_slice(&[0u32; MAX_BK_COUNT as usize]);
    }
}
impl HasDescriptor for BackgroundDatastore
{
    fn write_descriptor_info<'d>(&self, sets: &device_resources::DescriptorSets<'d>) -> Vec<VkWriteDescriptorSet>
    {
        vec![VkWriteDescriptorSet
        {
            sType: VkStructureType::WriteDescriptorSet, pNext: std::ptr::null(),
            dstSet: sets.sets[self.descriptor_set_index], dstBinding: 0, dstArrayElement: 0,
            descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
            pBufferInfo: &self.descriptor_buffer_info, pImageInfo: std::ptr::null(), pTexelBufferView: std::ptr::null()
        }]
    }
}
