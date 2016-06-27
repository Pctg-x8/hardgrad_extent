extern crate libc;
extern crate x11;
mod mainframe;
#[macro_use]
mod vkffi;
mod vulkan;
mod render;

// Only Defined in Platform
#[cfg(feature = "use_x11")]
mod xorg;

use mainframe::MainFrame;

fn main() { std::process::exit(MainFrame::launch_static()); }
