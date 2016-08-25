// Prelude: Resources(Buffer and Image)

use prelude::internals::*;
use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use render_vk::traits::*;
use std::os::raw::c_void;

pub trait Resource { fn get_memory_requirements(&self) -> VkMemoryRequirements; }
pub trait BufferResource { fn get_resource(&self) -> VkBuffer; }
pub trait ImageResource { fn get_resource(&self) -> VkImage; }
pub trait DescriptedImage : std::marker::Sized { fn new(engine: &Engine, desc: &VkImageCreateInfo) -> Result<Self, EngineError>; }

pub trait BufferInternals : std::marker::Sized
{
	fn new(engine: &Engine, size: VkDeviceSize, usage: VkBufferUsageFlags) -> Result<Self, EngineError>;
}
pub trait LinearImage2DInternals : std::marker::Sized
{
	fn new(engine: &Engine, size: VkExtent2D, format: VkFormat) -> Result<Self, EngineError>;
}
pub struct Buffer { internal: vk::Buffer, size: VkDeviceSize }
impl Resource for Buffer { fn get_memory_requirements(&self) -> VkMemoryRequirements { self.internal.get_memory_requirements() } }
impl BufferResource for Buffer { fn get_resource(&self) -> VkBuffer { self.internal.get() } }
pub struct Image1D { internal: vk::Image, size: u32 }
pub struct Image2D { internal: vk::Image, size: VkExtent2D }
pub struct LinearImage2D { internal: vk::Image, size: VkExtent2D }
pub struct Image3D { internal: vk::Image, size: VkExtent3D }
impl Resource for Image1D { fn get_memory_requirements(&self) -> VkMemoryRequirements { self.internal.get_memory_requirements() } }
impl Resource for Image2D { fn get_memory_requirements(&self) -> VkMemoryRequirements { self.internal.get_memory_requirements() } }
impl Resource for LinearImage2D { fn get_memory_requirements(&self) -> VkMemoryRequirements { self.internal.get_memory_requirements() } }
impl Resource for Image3D { fn get_memory_requirements(&self) -> VkMemoryRequirements { self.internal.get_memory_requirements() } }
impl ImageResource for Image1D { fn get_resource(&self) -> VkImage { self.internal.get() } }
impl ImageResource for Image2D { fn get_resource(&self) -> VkImage { self.internal.get() } }
impl ImageResource for LinearImage2D { fn get_resource(&self) -> VkImage { self.internal.get() } }
impl ImageResource for Image3D { fn get_resource(&self) -> VkImage { self.internal.get() } }
impl InternalExports<vk::Image> for Image1D { fn get_internal(&self) -> &vk::Image { &self.internal } }
impl InternalExports<vk::Image> for Image2D { fn get_internal(&self) -> &vk::Image { &self.internal } }
impl InternalExports<vk::Image> for LinearImage2D { fn get_internal(&self) -> &vk::Image { &self.internal } }
impl InternalExports<vk::Image> for Image3D { fn get_internal(&self) -> &vk::Image { &self.internal } }
impl BufferInternals for Buffer
{
	fn new(engine: &Engine, size: VkDeviceSize, usage: VkBufferUsageFlags) -> Result<Self, EngineError>
	{
		vk::Buffer::new(engine.get_device().get_internal(), &VkBufferCreateInfo
		{
			sType: VkStructureType::BufferCreateInfo, pNext: std::ptr::null(), flags: 0,
			size: size, usage: usage, sharingMode: VkSharingMode::Exclusive,
			queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null(),
		}).map(|buf| Buffer { internal: buf, size: size }).map_err(EngineError::from)
	}
}
impl LinearImage2DInternals for LinearImage2D
{
	fn new(engine: &Engine, size: VkExtent2D, format: VkFormat) -> Result<Self, EngineError>
	{
		let VkExtent2D(width, height) = size;
		vk::Image::new(engine.get_device().get_internal(), &VkImageCreateInfo
		{
			sType: VkStructureType::ImageCreateInfo, pNext: std::ptr::null(), flags: 0,
			imageType: VkImageType::Dim2, format: format, extent: VkExtent3D(width, height, 1),
			mipLevels: 1, arrayLayers: 1, samples: VK_SAMPLE_COUNT_1_BIT, tiling: VkImageTiling::Linear,
			usage: VK_IMAGE_USAGE_TRANSFER_SRC_BIT, sharingMode: VkSharingMode::Exclusive,
			initialLayout: VkImageLayout::Preinitialized,
			queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null()
		}).map(|img| LinearImage2D { internal: img, size: size }).map_err(EngineError::from)
	}
}
impl DescriptedImage for Image1D
{
	fn new(engine: &Engine, desc: &VkImageCreateInfo) -> Result<Self, EngineError>
	{
		vk::Image::new(engine.get_device().get_internal(), &VkImageCreateInfo
		{
			usage: desc.usage | VK_IMAGE_USAGE_TRANSFER_DST_BIT,
			.. *desc
		}).map(|img| Image1D { internal: img, size: desc.extent.0 }).map_err(EngineError::from)
	}
}
impl DescriptedImage for Image2D
{
	fn new(engine: &Engine, desc: &VkImageCreateInfo) -> Result<Self, EngineError>
	{
		vk::Image::new(engine.get_device().get_internal(), &VkImageCreateInfo
		{
			usage: desc.usage | VK_IMAGE_USAGE_TRANSFER_DST_BIT,
			.. *desc
		}).map(|img| Image2D { internal: img, size: VkExtent2D(desc.extent.0, desc.extent.1) }).map_err(EngineError::from)
	}
}
impl DescriptedImage for Image3D
{
	fn new(engine: &Engine, desc: &VkImageCreateInfo) -> Result<Self, EngineError>
	{
		vk::Image::new(engine.get_device().get_internal(), &VkImageCreateInfo
		{
			usage: desc.usage | VK_IMAGE_USAGE_TRANSFER_DST_BIT,
			.. *desc
		}).map(|img| Image3D { internal: img, size: desc.extent }).map_err(EngineError::from)
	}
}
pub enum SampleCount { Bit1, Bit2, Bit4, Bit8, Bit16, Bit32, Bit64 }
pub trait ImageDescriptor : std::marker::Sized + InternalExports<VkImageCreateInfo>
{
	fn mip_levels(mut self, levels: u32) -> Self;
	fn array_layers(mut self, layers: u32) -> Self;
	fn sample_flags(mut self, samples: &[SampleCount]) -> Self;
}
pub struct ImageDescriptor1 { internal: VkImageCreateInfo }
pub struct ImageDescriptor2 { internal: VkImageCreateInfo }
pub struct ImageDescriptor3 { internal: VkImageCreateInfo }
impl ImageDescriptor1
{
	pub fn new(format: VkFormat, extent: u32, usage: VkImageUsageFlags) -> Self
	{
		ImageDescriptor1
		{
			internal: VkImageCreateInfo
			{
				sType: VkStructureType::ImageCreateInfo, pNext: std::ptr::null(), flags: 0,
				imageType: VkImageType::Dim1, format: format, extent: VkExtent3D(extent, 1, 1),
				mipLevels: 1, arrayLayers: 1, samples: VK_SAMPLE_COUNT_1_BIT, tiling: VkImageTiling::Optimal,
				usage: usage, sharingMode: VkSharingMode::Exclusive, initialLayout: VkImageLayout::Preinitialized,
				queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null()
			}
		}
	}
}
impl ImageDescriptor2
{
	pub fn new(format: VkFormat, extent: VkExtent2D, usage: VkImageUsageFlags) -> Self
	{
		let VkExtent2D(width, height) = extent;
		ImageDescriptor2
		{
			internal: VkImageCreateInfo
			{
				sType: VkStructureType::ImageCreateInfo, pNext: std::ptr::null(), flags: 0,
				imageType: VkImageType::Dim2, format: format, extent: VkExtent3D(width, height, 1),
				mipLevels: 1, arrayLayers: 1, samples: VK_SAMPLE_COUNT_1_BIT, tiling: VkImageTiling::Optimal,
				usage: usage, sharingMode: VkSharingMode::Exclusive, initialLayout: VkImageLayout::Preinitialized,
				queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null()
			}
		}
	}
}
impl ImageDescriptor3
{
	pub fn new(format: VkFormat, extent: VkExtent3D, usage: VkImageUsageFlags) -> Self
	{
		ImageDescriptor3
		{
			internal: VkImageCreateInfo
			{
				sType: VkStructureType::ImageCreateInfo, pNext: std::ptr::null(), flags: 0,
				imageType: VkImageType::Dim3, format: format, extent: extent,
				mipLevels: 1, arrayLayers: 1, samples: VK_SAMPLE_COUNT_1_BIT, tiling: VkImageTiling::Optimal,
				usage: usage, sharingMode: VkSharingMode::Exclusive, initialLayout: VkImageLayout::Preinitialized,
				queueFamilyIndexCount: 0, pQueueFamilyIndices: std::ptr::null()
			}
		}
	}
}
impl InternalExports<VkImageCreateInfo> for ImageDescriptor1 { fn get_internal(&self) -> &VkImageCreateInfo { &self.internal } }
impl InternalExports<VkImageCreateInfo> for ImageDescriptor2 { fn get_internal(&self) -> &VkImageCreateInfo { &self.internal } }
impl InternalExports<VkImageCreateInfo> for ImageDescriptor3 { fn get_internal(&self) -> &VkImageCreateInfo { &self.internal } }
macro_rules! ImplImageDescriptor
{
	(for $t: ty) =>
	{
		impl ImageDescriptor for $t
		{
			fn mip_levels(mut self, levels: u32) -> Self
			{
				self.internal.mipLevels = levels;
				self
			}
			fn array_layers(mut self, layers: u32) -> Self
			{
				self.internal.arrayLayers = layers;
				self
			}
			fn sample_flags(mut self, samples: &[SampleCount]) -> Self
			{
				self.internal.samples = samples.into_iter().fold(0, |flags, c| match c
				{
					&SampleCount::Bit1 => flags | VK_SAMPLE_COUNT_1_BIT,
					&SampleCount::Bit2 => flags | VK_SAMPLE_COUNT_2_BIT,
					&SampleCount::Bit4 => flags | VK_SAMPLE_COUNT_4_BIT,
					&SampleCount::Bit8 => flags | VK_SAMPLE_COUNT_8_BIT,
					&SampleCount::Bit16 => flags | VK_SAMPLE_COUNT_16_BIT,
					&SampleCount::Bit32 => flags | VK_SAMPLE_COUNT_32_BIT,
					&SampleCount::Bit64 => flags | VK_SAMPLE_COUNT_64_BIT
				});
				self
			}
		}
	}
}
ImplImageDescriptor!(for ImageDescriptor1);
ImplImageDescriptor!(for ImageDescriptor2);
ImplImageDescriptor!(for ImageDescriptor3);

