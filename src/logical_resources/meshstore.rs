use render_vk::wrap as vk;
use vkffi::*;
use std;
use vertex_formats::*;
use traits::*;

pub struct Meshstore
{
	pub unit_cube_vertices_offset: VkDeviceSize,
	pub unit_plane_vertices_offset: VkDeviceSize
}
impl Meshstore
{
	pub fn new(offset: VkDeviceSize) -> Self
	{
		Meshstore
		{
			unit_cube_vertices_offset: offset,
			unit_plane_vertices_offset: offset + std::mem::size_of::<[Position; 4]>() as VkDeviceSize
		}
	}
}
impl DeviceStore for Meshstore
{
	fn required_sizes() -> Vec<VkDeviceSize>
	{
		vec![std::mem::size_of::<[Position; 4]>() as VkDeviceSize * 2]
	}
	fn initial_stage_data(&self, mapped_range: &vk::MemoryMappedRange)
	{
		let ucv_range = mapped_range.map_mut::<[Position; 4]>(self.unit_cube_vertices_offset);
		let upv_range = mapped_range.map_mut::<[Position; 4]>(self.unit_plane_vertices_offset);

		ucv_range[0] = Position(-1.0f32, -1.0f32, -1.0f32, 1.0f32);
		ucv_range[1] = Position( 1.0f32, -1.0f32, -1.0f32, 1.0f32);
		ucv_range[2] = Position( 1.0f32,  1.0f32, -1.0f32, 1.0f32);
		ucv_range[3] = Position(-1.0f32,  1.0f32, -1.0f32, 1.0f32);
		upv_range[0] = Position(-1.0f32, -1.0f32, 0.0f32, 1.0f32);
		upv_range[1] = Position( 1.0f32, -1.0f32, 0.0f32, 1.0f32);
		upv_range[2] = Position( 1.0f32,  1.0f32, 0.0f32, 1.0f32);
		upv_range[3] = Position(-1.0f32,  1.0f32, 0.0f32, 1.0f32);
	}
}
