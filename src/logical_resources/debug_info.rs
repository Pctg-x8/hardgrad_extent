use render_vk::wrap as vk;
use vkffi::*;
use std;
use traits::*;
use freetype::*;
use unicode_normalization::*;
use device_resources;

pub struct DebugInfoResources<'d>
{
	#[allow(dead_code)] memory: vk::DeviceMemory<'d>, pub texture: vk::Image<'d>,
	pub texture_view: vk::ImageView<'d>, pub sampler: vk::Sampler<'d>,
	descriptor_index: u32, texture_info: VkDescriptorImageInfo
}
impl <'d> DebugInfoResources<'d>
{
	pub fn new(device: &'d vk::Device, transfer_queue: &'d vk::Queue, initializer_pool: &'d vk::CommandPool, descriptor_index: u32) -> Self
	{
		let texture_size = VkExtent2D(128, 128);
		let texture = device.create_single_image(texture_size, VkImageTiling::Optimal,
			VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT | VK_IMAGE_USAGE_SAMPLED_BIT).unwrap();
		let memory = device.allocate_memory_for_image(&texture, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
		memory.bind_image(&texture, 0).unwrap();
		let texture_view = device.create_image_view(&VkImageViewCreateInfo
		{
			sType: VkStructureType::ImageViewCreateInfo, pNext: std::ptr::null(), flags: 0,
			image: texture.get(), viewType: VkImageViewType::Dim2, format: VkFormat::R8_UNORM,
			components: VkComponentMapping { r: VkComponentSwizzle::R, g: VkComponentSwizzle::R, b: VkComponentSwizzle::R, a: VkComponentSwizzle::R },
			subresourceRange: VkImageSubresourceRange
			{
				aspectMask: VK_IMAGE_ASPECT_COLOR_BIT, baseArrayLayer: 0, baseMipLevel: 0,
				layerCount: 1, levelCount: 1
			}
		}).unwrap();
		let sampler = device.create_sampler(&VkSamplerCreateInfo
		{
			sType: VkStructureType::SamplerCreateInfo, pNext: std::ptr::null(), flags: 0,
			magFilter: VkFilter::Nearest, minFilter: VkFilter::Nearest, mipmapMode: VkSamplerMipmapMode::Nearest,
			addressModeU: VkSamplerAddressMode::ClampToBorder, addressModeV: VkSamplerAddressMode::ClampToBorder, addressModeW: VkSamplerAddressMode::ClampToBorder,
			mipLodBias: 0.0f32, anisotropyEnable: false as VkBool32, maxAnisotropy: 0.0f32,
			compareEnable: false as VkBool32, compareOp: VkCompareOp::Never, minLod: 0.0f32, maxLod: 1.0f32,
			borderColor: VkBorderColor::FloatTransparentBlack, unnormalizedCoordinates: false as VkBool32
		}).unwrap();

		// Transient Image/Memory for Staging
		let stage_texture = device.create_single_image(texture_size, VkImageTiling::Linear, VK_IMAGE_USAGE_TRANSFER_SRC_BIT).unwrap();
		let stage_memory = device.allocate_memory_for_image(&stage_texture, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT).unwrap();
		stage_memory.bind_image(&stage_texture, 0).unwrap();

		// render numeric characters
		{
			let mapped_range = stage_memory.map(0 .. 128 * 128).unwrap();
			let mut bitmap_range = mapped_range.range_mut::<u8>(0, 128 * 128);
			bitmap_range.clone_from_slice(&[0; 128 * 128]);

			// freetype //
			let ft_library = Library::init().unwrap();
			let ft_face = ft_library.new_face("resources/fonts/mplus-TESTFLIGHT-061/mplus-2c-regular.ttf", 0).unwrap();
			ft_face.set_char_size(0, 8 << 6, 0, 100).unwrap();

			// render numbers //
			let mut max_height = 0;
			let mut current_left = 0;
			for c in "0123456789.".nfc()
			{
				ft_face.load_char(c as usize, face::RENDER).unwrap();
				let glyph = ft_face.glyph();
				let bitmap = glyph.bitmap();
				// let (xo, yo) = (glyph.bitmap_left(), glyph.bitmap_top());
				let (width, height) = (bitmap.width(), bitmap.rows());
				let coordinate_list = (0 .. width).flat_map(|x| (0 .. height).map(move |y| (x, y)))
					.map(|(x, y)| (x, y, bitmap.buffer()[(x + y * bitmap.pitch()) as usize]));
				for (x, y, pixel) in coordinate_list
				{
					bitmap_range[((x + current_left) + (y + 1) * 128) as usize] += pixel;
				}
				current_left += width + 1;
				max_height = std::cmp::max(max_height, height);
			}

			// render fixed texts //
			let ft_size = Self::render_text(&ft_face, &mut bitmap_range, 0, max_height + 1, "Frame Time: ");
			let oec_size = Self::render_text(&ft_face, &mut bitmap_range, 0, max_height + 1 + ft_size.1 as i32 + 1, "Object[Enemy] Count: ");
		}

		// Initial Transferring //
		{
			let command_buffer = initializer_pool.allocate_primary_buffers(1).unwrap();
			let subres_range_color = VkImageSubresourceRange
			{
				aspectMask: VK_IMAGE_ASPECT_COLOR_BIT, baseMipLevel: 0, baseArrayLayer: 0,
				levelCount: 1, layerCount: 1
			};

			let image_barriers = [
				VkImageMemoryBarrier
				{
					sType: VkStructureType::ImageMemoryBarrier, pNext: std::ptr::null(),
					srcAccessMask: VK_ACCESS_HOST_WRITE_BIT, dstAccessMask: VK_ACCESS_TRANSFER_READ_BIT,
					oldLayout: VkImageLayout::Preinitialized, newLayout: VkImageLayout::TransferSrcOptimal,
					srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
					image: stage_texture.get(), subresourceRange: subres_range_color
				},
				VkImageMemoryBarrier
				{
					sType: VkStructureType::ImageMemoryBarrier, pNext: std::ptr::null(),
					srcAccessMask: VK_ACCESS_HOST_WRITE_BIT, dstAccessMask: VK_ACCESS_TRANSFER_WRITE_BIT,
					oldLayout: VkImageLayout::Preinitialized, newLayout: VkImageLayout::TransferDestOptimal,
					srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
					image: texture.get(), subresourceRange: subres_range_color
				}
			];
			let image_barriers_to_use = [
				VkImageMemoryBarrier
				{
					sType: VkStructureType::ImageMemoryBarrier, pNext: std::ptr::null(),
					srcAccessMask: VK_ACCESS_TRANSFER_READ_BIT, dstAccessMask: VK_ACCESS_HOST_WRITE_BIT,
					oldLayout: VkImageLayout::TransferSrcOptimal, newLayout: VkImageLayout::General,
					srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
					image: stage_texture.get(), subresourceRange: subres_range_color
				},
				VkImageMemoryBarrier
				{
					sType: VkStructureType::ImageMemoryBarrier, pNext: std::ptr::null(),
					srcAccessMask: VK_ACCESS_TRANSFER_WRITE_BIT, dstAccessMask: VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | VK_ACCESS_INPUT_ATTACHMENT_READ_BIT | VK_ACCESS_SHADER_READ_BIT,
					oldLayout: VkImageLayout::TransferDestOptimal, newLayout: VkImageLayout::ShaderReadOnlyOptimal,
					srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED, dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED,
					image: texture.get(), subresourceRange: subres_range_color
				}
			];

			let copy_region = VkImageCopy(VkImageSubresourceLayers(VK_IMAGE_ASPECT_COLOR_BIT, 0, 0, 1), VkOffset3D(0, 0, 0),
				VkImageSubresourceLayers(VK_IMAGE_ASPECT_COLOR_BIT, 0, 0, 1), VkOffset3D(0, 0, 0), VkExtent3D(128, 128, 1));
			command_buffer.begin(0).unwrap()
				.resource_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, &[], &[], &image_barriers)
				.copy_image(&stage_texture, VkImageLayout::TransferSrcOptimal, &texture, VkImageLayout::TransferDestOptimal, &[copy_region])
				.resource_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, &[], &[], &image_barriers_to_use);
			transfer_queue.submit_commands(&[command_buffer[0]], &[], &[], None).unwrap();
			transfer_queue.wait_for_idle().unwrap();
		}

		DebugInfoResources
		{
			texture_info: VkDescriptorImageInfo(sampler.get(), texture_view.get(), VkImageLayout::ShaderReadOnlyOptimal),
			memory: memory, texture: texture, texture_view: texture_view, sampler: sampler,
			descriptor_index: descriptor_index
		}
	}

