// XCB Objective Wrapper
#![allow(dead_code)]

use xcb; use std; use libc;
use traits::*;
use vkffi::*;
use render_vk::wrap as vk;

// XCB FFI Porting: size hints
pub const XCB_ICCCM_SIZE_HINT_US_SIZE: u32 = 1 << 1;
pub const XCB_ICCCM_SIZE_HINT_P_POSITION: u32 = 1 << 2;
pub const XCB_ICCCM_SIZE_HINT_P_SIZE: u32 = 1 << 3;
pub const XCB_ICCCM_SIZE_HINT_P_MIN_SIZE: u32 = 1 << 4;
pub const XCB_ICCCM_SIZE_HINT_P_MAX_SIZE: u32 = 1 << 5;
pub const XCB_ICCCM_SIZE_HINT_P_RESIZE_INC: u32 = 1 << 6;
pub const XCB_ICCCM_SIZE_HINT_P_ASPECT: u32 = 1 << 7;
pub const XCB_ICCCM_SIZE_HINT_BASE_SIZE: u32 = 1 << 8;
pub const XCB_ICCCM_SIZE_HINT_P_WIN_GRAVITY: u32 = 1 << 9;
#[repr(C)]
pub struct xcb_size_hints_t
{
	pub flags: u32, pub x: i32, pub y: i32,
	pub width: i32, pub height: i32,
	pub min_width: i32, pub min_height: i32,
	pub max_width: i32, pub max_height: i32,
	pub width_inc: i32, pub height_inc: i32,
	pub min_aspect_num: i32, pub min_aspect_den: i32,
	pub max_aspect_num: i32, pub max_aspect_den: i32,
	pub base_width: i32, pub base_height: i32,
	pub win_gravity: u32
}
impl std::default::Default for xcb_size_hints_t
{
	fn default() -> xcb_size_hints_t
	{
		xcb_size_hints_t
		{
			flags: 0, x: 0, y: 0,
			width: 0, height: 0,
			min_width: 0, min_height: 0,
			max_width: 0, max_height: 0,
			width_inc: 0, height_inc: 0,
			min_aspect_num: 0, min_aspect_den: 0,
			max_aspect_num: 0, max_aspect_den: 0,
			base_width: 0, base_height: 0,
			win_gravity: 0
		}
	}
}

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
	pub fn is_vk_presentation_support(&self, adapter: &vk::PhysicalDevice, queue_index: u32) -> bool
	{
		adapter.is_xcb_presentation_support(queue_index, self.con.get_raw_conn(), self.root_visual)
	}
	pub fn flush(&self) { self.con.flush(); }
	pub fn poll_event(&self) -> Option<xcb::GenericEvent> { self.con.poll_for_event() }
	pub fn get_raw(&self) -> *mut xcb::ffi::xcb_connection_t { self.con.get_raw_conn() }
	pub fn is_delete_window_message(&self, event_ptr: *const xcb::ffi::xproto::xcb_client_message_event_t) -> bool
	{
		unsafe { std::mem::transmute::<_, [u32; 5]>((*event_ptr).data)[0] == self.delete_window_atom }
	}
}
impl <'s> WindowProvider<XWindowHandle> for XServerConnection
{
	fn create_unresizable_window(&self, size: VkExtent2D, title: &str) -> XWindowHandle
	{
		let window_id = self.con.generate_id();
		let VkExtent2D(width, height) = size;
		// println!("creating window with resolution {}x{}", width, height);
		unsafe { xcb::ffi::xproto::xcb_create_window(self.con.get_raw_conn(), self.root_depth, window_id, self.root_screen,
			0, 0, width as u16, height as u16, 0, xcb::ffi::xproto::XCB_WINDOW_CLASS_INPUT_OUTPUT as u16, self.root_visual, 0, std::ptr::null()) };
		unsafe { xcb::ffi::xproto::xcb_change_property(self.con.get_raw_conn(), xcb::ffi::xproto::XCB_PROP_MODE_REPLACE as u8, window_id,
			xcb::xproto::ATOM_WM_NAME, xcb::xproto::ATOM_STRING, 8, title.len() as u32, title.as_ptr() as *const libc::c_void) };
		unsafe { xcb::ffi::xproto::xcb_change_property(self.con.get_raw_conn(), xcb::ffi::xproto::XCB_PROP_MODE_REPLACE as u8, window_id,
			self.protocols_atom, 4, 32, 1, std::mem::transmute(&self.delete_window_atom)) };

		let size_hints = xcb_size_hints_t
		{
			min_width: width as i32, min_height: height as i32,
			max_width: width as i32, max_height: height as i32,
			flags: XCB_ICCCM_SIZE_HINT_P_MAX_SIZE | XCB_ICCCM_SIZE_HINT_P_MIN_SIZE | XCB_ICCCM_SIZE_HINT_P_RESIZE_INC,
			.. Default::default()
		};
		unsafe { xcb::ffi::xproto::xcb_change_property(self.con.get_raw_conn(), xcb::ffi::xproto::XCB_PROP_MODE_REPLACE as u8, window_id,
			xcb::xproto::ATOM_WM_NORMAL_HINTS, xcb::xproto::ATOM_WM_SIZE_HINTS, 32, 1, std::mem::transmute(&size_hints)) };

		window_id
	}
	fn show_window(&self, handle: XWindowHandle)
	{
		unsafe { xcb::ffi::xproto::xcb_map_window(self.con.get_raw_conn(), handle) };
	}
}
impl MessageHandler for XServerConnection
{
	fn process_messages(&self) -> bool
	{
		fn recursive_process(this: &XServerConnection) -> bool
		{
			match this.poll_event()
			{
				Some(ev) =>
				{
					match unsafe { (*ev.ptr).response_type & 0x7f }
					{
						xcb::ffi::xproto::XCB_CLIENT_MESSAGE =>
						{
							let event_ptr = unsafe { std::mem::transmute::<_, *mut xcb::ffi::xproto::xcb_client_message_event_t>(ev.ptr) };
							if this.is_delete_window_message(event_ptr) { false } else { recursive_process(this) }
						},
						_ =>
						{
							println!("xcb event response: {}", unsafe { (*ev.ptr).response_type });
							recursive_process(this)
						}
					}
				},
				None => true
			}
		}

		recursive_process(self)
	}
}

pub type XWindowHandle = xcb::ffi::xcb_window_t;
