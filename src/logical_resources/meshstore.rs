use render_vk::wrap as vk;
use vkffi::*;
use std;
use vertex_formats::*;
use traits::*;

pub struct Meshstore
{
	pub unit_cube_vertices_offset: VkDeviceSize, pub unit_cube_indices_offset: VkDeviceSize
}
impl Meshstore
{
	pub fn new(offset: VkDeviceSize) -> Self
	{
		Meshstore
		{
			unit_cube_vertices_offset: offset,
			unit_cube_indices_offset: offset + (std::mem::size_of::<Position>() * 8) as VkDeviceSize
		}
	}
}
impl DeviceStore for Meshstore
{
	fn device_size() -> VkDeviceSize { (std::mem::size_of::<Position>() * 8 + std::mem::size_of::<u16>() * 24) as VkDeviceSize }
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
	{
		let ucv_range = mapped_range.range_mut::<Position>(self.unit_cube_vertices_offset, 8);
		let uci_range = mapped_range.range_mut::<u16>(self.unit_cube_indices_offset, 24);

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
}
