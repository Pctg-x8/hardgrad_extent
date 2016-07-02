#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// Vulkan C to Rust FFI Structs and Handles

use std::os::raw::*;
use libc::size_t;
use vkffi::enums::*;
use vkffi::types::*;
use vkffi::functions::*;
use vkffi::objects::*;
use vkffi::macros::*;
use x11;

// Basic Types //
#[repr(C)] pub struct VkOffset2D(pub i32, pub i32);
#[repr(C)] #[derive(Clone, Copy)] pub struct VkExtent2D(pub u32, pub u32);
#[repr(C)] pub struct VkRect2D(pub VkOffset2D, pub VkExtent2D);
#[repr(C)] #[derive(Clone)] pub struct VkExtent3D(pub u32, pub u32, pub u32);

#[repr(C)]
pub struct VkInstanceCreateInfo
{
	pub sType: VkStructureType, pub pNext: *const c_void,
	pub flags: VkInstanceCreateFlags, pub pApplicationInfo: *const VkApplicationInfo,
	pub enabledLayerCount: u32, pub ppEnabledLayerNames: *const *const c_char,
	pub enabledExtensionCount: u32, pub ppEnabledExtensionNames: *const *const c_char
}
#[repr(C)]
pub struct VkApplicationInfo
{
	pub sType: VkStructureType, pub pNext: *const c_void,
	pub pApplicationName: *const c_char, pub applicationVersion: u32,
	pub pEngineName: *const c_char, pub engineVersion: u32,
	pub apiVersion: u32
}
#[repr(C)]
pub struct VkAllocationCallbacks
{
	pub pUserData: *mut c_void,
	pub pfnAllocation: PFN_vkAllocationFunction,
	pub pfnReallocation: PFN_vkReallocationFunction,
	pub fnFree: PFN_vkFreeFunction,
	pub pfnInternalAllocation: PFN_vkInternalAllocationNotification,
	pub pfnInternalFree: PFN_vkInternalFreeNotification
}
#[repr(C)]
pub struct VkPhysicalDeviceFeatures
{
	pub robustBufferAccess: VkBool32,
	pub fullDrawIndexUint32: VkBool32,
	pub imageCubeArray: VkBool32,
	pub independentBlend: VkBool32,
	pub geometryShader: VkBool32,
	pub tessellationShader: VkBool32,
	pub sampleRateShading: VkBool32,
	pub dualSrcBlend: VkBool32,
	pub logicOp: VkBool32,
	pub multiDrawIndirect: VkBool32,
	pub drawIndirectFirstInstance: VkBool32,
	pub depthClamp: VkBool32,
	pub depthBiasClamp: VkBool32,
	pub fillModeNonSolid: VkBool32,
	pub depthBounds: VkBool32,
	pub wideLines: VkBool32,
	pub largePoints: VkBool32,
	pub alphaToOne: VkBool32,
	pub multiViewport: VkBool32,
	pub samplerAnisotropy: VkBool32,
	pub textureCompressionETC2: VkBool32,
	pub textureCompressionASTC_LDR: VkBool32,
	pub textureCompressionBC: VkBool32,
	pub occlusionQueryPrecise: VkBool32,
	pub pipelineStatisticsQuery: VkBool32,
	pub vertexPipelineStoresAndAtomics: VkBool32,
	pub fragmentStoresAndAtomics: VkBool32,
	pub shaderTessellationAndGeometryPointSize: VkBool32,
	pub shaderImageGatherExtended: VkBool32,
	pub shaderStorageImageExtendedFormats: VkBool32,
	pub shaderStorageImageMultisample: VkBool32,
	pub shaderStorageImageReadWithoutFormat: VkBool32,
	pub shaderStorageImageWriteWithoutFormat: VkBool32,
	pub shaderUniformBufferArrayDynamicIndexing: VkBool32,
	pub shaderSampledImageArrayDynamicIndexing: VkBool32,
	pub shaderStorageBufferArrayDynamicIndexing: VkBool32,
	pub shaderStorageImageArrayDynamicIndexing: VkBool32,
	pub shaderClipDistance: VkBool32,
	pub shaderCullDistance: VkBool32,
	pub shaderFloat64: VkBool32,
	pub shaderInt64: VkBool32,
	pub shaderInt16: VkBool32,
	pub shaderResourceResidency: VkBool32,
	pub shaderResoruceMinLod: VkBool32,
	pub sparseBinding: VkBool32,
	pub sparseResidencyBuffer: VkBool32,
	pub sparseResidencyImage2D: VkBool32,
	pub sparseResidencyImage3D: VkBool32,
	pub sparseResidency2Samples: VkBool32,
	pub sparseResidency4SAmples: VkBool32,
	pub sparseResidency8Samples: VkBool32,
	pub sparseResidency16Samples: VkBool32,
	pub sparseResidencyAliased: VkBool32,
	pub variableMultisampleRate: VkBool32,
	pub inheritedQueries: VkBool32
}
#[repr(C)] pub struct VkPhysicalDeviceLimits
{
    pub maxImageDimension1D: u32,
    pub maxImageDimension2D: u32,
    pub maxImageDimension3D: u32,
    pub maxImageDimensionCube: u32,
    pub maxImageArrayLayers: u32,
    pub maxTexelBufferElements: u32,
    pub maxUniformBufferRange: u32,
    pub maxStorageBufferRange: u32,
    pub maxPushConstantsSize: u32,
    pub maxMemoryAllocationCount: u32,
    pub maxSamplerAllocationCount: u32,
    pub bufferImageGranularity: VkDeviceSize,
    pub sparseAddressSpaceSize: VkDeviceSize,
    pub maxBoundDescriptorSets: u32,
    pub maxPerStageDescriptorSamplers: u32,
    pub maxPerStageDescriptorUniformBuffers: u32,
    pub maxPerStageDescriptorStorageBuffers: u32,
    pub maxPerStageDescriptorSampledImages: u32,
    pub maxPerStageDescriptorStorageImages: u32,
    pub maxPerStageDescriptorInputAttachments: u32,
    pub maxPerStageResources: u32,
    pub maxDescriptorSetSamplers: u32,
    pub maxDescriptorSetUniformBuffers: u32,
    pub maxDescriptorSetUniformBuffersDynamic: u32,
    pub maxDescriptorSetStorageBuffers: u32,
    pub maxDescriptorSetStorageBuffersDynamic: u32,
    pub maxDescriptorSetSampledImages: u32,
    pub maxDescriptorSetStorageImages: u32,
    pub maxDescriptorSetInputAttachments: u32,
    pub maxVertexInputAttributes: u32,
    pub maxVertexInputBindings: u32,
    pub maxVertexInputAttributeOffset: u32,
    pub maxVertexInputBindingStride: u32,
    pub maxVertexOutputComponents: u32,
    pub maxTessellationGenerationLevel: u32,
    pub maxTessellationPatchSize: u32,
    pub maxTessellationControlPerVertexInputComponents: u32,
    pub maxTessellationControlPerVertexOutputComponents: u32,
    pub maxTessellationControlPerPatchOutputComponents: u32,
    pub maxTessellationControlTotalOutputComponents: u32,
    pub maxTessellationEvaluationInputComponents: u32,
    pub maxTessellationEvaluationOutputComponents: u32,
    pub maxGeometryShaderInvocations: u32,
    pub maxGeometryInputComponents: u32,
    pub maxGeometryOutputComponents: u32,
    pub maxGeometryOutputVertices: u32,
    pub maxGeometryTotalOutputComponents: u32,
    pub maxFragmentInputComponents: u32,
    pub maxFragmentOutputAttachments: u32,
    pub maxFragmentDualSrcAttachments: u32,
    pub maxFragmentCombinedOutputResources: u32,
    pub maxComputeSharedMemorySize: u32,
    pub maxComputeWorkGroupCount: [u32; 3],
    pub maxComputeWorkGroupInvocations: u32,
    pub maxComputeWorkGroupSize: [u32; 3],
    pub subPixelPrecisionBits: u32,
    pub subTexelPrecisionBits: u32,
    pub mipmapPrecisionBits: u32,
    pub maxDrawIndexedIndexValue: u32,
    pub maxDrawIndirectCount: u32,
    pub maxSamplerLodBias: f32,
    pub maxSamplerAnisotropy: f32,
    pub maxViewports: u32,
    pub maxViewportDimensions: [u32; 2],
    pub viewportBoundsRange: [f32; 2],
    pub viewportSubPixelBits: u32,
    pub minMemoryMapAlignment: size_t,
    pub minTexelBufferOffsetAlignment: VkDeviceSize,
    pub minUniformBufferOffsetAlignment: VkDeviceSize,
    pub minStorageBufferOffsetAlignment: VkDeviceSize,
    pub minTexelOffset: i32,
    pub maxTexelOffset: u32,
    pub minTexelGatherOffset: i32,
    pub maxTexelGatherOffset: u32,
    pub minInterpolationOffset: f32,
    pub maxInterpolationOffset: f32,
    pub subPixelInterpolationOffsetBits: u32,
    pub maxFramebufferWidth: u32,
    pub maxFramebufferHeight: u32,
    pub maxFramebufferLayers: u32,
    pub framebufferColorSampleCounts: VkSampleCountFlags,
    pub framebufferDepthSampleCounts: VkSampleCountFlags,
    pub framebufferStencilSampleCounts: VkSampleCountFlags,
    pub framebufferNoAttachmentsSampleCounts: VkSampleCountFlags,
    pub maxColorAttachments: u32,
    pub sampledImageColorSampleCounts: VkSampleCountFlags,
    pub sampledImageIntegerSampleCounts: VkSampleCountFlags,
    pub sampledImageDepthSampleCounts: VkSampleCountFlags,
    pub sampledImageStencilSampleCounts: VkSampleCountFlags,
    pub storageImageSampleCounts: VkSampleCountFlags,
    pub maxSampleMaskWords: u32,
    pub timestampComputeAndGraphics: VkBool32,
    pub timestampPeriod: f32,
    pub maxClipDistances: u32,
    pub maxCullDistances: u32,
    pub maxCombinedClipAndCullDistances: u32,
    pub discreteQueuePriorities: u32,
    pub pointSizeRange: [f32; 2],
    pub lineWidthRange: [f32; 2],
    pub pointSizeGranularity: f32,
    pub lineWidthGranularity: f32,
    pub strictLines: VkBool32,
    pub standardSampleLocations: VkBool32,
    pub optimalBufferCopyOffsetAlignment: VkDeviceSize,
    pub optimalBufferCopyRowPitchAlignment: VkDeviceSize,
    pub nonCoherentAtomSize: VkDeviceSize
}
#[repr(C)] pub struct VkPhysicalDeviceSparseProperties
{
	pub residencyStandard2DBlockShape: VkBool32,
	pub residencyStandard2DMultisampleBlockShape: VkBool32,
	pub residencyStandard3DBlockShape: VkBool32,
	pub residencyAlignedMipSize: VkBool32,
	pub residencyNonResidentStrict: VkBool32
}
#[repr(C)] pub struct VkPhysicalDeviceProperties
{
	pub apiVersion: u32, pub driverVersion: u32, pub vendorID: u32, pub deviceID: u32,
	pub deviceType: VkPhysicalDeviceType, pub deviceName: [c_char; VK_MAX_PHYSICAL_DEVICE_NAME_SIZE],
	pub pipelineCacheUUID: [u8; VK_UUID_SIZE], pub limits: VkPhysicalDeviceLimits,
	pub sparseProperties: VkPhysicalDeviceSparseProperties
}
#[repr(C)] #[derive(Clone)] pub struct VkQueueFamilyProperties
{
	pub queueFlags: VkQueueFlags, pub queueCount: u32, pub timestampValidBits: u32,
	pub minImageTransferGranularity: VkExtent3D
}
#[repr(C)] pub struct VkPhysicalDeviceMemoryProperties
{
	pub memoryTypeCount: u32, pub memoryTypes: [VkMemoryType; VK_MAX_MEMORY_TYPES],
	pub memoryHeapCount: u32, pub memoryHeaps: [VkMemoryHeap; VK_MAX_MEMORY_HEAPS]
}
#[repr(C)] pub struct VkMemoryType(VkMemoryPropertyFlags, u32);
#[repr(C)] pub struct VkMemoryHeap(VkDeviceSize, VkMemoryHeapFlags);
#[repr(C)]
pub struct VkDeviceQueueCreateInfo
{
	pub sType: VkStructureType,
	pub pNext: *const c_void,
	pub flags: VkDeviceQueueCreateFlags,
	pub queueFamilyIndex: u32,
	pub queueCount: u32,pub pQueuePriorities: *const f32
}
#[repr(C)]
pub struct VkDeviceCreateInfo
{
	pub sType: VkStructureType,
	pub pNext: *const c_void,
	pub flags: VkDeviceCreateFlags,
	pub queueCreateInfoCount: u32,
	pub pQueueCreateInfos: *const VkDeviceQueueCreateInfo,
	pub enabledLayerCount: u32,
	pub ppEnabledLayerNames: *const *const c_char,
	pub enabledExtensionCount: u32,
	pub ppEnabledExtensionNames: *const *const c_char,
	pub pEnabledFeatures: *const VkPhysicalDeviceFeatures
}
#[repr(C)] pub struct VkImageViewCreateInfo
{
	pub sType: VkStructureType, pub pNext: *const c_void,
	pub flags: VkImageViewCreateFlags, pub image: VkImage,
	pub viewType: VkImageViewType, pub format: VkFormat,
	pub components: VkComponentMapping, pub subresourceRange: VkImageSubresourceRange
}
#[repr(C)] pub struct VkComponentMapping
{
	pub r: VkComponentSwizzle, pub g: VkComponentSwizzle, pub b: VkComponentSwizzle, pub a: VkComponentSwizzle
}
#[repr(C)] pub struct VkImageSubresourceRange
{
	pub aspectMask: VkImageAspectFlags,
	pub baseMipLevel: u32, pub levelCount: u32,
	pub baseArrayLayer: u32, pub layerCount: u32
}
#[repr(C)] pub struct VkAttachmentDescription
{
	pub flags: VkAttachmentDescriptionFlags, pub format: VkFormat,
	pub samples: VkSampleCountFlagBits,
	pub loadOp: VkAttachmentLoadOp, pub storeOp: VkAttachmentStoreOp,
	pub stencilLoadOp: VkAttachmentLoadOp, pub stencilStoreOp: VkAttachmentStoreOp,
	pub initialLayout: VkImageLayout, pub finalLayout: VkImageLayout
}
#[repr(C)] pub struct VkAttachmentReference
{
	pub attachment: u32, pub layout: VkImageLayout
}
#[repr(C)] pub struct VkSubpassDescription
{
	pub flags: VkSubpassDescriptionFlags, pub pipelineBindPoint: VkPipelineBindPoint,
	pub inputAttachmentCount: u32, pub pInputAttachments: *const VkAttachmentReference,
	pub colorAttachmentCount: u32, pub pColorAttachments: *const VkAttachmentReference,
	pub pResolveAttachments: *const VkAttachmentReference, pub pDepthStencilAttachment: *const VkAttachmentReference,
	pub preserveAttachmentCount: u32, pub pPreserveAttachments: *const u32
}
#[repr(C)] pub struct VkSubpassDependency
{
	pub srcSubpass: u32, pub dstSubpass: u32,
	pub srcStageMask: VkPipelineStageFlags, pub dstStageMask: VkPipelineStageFlags,
	pub srcAccessMask: VkAccessFlags, pub dstAccessMask: VkAccessFlags,
	pub dependencyFlags: VkDependencyFlags
}
#[repr(C)] pub struct VkRenderPassCreateInfo
{
	pub sType: VkStructureType, pub pNext: *const c_void, pub flags: VkRenderPassCreateFlags,
	pub attachmentCount: u32, pub pAttachments: *const VkAttachmentDescription,
	pub subpassCount: u32, pub pSubpasses: *const VkSubpassDescription,
	pub dependencyCount: u32, pub pDependencies: *const VkSubpassDependency
}
#[repr(C)] pub struct VkFramebufferCreateInfo
{
	pub sType: VkStructureType, pub pNext: *const c_void, pub flags: VkFramebufferCreateFlags,
	pub renderPass: VkRenderPass, pub attachmentCount: u32, pub pAttachments: *const VkImageView,
	pub width: u32, pub height: u32, pub layers: u32
}

