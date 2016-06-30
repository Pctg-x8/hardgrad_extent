// Vulkan C to Rust FFI Type Aliases

pub type VkFlags = u32;
pub type VkBool32 = u32;
pub type VkDeviceSize = u64;
pub type VkSampleMask = u32;

pub type VkInstanceCreateFlags = VkFlags;
pub type VkSampleCountFlags = VkFlags;
pub type VkQueueFlags = VkFlags;
pub type VkMemoryPropertyFlags = VkFlags;
pub type VkMemoryHeapFlags = VkFlags;
pub type VkDeviceCreateFlags = VkFlags;
pub type VkDeviceQueueCreateFlags = VkFlags;
pub type VkImageUsageFlags = VkFlags;
pub type VkXlibSurfaceCreateFlagsKHR = VkFlags;
pub type VkSurfaceTransformFlagsKHR = VkFlags;
pub type VkCompositeAlphaFlagsKHR = VkFlags;
pub type VkSwapchainCreateFlagsKHR = VkFlags;
pub type VkDebugReportFlagsEXT = VkFlags;
