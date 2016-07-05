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
pub type VkImageViewCreateFlags = VkFlags;
pub type VkImageAspectFlags = VkFlags;
pub type VkAttachmentDescriptionFlags = VkFlags;
pub type VkDescriptorPoolCreateFlags = VkFlags;
pub type VkDescriptorPoolResetFlags = VkFlags;
pub type VkFramebufferCreateFlags = VkFlags;
pub type VkRenderPassCreateFlags = VkFlags;
pub type VkSubpassDescriptionFlags = VkFlags;
pub type VkAccessFlags = VkFlags;
pub type VkSampleCountFlagBits = VkFlags;
pub type VkPipelineStageFlags = VkFlags;
pub type VkMemoryMapFlags = VkFlags;
pub type VkDependencyFlags = VkFlags;

pub type VkXcbSurfaceCreateFlagsKHR = VkFlags;
pub type VkSurfaceTransformFlagsKHR = VkFlags;
pub type VkCompositeAlphaFlagsKHR = VkFlags;
pub type VkSwapchainCreateFlagsKHR = VkFlags;

pub type VkDebugReportFlagsEXT = VkFlags;
