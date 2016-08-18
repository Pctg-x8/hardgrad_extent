// Prelude: Resources(Buffer and Image)

use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;

pub trait ImageResource { fn get_resource(&self) -> VkImage; }

pub struct Buffer { pub internal: vk::Buffer }
pub struct Image2D { pub internal: vk::Image }
impl ImageResource for Image2D { fn get_resource(&self) -> VkImage { self.internal.get() } }
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
