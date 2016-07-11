use render_vk::wrap as vk;
use render_vk::traits::*;
use vkffi::*;
use std;
use vertex_formats::*;

fn block_count(x: VkDeviceSize, a: VkDeviceSize) -> VkDeviceSize { ((x as f32) / (a as f32)).ceil() as VkDeviceSize }
fn alignment(x: VkDeviceSize, a: VkDeviceSize) -> VkDeviceSize { block_count(x, a) * a }

pub struct MeshStore<'d>
{
	pub buffer: vk::Buffer<'d>, #[allow(dead_code)] memory: vk::DeviceMemory<'d>,
	pub unit_cube_vertices_offset: VkDeviceSize, pub unit_cube_indices_offset: VkDeviceSize
}
impl <'d> MeshStore<'d>
{
	pub fn new(pdev: &vk::PhysicalDevice, device: &'d vk::Device) -> Self
	{
		let ucv_size = alignment((std::mem::size_of::<Position>() * 8) as VkDeviceSize, 1);
		let uci_size = alignment((std::mem::size_of::<u16>() * 24) as VkDeviceSize, 1);
		let buf = device.create_buffer(VK_BUFFER_USAGE_VERTEX_BUFFER_BIT, (ucv_size + uci_size) as usize).unwrap();
		let size_req = buf.get_memory_requirements();
		let memindex = pdev.get_memory_type_index(VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT).expect("Unable to find mappable memory");
		let alloc_info = VkMemoryAllocateInfo
		{
			sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
			allocationSize: size_req.size, memoryTypeIndex: memindex as u32
		};
		let memory = device.allocate_memory(&alloc_info).unwrap();
		memory.bind_buffer(&buf, 0).unwrap();

		// initial storing //
		let ucv_offs = 0 as VkDeviceSize; let uci_offs = ucv_size as VkDeviceSize;
		{
			let mapped_range = memory.map(0 .. (ucv_size + uci_size)).unwrap();
			let ucv_range = mapped_range.range_mut::<Position>(0, 8);
			let uci_range = mapped_range.range_mut::<u16>(uci_offs, 24);

			ucv_range[0] = Position(-1.0f32, -1.0f32, -1.0f32, 1.0f32);
			ucv_range[1] = Position( 1.0f32, -1.0f32, -1.0f32, 1.0f32);
			ucv_range[2] = Position( 1.0f32,  1.0f32, -1.0f32, 1.0f32);
			ucv_range[3] = Position(-1.0f32,  1.0f32, -1.0f32, 1.0f32);
			ucv_range[4] = Position(-1.0f32, -1.0f32,  1.0f32, 1.0f32);
			ucv_range[5] = Position( 1.0f32, -1.0f32,  1.0f32, 1.0f32);
			ucv_range[6] = Position( 1.0f32,  1.0f32,  1.0f32, 1.0f32);
			ucv_range[7] = Position(-1.0f32,  1.0f32,  1.0f32, 1.0f32);
			uci_range[ 0 * 2 + 0] = 0; uci_range[ 0 * 2 + 1] = 1;
			uci_range[ 1 * 2 + 0] = 1; uci_range[ 1 * 2 + 1] = 2;
			uci_range[ 2 * 2 + 0] = 2; uci_range[ 2 * 2 + 1] = 3;
			uci_range[ 3 * 2 + 0] = 3; uci_range[ 3 * 2 + 1] = 0;
			uci_range[ 4 * 2 + 0] = 4; uci_range[ 4 * 2 + 1] = 5;
			uci_range[ 5 * 2 + 0] = 5; uci_range[ 5 * 2 + 1] = 6;
			uci_range[ 6 * 2 + 0] = 6; uci_range[ 6 * 2 + 1] = 7;
			uci_range[ 7 * 2 + 0] = 7; uci_range[ 7 * 2 + 1] = 4;
			uci_range[ 8 * 2 + 0] = 0; uci_range[ 8 * 2 + 1] = 4;
			uci_range[ 9 * 2 + 0] = 1; uci_range[ 9 * 2 + 1] = 5;
			uci_range[10 * 2 + 0] = 2; uci_range[10 * 2 + 1] = 6;
			uci_range[11 * 2 + 0] = 3; uci_range[11 * 2 + 1] = 7;
		}

		MeshStore
		{
			buffer: buf, memory: memory,
			unit_cube_vertices_offset: ucv_offs, unit_cube_indices_offset: uci_offs
		}
	}
}