pub struct ImageSubresourceRange(VkImageAspectFlags, u32, u32, u32, u32);
impl ImageSubresourceRange
{
	pub fn base_color() -> Self
	{
		ImageSubresourceRange(VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1)
	}
}
impl std::convert::Into<VkImageSubresourceRange> for ImageSubresourceRange
{
	fn into(self) -> VkImageSubresourceRange { (&self).into() }
}
impl <'a> std::convert::Into<VkImageSubresourceRange> for &'a ImageSubresourceRange
{
	fn into(self) -> VkImageSubresourceRange
	{
		let ImageSubresourceRange(aspect, base_mip, level_count, base_array, layer_count) = *self;
		VkImageSubresourceRange
		{
			aspectMask: aspect,
			baseMipLevel: base_mip, levelCount: level_count,
			baseArrayLayer: base_array, layerCount: layer_count
		}
	}
}

#[derive(Clone, Copy)]
pub enum BufferDataType
{
	Vertex, Index, Uniform
}
pub struct BufferPreallocator
{
	usage_flags: VkBufferUsageFlags, offsets: Vec<usize>
}
pub trait BufferPreallocatorInternals
{
	fn new(usage: VkBufferUsageFlags, offsets: Vec<usize>) -> Self;
	fn get_usage(&self) -> VkBufferUsageFlags;
}
impl BufferPreallocatorInternals for BufferPreallocator
{
	fn new(usage: VkBufferUsageFlags, offsets: Vec<usize>) -> Self { BufferPreallocator { usage_flags: usage, offsets: offsets } }
	fn get_usage(&self) -> VkBufferUsageFlags { self.usage_flags }
}
impl BufferPreallocator
{
	pub fn offset(&self, index: usize) -> usize { self.offsets[index] }
	pub fn total_size(&self) -> VkDeviceSize { self.offsets.last().map(|&x| x).unwrap_or(0) as VkDeviceSize }
}

