extern crate libc;

// Only Defined in Platform
#[cfg(feature = "use_x11")] extern crate x11;
#[cfg(feature = "use_x11")] mod xorg;
#[cfg(feature = "use_win32")] extern crate winapi;
#[cfg(feature = "use_win32")] extern crate kernel32;
#[cfg(feature = "use_win32")] extern crate user32;
#[cfg(feature = "use_win32")] extern crate widestring;
#[cfg(feature = "use_win32")] mod win;
#[macro_use]
#[cfg(feature = "use_vk")] mod vkffi;
#[cfg(feature = "use_vk")] mod vulkan;
#[cfg(feature = "use_d3d12")] extern crate d3d12;
#[cfg(feature = "use_d3d12")] extern crate dxgi;
#[cfg(feature = "use_d3d12")] extern crate dxguid;
#[cfg(feature = "use_d3d12")] mod d3d12_sw;

mod mainframe;
mod render;

use mainframe::MainFrame;

fn main() { std::process::exit(MainFrame::launch_static()); }