	fn render_text(face: &Face, bitmap_range: &mut [u8], left: i32, top: i32, text: &str) -> VkExtent2D
	{
		let ft_baseline = face.ascender() as i32 * face.size_metrics().unwrap().y_ppem as i32 / face.em_size() as i32;

		let mut left_accum = left;
		let mut prev_char: Option<char> = None;
		let mut max_height = ft_baseline;
		for c in text.nfc()
		{
			face.load_char(c as usize,face::RENDER).unwrap();
			let (kern_left, kern_top) = if let Some(pc) = prev_char
			{
				face.get_kerning(pc as u32, c as u32, face::KerningMode::KerningDefault)
					.map(|Vector { x, y }| (x as i32 >> 6, y as i32 >> 6)).unwrap()
			}
			else
			{
				(0, 0)
			};
			let glyph = face.glyph();
			let bitmap = glyph.bitmap();
			let (xo, yo) = (glyph.bitmap_left(), glyph.bitmap_top());
			let (width, height) = (bitmap.width(), bitmap.rows());
			let coordinate_list = (0 .. width).flat_map(|x| (0 .. height)
				.map(move |y| (x, y))).map(|(x, y)| (x, y, bitmap.buffer()[(x + y * bitmap.pitch()) as usize]));
			let left_offset = left_accum + xo + kern_left;
			let top_offset = top + (ft_baseline - yo) + kern_top;
			for (x, y, pixel) in coordinate_list
			{
				bitmap_range[((x + left_offset) + (y + top_offset) * 128) as usize] += pixel;
			}
			left_accum += glyph.advance().x as i32 >> 6;
			prev_char = Some(c);
			max_height = std::cmp::max(max_height, height);
		}

		VkExtent2D((left_accum - left) as u32, max_height as u32)
	}
}

impl <'d> HasDescriptor for DebugInfoResources<'d>
{
	fn write_descriptor_info(&self, desc_set: &device_resources::DescriptorSets) -> VkWriteDescriptorSet
	{
		VkWriteDescriptorSet
		{
			sType: VkStructureType::WriteDescriptorSet, pNext: std::ptr::null(),
			dstSet: desc_set.sets[self.descriptor_index as usize], dstBinding: 0, dstArrayElement: 0,
			descriptorType: VkDescriptorType::CombinedImageSampler, descriptorCount: 1,
			pBufferInfo: std::ptr::null(), pImageInfo: &self.texture_info, pTexelBufferView: std::ptr::null()
		}
	}
}