// XCB Objective Wrapper

use xcb; use std; use libc;
use traits::*;
use vkffi::*;
use render_vk::wrap as vk;

pub struct XServerConnection
{
	con: xcb::Connection, root_screen: xcb::ffi::xcb_window_t, root_visual: xcb::ffi::xcb_visualid_t, root_depth: u8,
	protocols_atom: xcb::ffi::xproto::xcb_atom_t, delete_window_atom: xcb::ffi::xproto::xcb_atom_t
}
impl XServerConnection
{
	pub fn connect() -> Self
	{
		let (con, screen_default_num) = xcb::Connection::connect(None).unwrap();
		let default_screen =
		{
			fn recursive(mut iter: xcb::ffi::xproto::xcb_screen_iterator_t, remain: i32) -> Option<*mut xcb::ffi::xproto::xcb_screen_t>
			{
				if remain <= 0 { Some(iter.data) }
				else if iter.rem == 0 { None }
				else
				{
					unsafe { xcb::ffi::xproto::xcb_screen_next(&mut iter) };
					recursive(iter, remain - 1)
				}
			}
			let iter = unsafe { xcb::ffi::xproto::xcb_setup_roots_iterator(con.get_setup().ptr) };
			recursive(iter, screen_default_num).expect("Unable to find default screen")
		};
		let root_screen = unsafe { (*default_screen).root };
		let visual_id = unsafe { (*default_screen).root_visual };
		let depth = unsafe { (*default_screen).root_depth };

		let protocols_str = "WM_PROTOCOLS";
		let delete_window_str = "WM_DELETE_WINDOW";
		let ia_protocols_cookie = unsafe { xcb::ffi::xproto::xcb_intern_atom(con.get_raw_conn(), false as u8, protocols_str.len() as u16, protocols_str.as_ptr() as *const i8) };
		let ia_delete_window_cookie = unsafe { xcb::ffi::xproto::xcb_intern_atom(con.get_raw_conn(), false as u8, delete_window_str.len() as u16, delete_window_str.as_ptr() as *const i8) };

		XServerConnection
		{
			protocols_atom: unsafe { (*xcb::ffi::xproto::xcb_intern_atom_reply(con.get_raw_conn(), ia_protocols_cookie, std::ptr::null_mut())).atom },
			delete_window_atom: unsafe { (*xcb::ffi::xproto::xcb_intern_atom_reply(con.get_raw_conn(), ia_delete_window_cookie, std::ptr::null_mut())).atom },
			con: con, root_screen: root_screen, root_visual: visual_id, root_depth: depth
		}
	}
	pub fn new_window(&self, size: VkExtent2D, title: &str) -> XWindow
	{
		let window_id = self.con.generate_id();
		let VkExtent2D(width, height) = size;
		println!("creating window with resolution {}x{}", width, height);
		unsafe { xcb::ffi::xproto::xcb_create_window(self.con.get_raw_conn(), self.root_depth, window_id, self.root_screen,
			0, 0, width as u16, height as u16, 0, xcb::ffi::xproto::XCB_WINDOW_CLASS_INPUT_OUTPUT as u16, self.root_visual, 0, std::ptr::null()) };
		unsafe { xcb::ffi::xproto::xcb_change_property(self.con.get_raw_conn(), xcb::ffi::xproto::XCB_PROP_MODE_REPLACE as u8, window_id,
			xcb::xproto::ATOM_WM_NAME, xcb::xproto::ATOM_STRING, 8, title.len() as u32, title.as_ptr() as *const libc::c_void) };
		unsafe { xcb::ffi::xproto::xcb_change_property(self.con.get_raw_conn(), xcb::ffi::xproto::XCB_PROP_MODE_REPLACE as u8, window_id,
			self.protocols_atom, 4, 32, 1, std::mem::transmute(&self.delete_window_atom)) };
		XWindow { con_ref: self, internal: window_id }
	}
	pub fn is_vk_presentation_support(&self, adapter: &vk::PhysicalDevice, queue_index: u32) -> bool
	{
		unsafe { vkGetPhysicalDeviceXcbPresentationSupportKHR(adapter.get(), queue_index, self.con.get_raw_conn(), self.root_visual) == 1 }
	}
	pub fn flush(&self) { self.con.flush(); }
	pub fn poll_event(&self) -> Option<xcb::GenericEvent> { self.con.poll_for_event() }
	pub fn get_raw(&self) -> *mut xcb::ffi::xcb_connection_t { self.con.get_raw_conn() }
	pub fn is_delete_window_message(&self, event_ptr: *const xcb::ffi::xproto::xcb_client_message_event_t) -> bool
	{
		unsafe { std::mem::transmute::<_, [u32; 5]>((*event_ptr).data)[0] == self.delete_window_atom }
	}
}
pub struct XWindow<'c>
{
	con_ref: &'c XServerConnection, internal: xcb::ffi::xcb_window_t
}
impl <'p> XWindow<'p>
{
	pub fn map(&self)
	{
		unsafe { xcb::ffi::xproto::xcb_map_window(self.con_ref.con.get_raw_conn(), self.internal) };
	}
}
impl <'p> InternalProvider<xcb::ffi::xcb_window_t> for XWindow<'p>
{
	fn get(&self) -> xcb::ffi::xcb_window_t { self.internal }
}
impl <'p> HasParent for XWindow<'p>
{
	type ParentRefType = &'p XServerConnection;
	fn parent(&self) -> &'p XServerConnection { self.con_ref }
}
