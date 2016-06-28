// Render Engine

use render::backend::RenderBackend;

pub struct RenderEngine
{
	backend: RenderBackend
}
impl RenderEngine
{
	fn init() -> Self
	{
		RenderEngine
		{
			backend: RenderBackend::init()
		}
	}
}
