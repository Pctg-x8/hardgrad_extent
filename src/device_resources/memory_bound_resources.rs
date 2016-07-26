// Memory bound Resources

use vkffi::*;
use render_vk::wrap as vk;
use render_vk::memory::*;
use traits::*;

use logical_resources::*;

pub struct MemoryPreallocator
{
	pub meshstore_base: VkDeviceSize,
	pub projection_matrixes_base: VkDeviceSize,
	pub enemy_datastore_base: VkDeviceSize,
	pub total_size: VkDeviceSize
}
impl MemoryPreallocator
{
	pub fn new(adapter: &vk::PhysicalDevice) -> Self
	{
		fn align(x: VkDeviceSize, a: VkDeviceSize) -> VkDeviceSize { (x as f64 / a as f64).ceil() as VkDeviceSize * a }
		let adapter_limits = adapter.get_properties().limits;
		let uniform_buffer_alignment = adapter_limits.minUniformBufferOffsetAlignment;
		let align_for_uniform_buffer = |x: VkDeviceSize| align(x, uniform_buffer_alignment);

		// Request all memory block sizes
		let meshstore_sizes = Meshstore::required_sizes();
		let meshstore_aligned_size = align_for_uniform_buffer(meshstore_sizes.iter().fold(0, |a, x| a + x));
		let projection_matrixes_sizes = ProjectionMatrixes::required_sizes();
		let projection_matrixes_aligned_size = align_for_uniform_buffer(projection_matrixes_sizes.iter().fold(0, |a, x| a + x));
		let enemy_datastore_sizes = EnemyDatastore::required_sizes();
		let enemy_datastore_size: VkDeviceSize = enemy_datastore_sizes.iter().fold(0, |a, x| a + x);
		// Preallocations
		let meshstore_base_offs = 0;
		let projection_matrixes_base_offs = meshstore_base_offs + meshstore_aligned_size;
		let enemy_datastore_base_offs = projection_matrixes_base_offs + projection_matrixes_aligned_size;

		println!("-- Preallocator Report:");
		println!("---- Alignment for UniformBuffer: {}", uniform_buffer_alignment);
		println!("---- Meshstore Size: {}", meshstore_sizes.iter().fold(0, |x, y| x + y));
		println!("---- ProjectionMatrixes Size: {}", projection_matrixes_sizes.iter().fold(0, |x, y| x + y));
		println!("---- EnemyDatastore Size: {}", enemy_datastore_sizes.iter().fold(0, |x, y| x + y));
		println!("---- Meshstore from {}", meshstore_base_offs);
		println!("---- ProjectionMatrixes from {}", projection_matrixes_base_offs);
		println!("---- EnemyDatastore from {}", enemy_datastore_base_offs);

		MemoryPreallocator
		{
			total_size: enemy_datastore_base_offs + enemy_datastore_size,
			meshstore_base: meshstore_base_offs,
			projection_matrixes_base: projection_matrixes_base_offs,
			enemy_datastore_base: enemy_datastore_base_offs
		}
	}
}

pub struct MemoryBoundResources<'d>
{
	pub buffer: DeviceBuffer<'d>, pub stage_buffer: StagingBuffer<'d>
}
impl <'d> MemoryBoundResources<'d>
{
	pub fn new(device: &'d vk::Device, preallocator: &MemoryPreallocator) -> Self
	{
		MemoryBoundResources
		{
			buffer: DeviceBuffer::new(device, preallocator.total_size, VK_BUFFER_USAGE_VERTEX_BUFFER_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT | VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT),
			stage_buffer: StagingBuffer::new(device, preallocator.total_size)
		}
	}
}
