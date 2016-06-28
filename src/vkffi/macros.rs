#![allow(dead_code)]

// Vulkan C to Rust FFI Macros/Compile-Time functions

pub const VK_VERSION_1_0: u32 = 1;

macro_rules! VK_MAKE_VERSION
{
	($major: expr, $minor: expr, $patch: expr) => (($major << 22) | ($minor << 12) | $patch)
}
pub const VK_API_VERSION_1_0: u32 = VK_MAKE_VERSION!(1, 0, 0);