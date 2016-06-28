extern crate libc;
mod mainframe;
#[macro_use]
#[cfg(feature = "use_vk")] mod vkffi;
#[cfg(feature = "use_vk")] mod vulkan;
mod render;

// Only Defined in Platform
#[cfg(feature = "use_x11")] extern crate x11;
#[cfg(feature = "use_x11")] mod xorg;
#[cfg(feature = "use_win32")] extern crate winapi;
#[cfg(feature = "use_win32")] extern crate kernel32;
#[cfg(feature = "use_win32")] extern crate user32;
#[cfg(feature = "use_win32")] extern crate widestring;
#[cfg(feature = "use_win32")] mod win;

use mainframe::MainFrame;

fn main() { std::process::exit(MainFrame::launch_static()); }
