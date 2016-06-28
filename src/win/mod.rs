// Platform Dependency: Windows Module

use winapi::*;
use kernel32::*;
use user32::*;
use widestring::*;
use ::std;
use traits::*;
#[cfg(feature = "use_vk")] use vulkan as vk;
#[cfg(feature = "use_vk")] use vkffi::*;

pub struct Frame
{
	handle: HWND
}
pub fn create_frame() -> Frame
{
	println!("-- Launching System on Windows Platform");

	let app_instance = unsafe { GetModuleHandleW(std::ptr::null()) };
	let class_name = WideCString::from_str("hardgrad::extent").unwrap();
	let window_name = WideCString::from_str("HardGrad -> Extent").unwrap();
	let cursor = unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) };

	let wce = WNDCLASSEXW
	{
		cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32, cbClsExtra: 0, cbWndExtra: 0,
		lpszClassName: class_name.as_ptr(), lpszMenuName: std::ptr::null(),
		lpfnWndProc: Some(Frame::window_proc), hInstance: app_instance,
		hIcon: std::ptr::null_mut(), hIconSm: std::ptr::null_mut(), hbrBackground: std::ptr::null_mut(), hCursor: cursor,
		style: CS_OWNDC
	};
	if unsafe { RegisterClassExW(&wce) } != 0
	{
		let window_style = WS_OVERLAPPED | WS_BORDER | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
		let mut window_rect = RECT { left: 0, top: 0, right: 640, bottom: 480 };
		unsafe { AdjustWindowRectEx(&mut window_rect, window_style, FALSE, WS_EX_APPWINDOW) };
		let handle = Some(unsafe { CreateWindowExW(WS_EX_APPWINDOW, class_name.as_ptr(), window_name.as_ptr(), window_style,
			CW_USEDEFAULT, CW_USEDEFAULT, window_rect.right - window_rect.left, window_rect.bottom - window_rect.top,
			std::ptr::null_mut(), std::ptr::null_mut(), app_instance, std::ptr::null_mut()) }).expect("Failed to create window");

		Frame { handle: handle }
	}
	else { panic!("Failed to register window class"); }
}
impl Frame
{
	unsafe extern "system" fn window_proc(handle: HWND, message: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT
	{
		match message
		{
			WM_DESTROY => { PostQuitMessage(0); DefWindowProcW(handle, message, wp, lp) },
			_ => DefWindowProcW(handle, message, wp, lp)
		}
	}
}

#[cfg(feature = "use_vk")]
impl VkSurfaceProvider for Frame
{
	fn create_surface_vk(&self) -> Result<vk::Surface, VkResult>
	{

	}
}
