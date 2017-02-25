use nalgebra::*;

use interlude::*;
use structures;
use constants::SCREEN_SIZE;

pub fn setup_parameters(memory_ref: &mut structures::UniformMemory, screen_size: &Size2)
{
	let &Size2(width, height) = screen_size;
	let (ortho_matrix, pixel_matrix, persp_matrix) = (
		OrthographicMatrix3::new(-SCREEN_SIZE, SCREEN_SIZE, 0.0f32, SCREEN_SIZE * 2.0, -220.0f32, 100.0f32),
		OrthographicMatrix3::new(0.0f32, width as f32, 0.0f32, height as f32, -2.0f32, 1.0f32),
		PerspectiveMatrix3::new(1.0f32, 70.0f32.to_radians(), 0.0f32, 100.0f32)
	);

	memory_ref.projection_matrixes.ortho = *ortho_matrix.as_matrix().transpose().as_ref();
	memory_ref.projection_matrixes.pixel = *pixel_matrix.as_matrix().transpose().as_ref();
	memory_ref.projection_matrixes.persp = *persp_matrix.as_matrix().transpose().as_ref();
}
