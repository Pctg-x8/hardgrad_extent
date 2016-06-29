// MainFrame

use render::backend::RenderBackend;
use traits::*;
#[cfg(feature = "use_x11")] use xorg as platform;
#[cfg(feature = "use_win32")] use win as platform;

pub enum MainFrame {}
impl MainFrame
{
	pub fn launch_static() -> i32
	{
		println!("=== HardGrad -> Extent ===");

		let internal = platform::create_frame();
		let backend = RenderBackend::init();
		let sw = internal.create_swapchain(&backend);

		0
	}
}
