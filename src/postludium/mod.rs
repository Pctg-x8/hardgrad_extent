// Postludum: High-configurable Game Engine layered on interlude

use std;
use interlude;
use interlude::ffi::*;
use interlude::traits::*;
use std::rc::Rc;
use std::path::Path;

pub enum DevConfParsingResult<T>
{
	Ok(T), IOError(std::io::Error), NumericParseError(std::num::ParseIntError),
	InvalidFormatError, InvalidUsageFlagError(String)
}
impl<T> DevConfParsingResult<T>
{
	pub fn unwrap(self) -> T
	{
		match self
		{
			DevConfParsingResult::Ok(t) => t,
			DevConfParsingResult::IOError(e) => panic!(e),
			DevConfParsingResult::NumericParseError(e) => panic!(e),
			DevConfParsingResult::InvalidFormatError => panic!("Invalid Image Format"),
			DevConfParsingResult::InvalidUsageFlagError(s) => panic!("Invalid Usage Flag: {}", s)
		}
	}
	pub fn is_invalid_format_err(&self) -> bool
	{
		match self { &DevConfParsingResult::InvalidFormatError => true, _ => false }
	}
	pub fn is_invalid_usage_flag_err(&self) -> bool
	{
		match self { &DevConfParsingResult::InvalidUsageFlagError(_) => true, _ => false }
	}
	pub fn is_numeric_parsing_failed(&self) -> bool
	{
		match self { &DevConfParsingResult::NumericParseError(_) => true, _ => false }
	}
}
impl<T> std::convert::From<Result<T, std::num::ParseIntError>> for DevConfParsingResult<T>
{
	fn from(r: Result<T, std::num::ParseIntError>) -> Self
	{
		match r { Ok(t) => DevConfParsingResult::Ok(t), Err(e) => DevConfParsingResult::NumericParseError(e) }
	}
}

#[derive(Clone, Copy)]
pub enum ImageDimensions { Single, Double, Triple }

fn is_ignored(c: &char) -> bool { *c == ' ' || *c == '\t' }
fn not_ignored(c: &char) -> bool { !is_ignored(c) }
pub fn skip_spaces(p: &[char]) -> &[char] { if !p.is_empty() && is_ignored(&p[0]) { skip_spaces(&p[1..]) } else { p } }
pub fn take_while<F: Fn(&char) -> bool>(p: &[char], pred: F) -> (&[char], &[char]) { let ptr = take_while_impl(p, 0, pred); (&p[..ptr], &p[ptr..]) }
fn take_while_impl<F: Fn(&char) -> bool>(data: &[char], ptr: usize, pred: F) -> usize
{
	if data.len() > ptr && pred(&data[ptr]) { take_while_impl(data, ptr + 1, pred) } else { ptr }
}

pub fn parse_image_format(args: &[char], screen_format: VkFormat) -> DevConfParsingResult<VkFormat>
{
	let (bit_arrange, rest) = take_while(args, not_ignored);
	let bit_arrange = bit_arrange.into_iter().cloned().collect::<String>();
	if bit_arrange == "$ScreenFormat" { DevConfParsingResult::Ok(screen_format) }
	else
	{
		let (element_type, _) = take_while(skip_spaces(rest), not_ignored);
		let element_type = element_type.into_iter().cloned().collect::<String>();
		match bit_arrange.as_ref()
		{
			"R8G8B8A8" => match element_type.as_ref()
			{
				"UNORM" => DevConfParsingResult::Ok(VkFormat::R8G8B8A8_UNORM),
				"SRGB" => DevConfParsingResult::Ok(VkFormat::R8G8B8A8_SRGB),
				_ => DevConfParsingResult::InvalidFormatError
			},
			"R16G16B16A16" => match element_type.as_ref()
			{
				"SFLOAT" => DevConfParsingResult::Ok(VkFormat::R16G16B16A16_SFLOAT),
				_ => DevConfParsingResult::InvalidFormatError
			},
			_ => DevConfParsingResult::InvalidFormatError
		}
	}
}
pub fn parse_image_usage_flags(args: &[char], agg_usage: VkImageUsageFlags, device_local_flag: bool) -> DevConfParsingResult<(VkImageUsageFlags, bool)>
{
	if args.is_empty() { DevConfParsingResult::InvalidUsageFlagError("".to_owned()) }
	else
	{
		let (usage_str, rest) = take_while(args, |c| not_ignored(c) && *c != '/');
		let usage_str = usage_str.into_iter().cloned().collect::<String>();
		let processing_rest = skip_spaces(rest);
		let has_next = !processing_rest.is_empty() && processing_rest[0] == '/';
		let current_parsed = match usage_str.as_ref()
		{
			"AsColorTexture" => Ok((interlude::ImageUsagePresets::AsColorTexture, false)),
			"Sampled" => Ok((VK_IMAGE_USAGE_SAMPLED_BIT, false)),
			"DeviceLocal" => Ok((0, true)),
			_ => Err(())
		};
		if let Ok((usage_bits, devlocal)) = current_parsed
		{
			if has_next
			{
				parse_image_usage_flags(skip_spaces(&processing_rest[1..]), agg_usage | usage_bits, device_local_flag | devlocal)
			}
			else { DevConfParsingResult::Ok((agg_usage | usage_bits, device_local_flag | devlocal)) }
		}
		else { DevConfParsingResult::InvalidUsageFlagError(usage_str) }
	}
}
pub fn parse_image_extent(args: &[char], dims: ImageDimensions, screen_size: VkExtent2D) -> DevConfParsingResult<VkExtent3D>
{
	let parse_arg = |input: &str| match input
	{
		"$ScreenWidth" => Ok(screen_size.0),
		"$ScreenHeight" => Ok(screen_size.1),
		_ => input.parse()
	};
	DevConfParsingResult::from(match dims
	{
		ImageDimensions::Single => parse_arg(&take_while(args, not_ignored).0.into_iter().cloned().collect::<String>()).map(|val| VkExtent3D(val, 1, 1)),
		ImageDimensions::Double =>
		{
			let (wstr, rest) = take_while(args, not_ignored);
			let (hstr, _) = take_while(skip_spaces(rest), not_ignored);

			parse_arg(&wstr.into_iter().cloned().collect::<String>()).and_then(|w| parse_arg(&hstr.into_iter().cloned().collect::<String>()).map(move |h| VkExtent3D(w, h, 1)))
		},
		ImageDimensions::Triple =>
		{
			let (wstr, rest) = take_while(args, not_ignored);
			let (hstr, rest) = take_while(skip_spaces(rest), not_ignored);
			let (dstr, _) = take_while(skip_spaces(rest), not_ignored);

			parse_arg(&wstr.into_iter().cloned().collect::<String>()).and_then(|w| parse_arg(&hstr.into_iter().cloned().collect::<String>()).and_then(move |h|
				parse_arg(&dstr.into_iter().cloned().collect::<String>()).map(move |d| VkExtent3D(w, h, d))))
		}
	})
}