#[repr(C)]
pub struct VkXlibSurfaceCreateInfoKHR
{
	pub sType: VkStructureType, pub pNext: *const c_void, pub flags: VkXlibSurfaceCreateFlagsKHR,
	pub dpy: *mut x11::xlib::Display, pub window: x11::xlib::Window
}
#[repr(C)]
pub struct VkSurfaceCapabilitiesKHR
{
	pub minImageCount: u32,
	pub maxImageCount: u32,
	pub currentExtent: VkExtent2D,
	pub minImageExtent: VkExtent2D,
	pub maxImageExtent: VkExtent2D,
	pub maxImageArrayLayers: u32,
	pub supportedTransforms: VkSurfaceTransformFlagsKHR,
	pub currentTransform: VkSurfaceTransformFlagBitsKHR,
	pub supportedCompositeAlpha: VkCompositeAlphaFlagsKHR,
	pub supportedUsageFlags: VkImageUsageFlags
}
#[repr(C)] #[derive(Clone)]
pub struct VkSurfaceFormatKHR
{
	pub format: VkFormat,
	pub colorSpace: VkColorSpaceKHR
}

#[repr(C)]
pub struct VkSwapchainCreateInfoKHR
{
	pub sType: VkStructureType,
	pub pNext: *const c_void,
	pub flags: VkSwapchainCreateFlagsKHR,
	pub surface: VkSurfaceKHR,
	pub minImageCount: u32,
	pub imageFormat: VkFormat,
	pub imageColorSpace: VkColorSpaceKHR,
	pub imageExtent: VkExtent2D,
	pub imageArrayLayers: u32,
	pub imageUsage: VkImageUsageFlags,
	pub imageSharingMode: VkSharingMode,
	pub queueFamilyIndexCount: u32,
	pub pQueueFamilyIndices: *const u32,
	pub preTransform: VkSurfaceTransformFlagBitsKHR,
	pub compositeAlpha: VkCompositeAlphaFlagBitsKHR,
	pub presentMode: VkPresentModeKHR,
	pub clipped: VkBool32,
	pub oldSwapchain: VkSwapchainKHR
}
#[repr(C)]
pub struct VkPresentInfoKHR
{
	pub sType: VkStructureType,
	pub pNext: *const c_void,
	pub waitSemaphoreCount: u32,
	pub pWaitSemaphores: *const VkSemaphore,
	pub swapchainCount: u32,
	pub pSwapchains: *const VkSwapchainKHR,
	pub pImageIndices: *const u32,
	pub pResults: *mut VkResult
}

#[repr(C)] pub struct VkDebugReportCallbackCreateInfoEXT
{
	pub sType: VkStructureType, pub pNext: *const c_void,
	pub flags: VkDebugReportFlagsEXT, pub pfnCallback: PFN_vkDebugReportCallbackEXT,
	pub pUserData: *mut c_void
}
