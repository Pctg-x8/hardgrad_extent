use x11;
use std;
use std::ffi::CString;

pub struct Display { pub internal: *mut x11::xlib::Display }
impl Display
{
	pub fn open(name: Option<&str>) -> Option<Self>
	{
		let cstr = name.map(|s| CString::new(s).unwrap());
		let cstr_ptr = cstr.map(|x| x.as_ptr()).unwrap_or(std::ptr::null());
		Some(unsafe { x11::xlib::XOpenDisplay(cstr_ptr) }).map(|x| Display { internal: x })
	}
	pub fn get_default_root_window<'a>(&'a self) -> WindowRef<'a>
	{
		WindowRef
		{
			display_ref: &self,
			internal_ref: unsafe { x11::xlib::XDefaultRootWindow(self.internal) }
		}
	}
	pub fn create_window<'a>(&'a self, root: &WindowRef<'a>, wa: &x11::xlib::XWindowAttributes) -> Option<WindowWithColormap<'a>>
	{
		match Some(unsafe { x11::xlib::XCreateColormap(self.internal, root.internal_ref, wa.visual, x11::xlib::AllocNone) })
		{
			Some(cmap) =>
			{
				let mut swa = x11::xlib::XSetWindowAttributes
				{
					colormap: cmap,
					event_mask: x11::xlib::KeyPressMask,
					background_pixmap: 0, do_not_propagate_mask: wa.do_not_propagate_mask,
					override_redirect: wa.override_redirect, save_under: wa.save_under,
					backing_store: wa.backing_store, backing_pixel: wa.backing_pixel,
					win_gravity: wa.win_gravity, bit_gravity: wa.bit_gravity,
					backing_planes: wa.backing_planes, cursor: 0, border_pixmap: 0, border_pixel: 0,
					background_pixel: 0
				};
				Some(unsafe { x11::xlib::XCreateWindow(self.internal, root.internal_ref, 0, 0, 640, 480, 0, wa.depth,
					x11::xlib::InputOutput as u32, wa.visual, x11::xlib::CWColormap | x11::xlib::CWEventMask, &mut swa) })
				.map(|wnd| WindowWithColormap { display_ref: &self, cmap: cmap, internal: wnd })
			},
			_ => None
		}
	}
	pub fn intern_atom(&self, value: &str, only_if_exists: bool) -> x11::xlib::Atom
	{
		unsafe { x11::xlib::XInternAtom(self.internal, CString::new(value).unwrap().as_ptr(), if only_if_exists { 1 } else { 0 }) }
	}
}
impl std::ops::Drop for Display
{
	fn drop(&mut self) { unsafe { x11::xlib::XCloseDisplay(self.internal); } }
}

pub trait XlibWindow
{
	fn get_window_attributes(&self) -> x11::xlib::XWindowAttributes;
	fn map(&self);
	fn set_title_raw(&self, name: &str);
	fn set_wm_protocols(&self, msg_atom: &mut [x11::xlib::Atom]);
}
pub struct WindowRef<'a> { display_ref: &'a Display, internal_ref: x11::xlib::Window }
pub struct WindowWithColormap<'a>
{
	pub display_ref: &'a Display,
	#[allow(dead_code)] cmap: x11::xlib::Colormap,
	pub internal: x11::xlib::Window
}
impl <'a> XlibWindow for WindowRef<'a>
{
	fn get_window_attributes(&self) -> x11::xlib::XWindowAttributes
	{
		let mut p: x11::xlib::XWindowAttributes = unsafe { std::mem::uninitialized() };
		unsafe { x11::xlib::XGetWindowAttributes(self.display_ref.internal, self.internal_ref, &mut p) };
		p
	}
	fn map(&self) { unreachable!(); }
	fn set_title_raw(&self, _: &str) { unreachable!(); }
	fn set_wm_protocols(&self, _: &mut [x11::xlib::Atom]) { unreachable!(); }
}
impl <'a> XlibWindow for WindowWithColormap<'a>
{
	fn get_window_attributes(&self) -> x11::xlib::XWindowAttributes
	{
		let mut p: x11::xlib::XWindowAttributes = unsafe { std::mem::uninitialized() };
		unsafe { x11::xlib::XGetWindowAttributes(self.display_ref.internal, self.internal, &mut p) };
		p
	}
	fn map(&self) { unsafe { x11::xlib::XMapWindow(self.display_ref.internal, self.internal) }; }
	fn set_title_raw(&self, name: &str)
	{
		unsafe { x11::xlib::XStoreName(self.display_ref.internal, self.internal, name.as_ptr() as *const i8) };
	}
	fn set_wm_protocols(&self, msg_atoms: &mut [x11::xlib::Atom])
	{
		unsafe { x11::xlib::XSetWMProtocols(self.display_ref.internal, self.internal, msg_atoms.as_mut_ptr(), msg_atoms.len() as i32) };
	}
}
