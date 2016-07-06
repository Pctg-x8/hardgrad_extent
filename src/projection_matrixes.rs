use nalgebra::*;

use traits::*;
use render_vk::wrap as vk;
use render_vk::wrap::MemoryAllocationRequired;
use vkffi::*;
use std;

pub struct ProjectionMatrixes<'d>
{
    pub buffer: vk::Buffer<'d>, #[allow(dead_code)] memory: vk::DeviceMemory<'d>,
    pub uniform_desc_set: vk::DescriptorSets<'d>,
    pub ortho_offs: VkDeviceSize, pub persp_offs: VkDeviceSize
}
impl <'d> ProjectionMatrixes<'d>
{
    pub fn new(adapter: &vk::PhysicalDevice, device: &'d vk::Device, desc_pool: &'d vk::DescriptorPool<'d>, desc_layout: &'d vk::DescriptorSetLayout<'d>) -> Self
    {
        let matrix_buffer_size = std::mem::size_of::<[[f32; 4]; 4]>();

        let buffer_info = VkBufferCreateInfo
        {
            sType: VkStructureType::BufferCreateInfo, pNext: std::ptr::null(),
            usage: VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT, size: matrix_buffer_size as VkDeviceSize * 2,
            sharingMode: VkSharingMode::Exclusive,
            queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null(), flags: 0
        };
        let buffer = device.create_buffer(&buffer_info).unwrap();
        let size_req = buffer.get_memory_requirements();
        let memindex = adapter.get_memory_type_index(VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT).expect("Unable to find host mappable heap");
        let alloc_info = VkMemoryAllocateInfo
        {
            sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
            allocationSize: size_req.size, memoryTypeIndex: memindex as u32
        };
        let memory = device.allocate_memory(&alloc_info).unwrap();
        memory.bind_buffer(&buffer, 0).unwrap();

        // initial storing //
        let ortho_offs = 0 as VkDeviceSize; let persp_offs = matrix_buffer_size as VkDeviceSize;
        {
            let mapped_range = memory.map(0 .. buffer_info.size).unwrap();

            let ortho_matrix = OrthographicMatrix3::new(0.0f32, 50.0f32, 0.0f32, 50.0f32, -100.0f32, 100.0f32);
            let persp_matrix = PerspectiveMatrix3::new(3.0f32 / 4.0f32, 60.0f32, -100.0f32, 100.0f32);

            mapped_range.range_mut(0, 4).clone_from_slice(ortho_matrix.as_matrix().as_ref());
            mapped_range.range_mut(persp_offs, 4).clone_from_slice(persp_matrix.as_matrix().as_ref());
        }

        // Descriptor Set //
        let sets = desc_pool.allocate_sets(&[desc_layout, desc_layout]).unwrap();
        let write_sets = [
            VkWriteDescriptorSet
            {
                sType: VkStructureType::WriteDescriptorSet, pNext: std::ptr::null(),
                dstSet: sets[0]
            }
        ];

        ProjectionMatrixes
        {
            buffer: buffer, memory: memory, uniform_desc_set: sets,
            ortho_offs: ortho_offs, persp_offs: persp_offs
        }
    }
}
