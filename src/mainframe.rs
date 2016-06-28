// MainFrame

use render::backend::RenderBackend;
#[cfg(feature = "use_x11")] use xorg as platform;
#[cfg(windows)] use win as platform;

pub struct MainFrame
{
	internal: platform::Frame
}
impl MainFrame
{
	pub fn launch_static() -> i32
	{
		println!("=== HardGrad -> Extent ===");

		let mf = MainFrame::create();
		let backend = RenderBackend::init();

		0
	}
	fn create() -> MainFrame
	{
		let internal = platform::create_frame();

		MainFrame { internal: internal }
	}
}
