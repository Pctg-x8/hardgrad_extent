extern crate libc;
mod mainframe;
#[macro_use]
mod vkffi;
mod vulkan;
mod render;

// Only Defined in Platform
#[cfg(feature = "use_x11")] extern crate x11;
#[cfg(feature = "use_x11")] mod xorg;
#[cfg(windows)] extern crate winapi;
#[cfg(windows)] extern crate kernel32;
#[cfg(windows)] extern crate user32;
#[cfg(windows)] extern crate widestring;
#[cfg(windows)] mod win;

use mainframe::MainFrame;

fn main() { std::process::exit(MainFrame::launch_static()); }
