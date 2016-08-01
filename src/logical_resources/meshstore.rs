use render_vk::wrap as vk;
use vkffi::*;
use std;
use vertex_formats::*;
use traits::*;
use structures;

pub struct Meshstore
{
	pub wire_render_offset: VkDeviceSize
}
impl Meshstore
{
	pub fn new(offset: VkDeviceSize) -> Self
	{
		Meshstore
		{
			wire_render_offset: offset
		}
	}
}
impl DeviceStore for Meshstore
{
	fn required_sizes() -> Vec<VkDeviceSize>
	{
		vec![std::mem::size_of::<structures::VertexMemoryForWireRender>() as VkDeviceSize]
	}
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
	{
		let vmem_wire = mapped_range.map_mut::<structures::VertexMemoryForWireRender>(self.wire_render_offset);

		vmem_wire.unit_plane_source_vts = [
			Position(-1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, 0.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, 0.0f32, 1.0f32)
		];
	}
}
