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
	pub fn new(adapter: &vk::PhysicalDevice, device: &'d vk::Device,
		desc_pool: &'d vk::DescriptorPool<'d>, desc_layout: &'d vk::DescriptorSetLayout<'d>,
		size: VkExtent2D) -> Self
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

			let VkExtent2D(width, height) = size;
			let (aspect, scaling) = (height as f32 / width as f32, 28.0f32);
			let ortho_matrix = OrthographicMatrix3::new(-scaling * aspect, scaling * aspect, 0.0f32, scaling, -100.0f32, 100.0f32);
			let persp_matrix = PerspectiveMatrix3::new(aspect, 70.0f32, -100.0f32, 100.0f32);

			{
				let r = mapped_range.range_mut::<f32>(0, 16);
				let matr = ortho_matrix.as_matrix();
				for x in 0 .. 4 { for y in 0 .. 4 { r[x + y * 4] = matr.as_ref()[x][y]; } }
			}
			{
				let r = mapped_range.range_mut::<f32>(persp_offs, 16);
				let matr = persp_matrix.as_matrix();
				for x in 0 .. 4 { for y in 0 .. 4 { r[x + y * 4] = matr.as_ref()[y][x]; } }
			}
		}

		// Descriptor Set //
		let sets = desc_pool.allocate_sets(&[desc_layout.get()]).unwrap();
		let ortho_buffer_info = VkDescriptorBufferInfo(buffer.get(), ortho_offs, std::mem::size_of::<[[f32; 4]; 4]>() as VkDeviceSize * 2);
		let write_sets = [
			VkWriteDescriptorSet
			{
				sType: VkStructureType::WriteDescriptorSet, pNext: std::ptr::null(),
				dstSet: sets[0], dstBinding: 0, dstArrayElement: 0,
				descriptorType: VkDescriptorType::UniformBuffer, descriptorCount: 1,
				pBufferInfo: &ortho_buffer_info, pImageInfo: std::ptr::null(), pTexelBufferView: std::ptr::null_mut()
			}
		];
		device.update_descriptor_sets(&write_sets, &[]);

		ProjectionMatrixes
		{
			buffer: buffer, memory: memory, uniform_desc_set: sets,
			ortho_offs: ortho_offs, persp_offs: persp_offs
		}
	}
}
