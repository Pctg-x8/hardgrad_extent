use nalgebra::*;

use interlude::ffi::*;
use structures;

pub fn setup_parameters(memory_ref: &mut structures::UniformMemory, screen_size: VkExtent2D)
{
	let VkExtent2D(width, height) = screen_size;
	let (aspect, scaling) = (width as f32 / height as f32, 35.0f32);
	let (ortho_matrix, pixel_matrix, persp_matrix) = (
		OrthographicMatrix3::new(-scaling, scaling, 0.0f32, scaling * aspect, -220.0f32, 100.0f32),
		OrthographicMatrix3::new(0.0f32, width as f32, 0.0f32, height as f32, -2.0f32, 1.0f32),
		PerspectiveMatrix3::new(aspect, 70.0f32.to_radians(), 0.0f32, 100.0f32)
	);

	memory_ref.projection_matrixes.ortho = *ortho_matrix.as_matrix().transpose().as_ref();
	memory_ref.projection_matrixes.pixel = *pixel_matrix.as_matrix().transpose().as_ref();
	memory_ref.projection_matrixes.persp = *persp_matrix.as_matrix().transpose().as_ref();
}