#[cfg(test)]
mod test
{
	use interlude;
	use interlude::ffi::*;
	use super::*;

	#[test] fn parse_image_formats()
	{
		assert_eq!(parse_image_format(&"R8G8B8A8 UNORM".chars().collect::<Vec<_>>(), VkFormat::R8G8B8A8_SRGB).unwrap(), VkFormat::R8G8B8A8_UNORM);
		assert_eq!(parse_image_format(&"$ScreenFormat".chars().collect::<Vec<_>>(), VkFormat::R16G16B16A16_UNORM).unwrap(), VkFormat::R16G16B16A16_UNORM);
		assert_eq!(parse_image_format(&"$ScreenFormat SFLOAT".chars().collect::<Vec<_>>(), VkFormat::R16G16B16A16_UNORM).unwrap(), VkFormat::R16G16B16A16_UNORM);
		assert!(parse_image_format(&"R8G8B8A8 SFLOAT".chars().collect::<Vec<_>>(), VkFormat::R8G8B8A8_SRGB).is_invalid_format_err());
		assert!(parse_image_format(&"aaa TEST".chars().collect::<Vec<_>>(), VkFormat::R8G8B8A8_SRGB).is_invalid_format_err());
	}
	#[test] fn parse_image_usage()
	{
		assert_eq!(parse_image_usage_flags(&"AsColorTexture / DeviceLocal".chars().collect::<Vec<_>>(), 0, false).unwrap(), (interlude::ImageUsagePresets::AsColorTexture, true));
		assert_eq!(parse_image_usage_flags(&"AsColorTexture/DeviceLocal".chars().collect::<Vec<_>>(), 0, false).unwrap(), (interlude::ImageUsagePresets::AsColorTexture, true));
		assert_eq!(parse_image_usage_flags(&"AsColorTexture".chars().collect::<Vec<_>>(), 0, false).unwrap(), (interlude::ImageUsagePresets::AsColorTexture, false));
		assert!(parse_image_usage_flags(&"AsColorTexture/".chars().collect::<Vec<_>>(), 0, false).is_invalid_usage_flag_err());
	}
	#[test] fn parse_image_extents()
	{
		assert_eq!(parse_image_extent(&"640".chars().collect::<Vec<_>>(), ImageDimensions::Single, VkExtent2D(1920, 1080)).unwrap(), VkExtent3D(640, 1, 1));
		assert_eq!(parse_image_extent(&"640 480".chars().collect::<Vec<_>>(), ImageDimensions::Double, VkExtent2D(1920, 1080)).unwrap(), VkExtent3D(640, 480, 1));
		assert_eq!(parse_image_extent(&"640 480 16".chars().collect::<Vec<_>>(), ImageDimensions::Triple, VkExtent2D(1920, 1080)).unwrap(), VkExtent3D(640, 480, 16));
		assert_eq!(parse_image_extent(&"$ScreenWidth $ScreenHeight".chars().collect::<Vec<_>>(), ImageDimensions::Double, VkExtent2D(1920, 1080)).unwrap(), VkExtent3D(1920, 1080, 1));
		assert_eq!(parse_image_extent(&"640 $ScreenHeight".chars().collect::<Vec<_>>(), ImageDimensions::Double, VkExtent2D(1920, 1080)).unwrap(), VkExtent3D(640, 1080, 1));
		assert!(parse_image_extent(&"$screenwidth $screenHeight".chars().collect::<Vec<_>>(), ImageDimensions::Double, VkExtent2D(1920, 1080)).is_numeric_parsing_failed());
		assert!(parse_image_extent(&"$screenwidth aaa".chars().collect::<Vec<_>>(), ImageDimensions::Double, VkExtent2D(1920, 1080)).is_numeric_parsing_failed());
	}
}

enum DevConfImage
{
	Dim1 { format: VkFormat, extent: u32, usage: VkImageUsageFlags },
	Dim2 { format: VkFormat, extent: VkExtent2D, usage: VkImageUsageFlags },
	Dim3 { format: VkFormat, extent: VkExtent3D, usage: VkImageUsageFlags }
}
pub struct ImageViewPair(pub Rc<interlude::traits::ImageResource>, pub Box<interlude::traits::ImageView>);
pub struct DevConfImages
{
	image_views: Vec<ImageViewPair>, samplers: Vec<interlude::SamplerState>
}
impl DevConfImages
{
	pub fn from_file<PathT: AsRef<Path>>(path: PathT) -> Self
	{
		DevConfImages { image_views: Vec::new(), samplers: Vec::new() }
	}
}
