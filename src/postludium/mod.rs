// Postludum: High-configurable Game Engine layered on interlude

use std;
use interlude;
use interlude::ffi::*;
use interlude::traits::*;
use std::rc::Rc;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use itertools::Itertools;
use std::collections::HashMap;

pub mod asm;
pub use self::asm::*;

pub trait LazyLines
{
	fn next(&mut self) -> Option<&(usize, String)>;
	fn pop(&mut self) -> Option<(usize, String)>;
}
#[allow(dead_code)]
pub struct LazyLinesStr<'a>
{
	iter: std::iter::Enumerate<std::str::Lines<'a>>, cache: Option<(usize, String)>
}
impl<'a> LazyLinesStr<'a>
{
	#[allow(dead_code)]
	pub fn new(source: &'a String) -> Self
	{
		LazyLinesStr { iter: source.lines().enumerate(), cache: None }
	}
}
impl<'a> LazyLines for LazyLinesStr<'a>
{
	fn next(&mut self) -> Option<&(usize, String)>
	{
		if self.cache.is_none() { self.cache = self.iter.next().map(|(u, s)| (u + 1, s.to_owned())); }
		self.cache.as_ref()
	}
	fn pop(&mut self) -> Option<(usize, String)>
	{
		if self.cache.is_none() { self.iter.next().map(|(u, s)| (u + 1, s.to_owned())) }
		else { std::mem::replace(&mut self.cache, None) }
	}
}
pub struct LazyLinesBR
{
	iter: std::iter::Enumerate<std::io::Lines<BufReader<File>>>, cache: Option<(usize, String)>
}
impl LazyLinesBR
{
	fn new(reader: BufReader<File>) -> Self { LazyLinesBR { iter: reader.lines().enumerate(), cache: None } }
}
impl LazyLines for LazyLinesBR
{
	fn next(&mut self) -> Option<&(usize, String)>
	{
		if self.cache.is_none() { self.cache = self.iter.next().map(|(u, s)| (u + 1, s.unwrap())); }
		self.cache.as_ref()
	}
	fn pop(&mut self) -> Option<(usize, String)>
	{
		if self.cache.is_none() { self.iter.next().map(|(u, s)| (u + 1, s.unwrap())) }
		else { std::mem::replace(&mut self.cache, None) }
	}
}

#[derive(Clone)]
pub enum DevConfParsingResult<T: Clone>
{
	Ok(T), NumericParseError(std::num::ParseIntError),
	InvalidFormatError, InvalidUsageFlagError(String), InvalidFilterError(String),
	UnsupportedDimension, UnsupportedParameter(String), InvalidSwizzle
}
impl<T: Clone> DevConfParsingResult<T>
{
	#[cfg(test)]
	pub fn unwrap(self) -> T
	{
		match self
		{
			DevConfParsingResult::Ok(t) => t,
			DevConfParsingResult::NumericParseError(e) => panic!(e),
			DevConfParsingResult::InvalidFormatError => panic!("Invalid Image Format"),
			DevConfParsingResult::InvalidSwizzle => panic!("Invalid Swizzle"),
			DevConfParsingResult::InvalidUsageFlagError(s) => panic!("Invalid Usage Flag: {}", s),
			DevConfParsingResult::InvalidFilterError(s) => panic!("Invalid Filter Type: {}", s),
			DevConfParsingResult::UnsupportedDimension => panic!("Unsupported Image Dimension"),
			DevConfParsingResult::UnsupportedParameter(dep) => panic!("Unsupported Parameter for {}", dep)
		}
	}
	pub fn unwrap_on_line(self, line: usize) -> T
	{
		match self
		{
			DevConfParsingResult::Ok(t) => t,
			DevConfParsingResult::NumericParseError(e) => panic!("{} at line {}", e, line),
			DevConfParsingResult::InvalidFormatError => panic!("Invalid Image Format at line {}", line),
			DevConfParsingResult::InvalidSwizzle => panic!("Invalid Swizzle at line {}", line),
			DevConfParsingResult::InvalidUsageFlagError(s) => panic!("Invalid Usage Flag: {} at line {}", s, line),
			DevConfParsingResult::InvalidFilterError(s) => panic!("Invalid Filter Type: {} at line {}", s, line),
			DevConfParsingResult::UnsupportedDimension => panic!("Unsupported Image Dimension at line {}", line),
			DevConfParsingResult::UnsupportedParameter(dep) => panic!("Unsupported Parameter for {} at line {}", dep, line)
		}
	}
	#[cfg(test)]
	pub fn is_invalid_format_err(&self) -> bool
	{
		match self { &DevConfParsingResult::InvalidFormatError => true, _ => false }
	}
	#[cfg(test)]
	pub fn is_invalid_usage_flag_err(&self) -> bool
	{
		match self { &DevConfParsingResult::InvalidUsageFlagError(_) => true, _ => false }
	}
	#[cfg(test)]
	pub fn is_numeric_parsing_failed(&self) -> bool
	{
		match self { &DevConfParsingResult::NumericParseError(_) => true, _ => false }
	}
	#[cfg(test)]
	pub fn is_invalid_filter_type_err(&self) -> bool
	{
		match self { &DevConfParsingResult::InvalidFilterError(_) => true, _ => false }
	}
}
impl<T: Clone> std::convert::From<Result<T, std::num::ParseIntError>> for DevConfParsingResult<T>
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
			"R8G8" => match element_type.as_ref()
			{
				"UNORM" => DevConfParsingResult::Ok(VkFormat::R8G8_UNORM),
				_ => DevConfParsingResult::InvalidFormatError
			},
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
			"BlockCompression4" => match element_type.as_ref()
			{
				"UNORM" => DevConfParsingResult::Ok(VkFormat::BC4_UNORM_BLOCK),
				_ => DevConfParsingResult::InvalidFormatError
			},
			"BlockCompression5" => match element_type.as_ref()
			{
				"UNORM" => DevConfParsingResult::Ok(VkFormat::BC5_UNORM_BLOCK),
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
			"Sampled" => Ok((VK_IMAGE_USAGE_SAMPLED_BIT, false)),
			"ColorAttachment" => Ok((VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT, false)),
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
		"$ScreenWidth" => Ok(screen_size.0), "$ScreenHeight" => Ok(screen_size.1), _ => input.parse()
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
pub fn parse_filter_type(args: &[char]) -> DevConfParsingResult<(interlude::Filter, interlude::Filter)>
{
	let (str, rest) = take_while(args, not_ignored);
	let str = str.into_iter().cloned().collect::<String>();
	let min_str = take_while(skip_spaces(rest), not_ignored).0.into_iter().cloned().collect::<String>();
	let mag_filter = match str.as_ref()
	{
		"Nearest" => DevConfParsingResult::Ok(interlude::Filter::Nearest),
		"Linear" => DevConfParsingResult::Ok(interlude::Filter::Linear),
		_ => DevConfParsingResult::InvalidFilterError(str)
	};
	let min_filter = if min_str.is_empty() { mag_filter.clone() } else
	{
		match min_str.as_ref()
		{
			"Nearest" => DevConfParsingResult::Ok(interlude::Filter::Nearest),
			"Linear" => DevConfParsingResult::Ok(interlude::Filter::Linear),
			_ => DevConfParsingResult::InvalidFilterError(min_str)
		}
	};
	match mag_filter
	{
		DevConfParsingResult::Ok(mag) => match min_filter
		{
			DevConfParsingResult::Ok(min) => DevConfParsingResult::Ok((mag, min)),
			DevConfParsingResult::InvalidFilterError(e) => DevConfParsingResult::InvalidFilterError(e),
			_ => unreachable!()
		},
		DevConfParsingResult::InvalidFilterError(e) => DevConfParsingResult::InvalidFilterError(e),
		_ => unreachable!()
	}
}
pub fn parse_component_map(args: &[char]) -> DevConfParsingResult<interlude::ComponentMapping>
{
	fn char_to_swizzle(ch: char) -> Result<interlude::ComponentSwizzle, ()>
	{
		match ch
		{
			'R' | 'r' => Ok(interlude::ComponentSwizzle::R),
			'G' | 'g' => Ok(interlude::ComponentSwizzle::G),
			'B' | 'b' => Ok(interlude::ComponentSwizzle::B),
			'A' | 'a' => Ok(interlude::ComponentSwizzle::A),
			_ => Err(())
		}
	}

	match char_to_swizzle(args[0]).and_then(|r|
		char_to_swizzle(args[1]).and_then(move |g|
		char_to_swizzle(args[2]).and_then(move |b|
		char_to_swizzle(args[3]).map(move |a| interlude::ComponentMapping(r, g, b, a)))))
	{
		Ok(cm) => DevConfParsingResult::Ok(cm),
		_ => DevConfParsingResult::InvalidSwizzle
	}
}
pub fn parse_configuration_image<LinesT: LazyLines>(lines_iter: &mut LinesT, screen_size: VkExtent2D, screen_format: VkFormat) -> DevConfImage
{
	let (headline, dim) =
	{
		let (headline, ref conf_head) = lines_iter.pop().unwrap();
		assert!(conf_head.starts_with("Image"));
		let dim_str = conf_head.chars().skip(5).skip_while(is_ignored).take_while(not_ignored).collect::<String>();
		let dim = match dim_str.as_ref()
		{
			"1D" => ImageDimensions::Single, "2D" => ImageDimensions::Double, "3D" => ImageDimensions::Triple, _ => DevConfParsingResult::UnsupportedDimension.unwrap_on_line(headline)
		};

		(headline, dim)
	};
	
	let (mut format, mut extent, mut usage, mut component_map) = (None, None, None, interlude::ComponentMapping::straight());
	while let Some((paramline, ref param)) =
	{
		let pop_next = if let Some(&(_, ref param)) = lines_iter.next() { if param.starts_with("-") { true } else { false } } else { false };
		if pop_next { lines_iter.pop() } else { None }
	}
	{
		let param_line = param.chars().skip(1).skip_while(is_ignored).collect::<Vec<_>>();
		let (param_name, rest) = take_while(&param_line, |c| not_ignored(c) && *c != ':');
		let param_value = skip_spaces(&skip_spaces(rest)[1..]);
		match param_name.into_iter().cloned().collect::<String>().as_ref()
		{
			"Format" => format = Some(parse_image_format(param_value, screen_format).unwrap_on_line(paramline)),
			"Extent" => extent = Some(parse_image_extent(param_value, dim, screen_size).unwrap_on_line(paramline)),
			"Usage" => usage = Some(parse_image_usage_flags(param_value, 0, false).unwrap_on_line(paramline)),
			"ComponentMap" => component_map = parse_component_map(param_value).unwrap_on_line(paramline),
			_ => DevConfParsingResult::UnsupportedParameter("Image".to_owned()).unwrap_on_line(paramline)
		}
	}
	let (usage, devlocal) = usage.expect(&format!("Usage parameter is not presented at line {}", headline));
	match dim
	{
		ImageDimensions::Single => DevConfImage::Dim1
		{
			format: format.expect(&format!("Format parameter is not presented at line {}", headline)),
			extent: extent.expect(&format!("Extent parameter is not presented at line {}", headline)).0,
			usage: usage, device_local: devlocal, component_map: component_map
		},
		ImageDimensions::Double => DevConfImage::Dim2
		{
			format: format.expect(&format!("Format parameter is not presented at line {}", headline)),
			extent: VkExtent2D::from(extent.expect(&format!("Extent parameter is not presented at line {}", headline))),
			usage: usage, device_local: devlocal, component_map: component_map
		},
		ImageDimensions::Triple => DevConfImage::Dim3
		{
			format: format.expect(&format!("Format parameter is not presented at line {}", headline)),
			extent: extent.expect(&format!("Extent parameter is not presented at line {}", headline)),
			usage: usage, device_local: devlocal, component_map: component_map
		}
	}
}
pub fn parse_configuration_sampler<LinesT: LazyLines>(lines_iter: &mut LinesT) -> DevConfSampler
{
	{
		let (_, ref conf_head) = lines_iter.pop().unwrap();
		assert!(conf_head.starts_with("Sampler"));
	}
	
	let (mut mag_filter, mut min_filter) = (interlude::Filter::Linear, interlude::Filter::Linear);
	while let Some((paramline, ref param)) =
	{
		let pop_next = if let Some(&(_, ref param)) = lines_iter.next() { if param.starts_with("-") { true } else { false } } else { false };
		if pop_next { lines_iter.pop() } else { None }
	}
	{
		let param_line = param.chars().skip(1).skip_while(is_ignored).collect::<Vec<_>>();
		let (param_name, rest) = take_while(&param_line, |c| not_ignored(c) && *c != ':');
		let param_value = skip_spaces(&skip_spaces(rest)[1..]);
		match param_name.into_iter().cloned().collect::<String>().as_ref()
		{
			"Filter" =>
			{
				let (magf, minf) = parse_filter_type(param_value).unwrap_on_line(paramline);
				mag_filter = magf; min_filter = minf;
			},
			_ => DevConfParsingResult::UnsupportedParameter("Sampler".to_owned()).unwrap_on_line(paramline)
		}
	}

	DevConfSampler
	{
		mag_filter: mag_filter, min_filter: min_filter
	}
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
		assert_eq!(parse_image_usage_flags(&"Sampled / DeviceLocal".chars().collect::<Vec<_>>(), 0, false).unwrap(), (VK_IMAGE_USAGE_SAMPLED_BIT, true));
		assert_eq!(parse_image_usage_flags(&"Sampled /DeviceLocal".chars().collect::<Vec<_>>(), 0, false).unwrap(), (VK_IMAGE_USAGE_SAMPLED_BIT, true));
		assert_eq!(parse_image_usage_flags(&"Sampled".chars().collect::<Vec<_>>(), 0, false).unwrap(), (VK_IMAGE_USAGE_SAMPLED_BIT, false));
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
	#[test] fn parse_filter_types()
	{
		assert_eq!(parse_filter_type(&"Nearest ".chars().collect::<Vec<_>>()).unwrap(), (interlude::Filter::Nearest, interlude::Filter::Nearest));
		assert_eq!(parse_filter_type(&"Linear".chars().collect::<Vec<_>>()).unwrap(), (interlude::Filter::Linear, interlude::Filter::Linear));
		assert_eq!(parse_filter_type(&"Linear Nearest".chars().collect::<Vec<_>>()).unwrap(), (interlude::Filter::Linear, interlude::Filter::Nearest));
		assert!(parse_filter_type(&"Bilinear".chars().collect::<Vec<_>>()).is_invalid_filter_type_err());
	}
	#[test] fn parse_image_conf()
	{
		let testcase = "Image 2D\n- Format: R8G8B8A8 UNORM\n- Extent: $ScreenWidth $ScreenHeight\n- Usage: Sampled / ColorAttachment\nImage 2D".to_owned();
		let mut testcase_wrap = LazyLinesStr::new(&testcase);
		let img = parse_configuration_image(&mut testcase_wrap, VkExtent2D(640, 480), VkFormat::R8G8B8A8_UNORM);
		match img
		{
			DevConfImage::Dim2 { format, extent, usage, device_local, .. } =>
			{
				assert_eq!(format, VkFormat::R8G8B8A8_UNORM);
				assert_eq!(extent, VkExtent2D(640, 480));
				assert_eq!(usage, VK_IMAGE_USAGE_SAMPLED_BIT | VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT);
				assert_eq!(device_local, false);
			},
			_ => unreachable!()
		}
		assert_eq!(testcase_wrap.next(), Some(&(5, "Image 2D".to_owned())));
	}
	#[test] fn parse_sampler_conf()
	{
		let testcase = "Sampler\n- Filter: Linear".to_owned();
		let mut testcase_wrap = LazyLinesStr::new(&testcase);
		let smp = parse_configuration_sampler(&mut testcase_wrap);
		assert_eq!(smp.mag_filter, interlude::Filter::Linear);
	}
}

enum NextInstruction
{
	Comment, Empty, Image, Sampler
}

#[derive(Debug)]
pub enum DevConfImage
{
	Dim1 { format: VkFormat, extent: u32, usage: VkImageUsageFlags, device_local: bool, component_map: interlude::ComponentMapping },
	Dim2 { format: VkFormat, extent: VkExtent2D, usage: VkImageUsageFlags, device_local: bool, component_map: interlude::ComponentMapping },
	Dim3 { format: VkFormat, extent: VkExtent3D, usage: VkImageUsageFlags, device_local: bool, component_map: interlude::ComponentMapping }
}
#[derive(Debug)]
pub struct DevConfSampler { mag_filter: interlude::Filter, min_filter: interlude::Filter }
pub struct DevConfImages
{
	image_views_1d: Vec<interlude::ImageView1D>, image_views_2d: Vec<interlude::ImageView2D>, image_views_3d: Vec<interlude::ImageView3D>,
	samplers: Vec<interlude::Sampler>,
	device_images: interlude::DeviceImage, staging_images: Option<interlude::StagingImage>
}
pub struct DevConfImagesWithStaging
{
	image_views_1d: Vec<interlude::ImageView1D>, image_views_2d: Vec<interlude::ImageView2D>, image_views_3d: Vec<interlude::ImageView3D>,
	samplers: Vec<interlude::Sampler>,
	device_images: interlude::DeviceImage, staging_images: interlude::StagingImage
}
impl DevConfImages
{
	pub fn from_file(engine: &interlude::Engine, asset_path: &str, screen_size: VkExtent2D, screen_format: VkFormat) -> Self
	{
		let path = engine.parse_asset(asset_path, "pdc");
		info!(target: "Postludium", "Parsing Device Configuration {:?}...", path);
		let mut flines = LazyLinesBR::new(BufReader::new(File::open(path).unwrap()));

		let (mut images, mut samplers) = (Vec::new(), Vec::new());
		while let Some(next) = flines.next().map(|&(headline, ref line)|
			if line.is_empty() { NextInstruction::Empty } else if line.starts_with("#") { NextInstruction::Comment }
			else if line.starts_with("Image") { NextInstruction::Image } else if line.starts_with("Sampler") { NextInstruction::Sampler }
			else { panic!("Unknown Configuration at line {}", headline) })
		{
			match next
			{
				NextInstruction::Empty | NextInstruction::Comment => { flines.pop(); },
				NextInstruction::Image =>
				{
					let obj = parse_configuration_image(&mut flines, screen_size, screen_format);
					println!("Found {:?}", obj);
					images.push(obj);
				},
				NextInstruction::Sampler =>
				{
					let obj = parse_configuration_sampler(&mut flines);
					println!("Found {:?}", obj);
					samplers.push(obj);
				}
			}
		}

		// FIXME: Device Local flags for Image 3D
		let mut image_descriptors1 = HashMap::new();
		let mut image_descriptors2 = HashMap::new();
		let mut image_descriptors3 = HashMap::new();
		for img in &images
		{
			match img
			{
				&DevConfImage::Dim1 { format, extent, usage, device_local: true, .. } => { image_descriptors1.entry((format, extent, usage, true))
					.or_insert(interlude::ImageDescriptor1::new(format, extent, usage).device_resource()); },
				&DevConfImage::Dim2 { format, extent, usage, device_local: true, .. } => { image_descriptors2.entry((format, extent, usage, true))
					.or_insert(interlude::ImageDescriptor2::new(format, extent, usage).device_resource()); },
				&DevConfImage::Dim1 { format, extent, usage, device_local: false, .. } => { image_descriptors1.entry((format, extent, usage, false))
					.or_insert(interlude::ImageDescriptor1::new(format, extent, usage)); },
				&DevConfImage::Dim2 { format, extent, usage, device_local: false, .. } => { image_descriptors2.entry((format, extent, usage, false))
					.or_insert(interlude::ImageDescriptor2::new(format, extent, usage)); },
				&DevConfImage::Dim3 { format, extent, usage, .. } => { image_descriptors3.entry((format, extent, usage, false))
					.or_insert(interlude::ImageDescriptor3::new(format, extent, usage)); }
			}
		}

		let images_1d = images.iter().filter_map(|img| match img
		{
			&DevConfImage::Dim1 { format, extent, usage, device_local, component_map } => Some((format, extent, usage, device_local, component_map)),
			_ => None	
		}).collect_vec();
		let images_2d = images.iter().filter_map(|img| match img
		{
			&DevConfImage::Dim2 { format, extent, usage, device_local, component_map } => Some((format, extent, usage, device_local, component_map)),
			_ => None	
		}).collect_vec();
		let images_3d = images.iter().filter_map(|img| match img
		{
			&DevConfImage::Dim3 { format, extent, usage, device_local, component_map } => Some((format, extent, usage, device_local, component_map)),
			_ => None	
		}).collect_vec();

		let image_descriptor_refs_1d = images_1d.iter().map(|&(format, extent, usage, device_local, _)| image_descriptors1.get(&(format, extent, usage, device_local)).unwrap()).collect_vec();
		let image_descriptor_refs_2d = images_2d.iter().map(|&(format, extent, usage, device_local, _)| image_descriptors2.get(&(format, extent, usage, device_local)).unwrap()).collect_vec();
		let image_descriptor_refs_3d = images_3d.iter().map(|&(format, extent, usage, _, _)| image_descriptors3.get(&(format, extent, usage, false)).unwrap()).collect_vec();
		let image_prealloc_with_moving = interlude::ImagePreallocator::new().image_1d(image_descriptor_refs_1d).image_2d(image_descriptor_refs_2d).image_3d(image_descriptor_refs_3d);
		let (backbuffers, staging_images) = Unrecoverable!(engine.create_double_image(&image_prealloc_with_moving));
		let image_views_1d = images_1d.iter().enumerate().map(|(nr, &(format, _, _, _, component_map))| 
			Unrecoverable!(interlude::ImageView1D::new(engine, &backbuffers.dim1vec()[nr], format, component_map, interlude::ImageSubresourceRange::base_color()))).collect_vec();
		let image_views_2d = images_2d.iter().enumerate().map(|(nr, &(format, _, _, _, component_map))|
			Unrecoverable!(interlude::ImageView2D::new(engine, &backbuffers.dim2vec()[nr], format, component_map, interlude::ImageSubresourceRange::base_color()))).collect_vec();
		let image_views_3d = images_3d.iter().enumerate().map(|(nr, &(format, _, _, _, component_map))|
			Unrecoverable!(interlude::ImageView3D::new(engine, &backbuffers.dim3vec()[nr], format, component_map, interlude::ImageSubresourceRange::base_color()))).collect_vec();

		let sampler_objects = samplers.iter().map(|dcs|
		{
			let sampler_state = interlude::SamplerState::new().filters(dcs.mag_filter, dcs.min_filter);
			Unrecoverable!(engine.create_sampler(&sampler_state))
		}).collect_vec();

		DevConfImages
		{
			image_views_1d: image_views_1d, image_views_2d: image_views_2d, image_views_3d: image_views_3d,
			samplers: sampler_objects,
			device_images: backbuffers, staging_images: staging_images
		}
	}
	pub fn ensure_has_staging(self) -> DevConfImagesWithStaging
	{
		DevConfImagesWithStaging
		{
			image_views_1d: self.image_views_1d, image_views_2d: self.image_views_2d, image_views_3d: self.image_views_3d, samplers: self.samplers,
			device_images: self.device_images, staging_images: self.staging_images.unwrap()
		}
	}

	pub fn images_1d(&self) -> &Vec<interlude::ImageView1D> { &self.image_views_1d }
	pub fn images_2d(&self) -> &Vec<interlude::ImageView2D> { &self.image_views_2d }
	pub fn images_3d(&self) -> &Vec<interlude::ImageView3D> { &self.image_views_3d }
	pub fn samplers(&self) -> &Vec<interlude::Sampler> { &self.samplers }
}
impl DevConfImagesWithStaging
{
	pub fn images_1d(&self) -> &Vec<interlude::ImageView1D> { &self.image_views_1d }
	pub fn images_2d(&self) -> &Vec<interlude::ImageView2D> { &self.image_views_2d }
	pub fn images_3d(&self) -> &Vec<interlude::ImageView3D> { &self.image_views_3d }
	pub fn samplers(&self) -> &Vec<interlude::Sampler> { &self.samplers }

	pub fn staging_images(&self) -> &Vec<interlude::LinearImage2D> { self.staging_images.dim2vec() }
	pub fn map_staging_images_memory(&self) -> interlude::MemoryMappedRange { Unrecoverable!(self.staging_images.map()) }
	pub fn staging_offsets(&self) -> &Vec<VkDeviceSize> { self.staging_images.image2d_offsets() }
}
