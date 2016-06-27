// Xorg Safety Bindings

pub mod xlib;
use ::std;

// extern in source
use self::xlib::*;

// externed common structures

pub struct Frame
{
	display_session: Display
}
pub fn create_frame() -> Frame
{
	println!("-- Launching System on X11 Platform");

	let display = Display::open(std::ptr::null_mut());

	Frame
	{
		display_session: display
	}
}
