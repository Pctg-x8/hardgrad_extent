#![allow(non_snake_case)]
#![allow(dead_code)]

// Vulkan C to Rust FFI Enumerations

#[repr(C)]
pub enum VkSystemAllocationScope
{
	Command = 0,
	Object = 1,
	Cache = 2,
	Device = 3,
	Instance = 4
}
#[repr(C)]
pub enum VkInternalAllocationType
{
	Executable = 0
}
#[repr(C)]
pub enum VkStructureType
{
	ApplicationInfo = 0,
	InstanceCreateInfo = 1,
	DeviceQueueCreateInfo = 2,
	DeviceCreateInfo = 3,
	SubmitInfo = 4,
	MemoryAllocateInfo = 5,
	MappedMemoryRange = 6,
	BindSparseInfo = 7,
	FenceCreateInfo = 8,
	SemaphoreCreateInfo = 9,
	EventCreateInfo = 10,
	QueryPoolCreateInfo = 11,
	BufferCreateInfo = 12,
	BufferViewCreateInfo = 13,
	ImageCreateInfo = 14,
	ImageViewCreateInfo = 15,
	ShaderModuleCreateInfo = 16,
	Pipeline_CacheCreateInfo = 17,
	Pipeline_ShaderStageCreateInfo = 18,
	Pipeline_VertexInputStateCreateInfo = 19,
	Pipeline_InputAssemblyStateCreateInfo = 20,
	Pipeline_TessellationStateCreateInfo = 21,
	Pipeline_ViewportStateCreateInfo = 22,
	Pipeline_RasterizationStateCreateInfo = 23,
	Pipeline_MultisampleStateCreateInfo = 24,
	Pipeline_DepthStencilStateCreateInfo = 25,
	Pipeline_ColorBlendstateCreateInfo = 26,
	Pipeline_DynamicStateCreateInfo = 27,
	GraphicsPipelineCreateInfo = 28,
	ComputePipelineCreateInfo = 29,
	PipelineLayoutCreateInfo = 30,
	SamplerCreateInfo = 31,
	DescriptorSetLayoutCreateInfo = 32,
	DescriptorPoolCreateInfo = 33,
	DescriptorSetAllocateInfo = 34,
	WriteDescriptorSet = 35,
	CopyDescriptorSet = 36,
	FramebufferCreateInfo = 37,
	RenderPassCreateInfo = 38,
	CommandPoolCreateInfo = 39,
	CommandBufferAllocateInfo = 40,
	CommandBufferInheritanceInfo = 41,
	CommandBufferBeginInfo = 42,
	RenderPassBeginInfo = 43,
	BufferMemoryBarrier = 44,
	ImageMemoryBarrier = 45,
	MemoryBarrier = 46,
	LoaderInstanceCreateInfo = 47,
	LoaderDeviceCreateInfo = 48,
	SwapchainCreateInfoKHR = 1000001000,
	PresentInfoKHr= 1000001001,
	DisplayModeCreateInfoKHR = 1000002000,
	DisplaySurfaceCreateInfoKHR = 1000002001,
	DisplayPresentInfoKHR = 1000003000,
	XlibSurfaceCreateInfoKHR = 10000040000,
	XcbSurfaceCreateInfoKHR = 1000005000,
	WaylandSurfaceCreateInfoKHR = 1000006000,
	MIRSurfaceCreateInfoKHR = 1000007000,
	AndroidSurfaceCreateInfoKHR = 1000008000,
	Win32SurfaceCreateInfoKHR = 1000009000,
	DebugReportCallbackCreateInfoEXT = 1000011000,
	Pipeline_RasterizationState_RasterizationOrderAMD = 1000018000,
	DebugMarker_ObjectNameInfoEXT = 1000022000,
	DebugMarker_ObjectTagInfoEXT = 1000022001,
	DebugMarker_MarkerInfoEXT = 1000022002
}

#[repr(C)] #[derive(PartialEq, Eq, Debug)]
pub enum VkResult
{
	Success = 0,
	NotReady = 1,
	Timeout = 2,
	EventSet = 3,
	EventReset = 4,
	Incomplete = 5,
	Error_OutOfHostMemory = -1,
	Error_OutOfDeviceMemory = -2,
	Error_InitializationFailed = -3,
	Error_DeviceLost = -4,
	Error_MemoryMapFailed = -5,
	Error_LayerNotPresented = -6,
	Error_ExtensionNotPresented = -7,
	Error_FeatureNotPresent = -8,
	Error_IncompatibleDriver = -9,
	Error_TooManyObjects = -10,
	Error_FormatNotSupported = -11,
	Error_SurfaceLostKHR = -1000000000,
	Error_NativeWindowInUseKHR = -1000000001,
	SuboptimalKHR = 1000001003,
	Error_OutOfDateKHR = -1000001004,
	Error_IncompatibleDisplayKHR = -1000003001,
	Error_ValidationFailedEXT = -1000011001,
	Error_InvalidShaderNV = -1000012000
}
