use render_vk::wrap as vk;
use vkffi::*;
use std;
use vertex_formats::*;
use traits::*;
use structures;

pub struct Meshstore
{
	pub wire_render_offset: VkDeviceSize,
	pub index_offset: VkDeviceSize
}
impl Meshstore
{
	pub fn new(offset: VkDeviceSize) -> Self
	{
		Meshstore
		{
			wire_render_offset: offset,
			index_offset: offset + Self::required_sizes()[0]
		}
	}
}
impl DeviceStore for Meshstore
{
	fn required_sizes() -> Vec<VkDeviceSize>
	{
		vec![
			std::mem::size_of::<structures::VertexMemoryForWireRender>() as VkDeviceSize,
			std::mem::size_of::<structures::IndexMemory>() as VkDeviceSize
		]
	}
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
	{
		let vmem_wire = mapped_range.map_mut::<structures::VertexMemoryForWireRender>(self.wire_render_offset);
		let imem = mapped_range.map_mut::<structures::IndexMemory>(self.index_offset);

		vmem_wire.unit_plane_source_vts = [
			Position(-1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, 0.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, 0.0f32, 1.0f32)
		];
		vmem_wire.player_cube_vts = [
			Position(-1.0f32, -1.0f32, -1.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, -1.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, -1.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, -1.0f32, 1.0f32),
			Position(-1.0f32, -1.0f32,  1.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32,  1.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32,  1.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32,  1.0f32, 1.0f32)
		];
		imem.player_cube_ids = [
			0, 1, 1, 2, 2, 3, 3, 0,
			4, 5, 5, 6, 6, 7, 7, 4,
			0, 4, 1, 5, 2, 6, 3, 7
		];
	}
}