pub mod ImageUsagePresets
{
	use vkffi::*;
	
	pub const AsColorTexture: VkImageUsageFlags = VK_IMAGE_USAGE_SAMPLED_BIT | VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
}

pub struct ResourcePreallocator<'a>
{
	buffer: Option<&'a BufferPreallocator>,
	dim1_images: Vec<&'a ImageDescriptor1>,
	dim2_images: Vec<&'a ImageDescriptor2>,
	dim3_images: Vec<&'a ImageDescriptor3>
}
impl <'a> ResourcePreallocator<'a>
{
	pub fn new() -> Self
	{
		ResourcePreallocator { buffer: None, dim1_images: Vec::new(), dim2_images: Vec::new(), dim3_images: Vec::new() }
	}
	pub fn buffer(mut self, buf: &'a BufferPreallocator) -> Self
	{
		self.buffer = Some(buf);
		self
	}
	pub fn image_1d(mut self, i1ds: Vec<&'a ImageDescriptor1>) -> Self
	{
		self.dim1_images = i1ds;
		self
	}
	pub fn image_2d(mut self, i2ds: Vec<&'a ImageDescriptor2>) -> Self
	{
		self.dim2_images = i2ds;
		self
	}
	pub fn image_3d(mut self, i3ds: Vec<&'a ImageDescriptor3>) -> Self
	{
		self.dim3_images = i3ds;
		self
	}
}
pub trait ResourcePreallocatorInternals<'a>
{
	fn buffer(&self) -> &Option<&'a BufferPreallocator>;
	fn dim1_images(&self) -> &[&'a ImageDescriptor1];
	fn dim2_images(&self) -> &[&'a ImageDescriptor2];
	fn dim3_images(&self) -> &[&'a ImageDescriptor3];
}
impl <'a> ResourcePreallocatorInternals<'a> for ResourcePreallocator<'a>
{
	fn buffer(&self) -> &Option<&'a BufferPreallocator> { &self.buffer }
	fn dim1_images(&self) -> &[&'a ImageDescriptor1] { &self.dim1_images }
	fn dim2_images(&self) -> &[&'a ImageDescriptor2] { &self.dim2_images }
	fn dim3_images(&self) -> &[&'a ImageDescriptor3] { &self.dim3_images }
}

pub struct DoubleBufferedMemory { device: vk::DeviceMemory, host: vk::DeviceMemory }
pub trait DoubleBufferedMemoryInternals where Self: std::marker::Sized
{
	fn new(engine: &Engine, bound_resources: &[&Resource]) -> Result<Self, EngineError>;
}
impl DoubleBufferedMemoryInternals for DoubleBufferedMemory
{
	fn new(engine: &Engine, bound_resources: &[&Resource]) -> Result<Self, EngineError>
	{
		let requirements = bound_resources.into_iter().map(|&res| res.get_memory_requirements());
		let offsets = requirements.chain([VkMemoryRequirements
		{
			size: 0, alignment: 0, memoryTypeBits: 0
		}].into_iter().map(|&x| x)).scan(0usize, |offs, req|
		{
			let ret_offset = ((*offs as f64 / req.alignment as f64).ceil() as VkDeviceSize) * req.alignment;
			*offs += (ret_offset + req.size) as usize;
			Some(ret_offset)
		}).collect::<Vec<_>>();
		// let max_alignment = requirements.fold(1usize, num::integer::lcm);
		vk::DeviceMemory::alloc(engine.get_device().get_internal(), &VkMemoryAllocateInfo
		{
			sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
			allocationSize: offsets.last().map(|&x| x).unwrap_or(0),
			memoryTypeIndex: engine.get_memory_type_index_for_device_local()
		}).and_then(|dev| vk::DeviceMemory::alloc(engine.get_device().get_internal(), &VkMemoryAllocateInfo
			{
				sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
				allocationSize: offsets.last().map(|&x| x).unwrap_or(0),
				memoryTypeIndex: engine.get_memory_type_index_for_host_visible()
			}).map(move |host| DoubleBufferedMemory { device: dev, host: host })).map_err(EngineError::from)
	}
}

pub struct DeviceResource
{
	buffer: Option<Buffer>, dim1_images: Vec<Image1D>, dim2_images: Vec<Image2D>, dim3_images: Vec<Image3D>,
	memory: vk::DeviceMemory, size: VkDeviceSize
}
pub trait DeviceResourceInternals : std::marker::Sized
{
	fn new(engine: &Engine, buffer: Option<Buffer>, d1_images: Vec<Image1D>, d2_images: Vec<Image2D>, d3_images: Vec<Image3D>)
		-> Result<Self, EngineError>;
}
impl DeviceResourceInternals for DeviceResource
{
	fn new(engine: &Engine, buffer: Option<Buffer>, d1_images: Vec<Image1D>, d2_images: Vec<Image2D>, d3_images: Vec<Image3D>)
		-> Result<Self, EngineError>
	{
		let buffer_size =
		{
			let buffer_requirements = buffer.as_ref().map(|b| b.get_memory_requirements());
			buffer_requirements.map(|x| x.size).unwrap_or(0)
		};
		let image_offsets = {
			let d1_image_requirements = d1_images.iter().map(|b| b.get_memory_requirements());
			let d2_image_requirements = d2_images.iter().map(|b| b.get_memory_requirements());
			let d3_image_requirements = d3_images.iter().map(|b| b.get_memory_requirements());
			
			d1_image_requirements.chain(d2_image_requirements).chain(d3_image_requirements)
				.chain([VkMemoryRequirements { size: 0, alignment: 1, memoryTypeBits: 0 }].into_iter().map(|&x| x)).scan(buffer_size, |offs, req|
				{
					let current_offs = ((*offs as f64 / req.alignment as f64).ceil() as VkDeviceSize) * req.alignment;
					*offs = current_offs + req.size;
					Some(current_offs)
				}).collect::<Vec<_>>()
		};
		let memory_size = image_offsets.last().map(|&x| x).unwrap_or(buffer_size);
		info!(target: "Prelude::Resource", "Going to allocate buffer for device {} bytes", memory_size);

		vk::DeviceMemory::alloc(engine.get_device().get_internal(), &VkMemoryAllocateInfo
		{
			sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
			allocationSize: memory_size, memoryTypeIndex: engine.get_memory_type_index_for_device_local()
		}).and_then(move |memory|
			{
				if let Some(ref buf) = buffer { memory.bind_buffer(&buf.internal, 0) } else { Ok(()) }
			}.and_then(|()|
			{
				let image_resources = d1_images.iter().map(|i| i as &InternalExports<vk::Image>)
					.chain(d2_images.iter().map(|i| i as &InternalExports<vk::Image>))
					.chain(d3_images.iter().map(|i| i as &InternalExports<vk::Image>));
				
				for (&offs, res) in image_offsets.iter().zip(image_resources)
				{
					try!(memory.bind_image(res.get_internal(), offs));
				}
				Ok(())
			}).map(move |()| DeviceResource
			{
				buffer: buffer, dim1_images: d1_images, dim2_images: d2_images, dim3_images: d3_images,
				memory: memory, size: memory_size
			})
		).map_err(EngineError::from)
	}
}
impl DeviceResource
{
	pub fn buffer(&self) -> Result<&Buffer, EngineError>
	{
		self.buffer.as_ref().ok_or(EngineError::GenericError("DeviceResource has no buffers"))
	}
}

pub struct StagingResource
{
	buffer: Option<Buffer>, linear_dim2_images: Vec<LinearImage2D>,
	memory: vk::DeviceMemory, size: VkDeviceSize
}
pub trait StagingResourceInternals : std::marker::Sized
{
	fn new(engine: &Engine, buffer: Option<Buffer>, ld2_images: Vec<LinearImage2D>)
		-> Result<Self, EngineError>;
}
impl StagingResourceInternals for StagingResource
{
	fn new(engine: &Engine, buffer: Option<Buffer>, ld2_images: Vec<LinearImage2D>)
		-> Result<Self, EngineError>
	{
		let buffer_requirements = buffer.as_ref().map(|b| b.get_memory_requirements());
		let buffer_size = buffer_requirements.map(|x| x.size).unwrap_or(0);
		let image_offsets =
		{
			let ld2_image_requirements = ld2_images.iter().map(|b| b.get_memory_requirements());

			ld2_image_requirements.chain([VkMemoryRequirements { size: 0, alignment: 1, memoryTypeBits: 0 }].into_iter().map(|&x| x))
				.scan(buffer_size, |offs, req|
				{
					let current_offs = ((*offs as f64 / req.alignment as f64).ceil() as VkDeviceSize) * req.alignment;
					*offs = current_offs + req.size;
					Some(current_offs)
				}).collect::<Vec<_>>()
		};
		let memory_size = image_offsets.last().map(|&x| x).unwrap_or(buffer_size);
		info!(target: "Prelude::Resource", "Going to allocate buffer for host {} bytes", memory_size);

		vk::DeviceMemory::alloc(engine.get_device().get_internal(), &VkMemoryAllocateInfo
		{
			sType: VkStructureType::MemoryAllocateInfo, pNext: std::ptr::null(),
			allocationSize: memory_size, memoryTypeIndex: engine.get_memory_type_index_for_host_visible()
		}).and_then(|memory|
			{
				if let Some(ref buf) = buffer { memory.bind_buffer(&buf.internal, 0) } else { Ok(()) }
			}.and_then(|()|
			{
				for (&offs, res) in image_offsets.iter().zip(ld2_images.iter())
				{
					try!(memory.bind_image(res.get_internal(), offs));
				}
				Ok(())
			}).map(move |()| StagingResource
			{
				buffer: buffer, linear_dim2_images: ld2_images, memory: memory, size: memory_size
			})
		).map_err(EngineError::from)
	}
}
impl StagingResource
{
	pub fn map(&self) -> Result<MemoryMappedRange, EngineError>
	{
		self.memory.map(0 .. self.size).map(|ptr| MemoryMappedRange { parent: self, ptr: ptr }).map_err(EngineError::from)
	}
	pub fn buffer(&self) -> Result<&Buffer, EngineError>
	{
		self.buffer.as_ref().ok_or(EngineError::GenericError("StagingResource has no buffers"))
	}
}

pub struct MemoryMappedRange<'a>
{
	parent: &'a StagingResource, ptr: *mut c_void
}
impl <'a> MemoryMappedRange<'a>
{
	pub fn map_mut<MappedStructureT>(&self, offset: usize) -> &mut MappedStructureT
	{
		let t: &mut MappedStructureT = unsafe { std::mem::transmute(std::mem::transmute::<_, usize>(self.ptr) + offset) };
		t
	}
}
impl <'a> std::ops::Drop for MemoryMappedRange<'a>
{
	fn drop(&mut self) { self.parent.memory.unmap(); }
}
