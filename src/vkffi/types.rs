// Vulkan C to Rust FFI Type Aliases

pub type VkFlags = u32;
pub type VkBool32 = u32;
pub type VkDeviceSize = u64;
pub type VkSampleMask = u32;

pub type VkInstanceCreateFlags = VkFlags;
pub type VkImageUsageFlags = VkFlags;
#[cfg(windows)]
pub type VkWin32SurfaceCreateFlagsKHR = VkFlags;
pub type VkSurfaceTransformFlagsKHR = VkFlags;
pub type VkCompositeAlphaFlagsKHR = VkFlags;
pub type VkSwapchainCreateFlagsKHR = VkFlags;
