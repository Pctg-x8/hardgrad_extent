// Xorg Xlib Safety Bindings

use x11;
use ::std;
use std::os::raw::c_char;

pub struct Display
{
	ptr: *mut x11::xlib::Display
}
impl Display
{
	pub fn open(name: *mut c_char) -> Display
	{
		Display
		{
			ptr: Some(unsafe { x11::xlib::XOpenDisplay(name) }).expect("Unable to open display")
		}
	}
}
impl std::ops::Drop for Display
{
	fn drop(&mut self) { unsafe { x11::xlib::XCloseDisplay(self.ptr) }; }
}
