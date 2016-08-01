use nalgebra::*;

use traits::*;
use vkffi::*;
use structures;

pub struct ProjectionMatrixes
{
	pub screen_size: VkExtent2D
}
impl ProjectionMatrixes
{
	pub fn new(screen_size: VkExtent2D) -> Self
	{
		ProjectionMatrixes { screen_size: screen_size }
	}
}
impl UniformStore for ProjectionMatrixes
{
	fn initial_stage_data(&self, memory_ref: &mut structures::UniformMemory)
	{
		let VkExtent2D(width, height) = self.screen_size;
		let (aspect, scaling) = (width as f32 / height as f32, 35.0f32);
		let ortho_matrix = OrthographicMatrix3::new(-scaling, scaling, 0.0f32, scaling * aspect, -200.0f32, 100.0f32);
		let pixel_matrix = OrthographicMatrix3::new(0.0f32, width as f32, 0.0f32, height as f32, -2.0f32, 1.0f32);
		let persp_matrix = PerspectiveMatrix3::new(aspect, 70.0f32.to_radians(), 0.0f32, 100.0f32);

		memory_ref.projection_matrixes.ortho = *ortho_matrix.as_matrix().transpose().as_ref();
		memory_ref.projection_matrixes.pixel = *pixel_matrix.as_matrix().transpose().as_ref();
		memory_ref.projection_matrixes.persp = *persp_matrix.as_matrix().transpose().as_ref();
	}
}
