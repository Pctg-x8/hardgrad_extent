use std::path::*;

pub mod pascalstring;
pub use pascalstring as PascalString;

pub mod colormode;
pub use colormode::*;
pub mod imageresource;
pub use imageresource::*;
pub mod layer;
pub use layer::*;
pub mod imagedata;
pub use imagedata::*;

// Common Errors occured in loading
#[derive(Debug)]
pub enum PSDLoadingError
{
	IOError(std::io::Error), EncodingError(std::string::FromUtf8Error),
	SignatureMismatching(&'static str), VersionMismatching,
	StructureSizeMismatching
}
impl std::convert::From<std::io::Error> for PSDLoadingError
{
	fn from(v: std::io::Error) -> PSDLoadingError { PSDLoadingError::IOError(v) }
}
impl std::convert::From<std::string::FromUtf8Error> for PSDLoadingError
{
	fn from(v: std::string::FromUtf8Error) -> PSDLoadingError { PSDLoadingError::EncodingError(v) } 
}

// Helper functions for reading Integer values
trait BinaryLoaderUtils : std::io::prelude::Read
{
	fn read_u8(&mut self) -> std::io::Result<u8>;
	fn read_u16(&mut self) -> std::io::Result<u16>;
	fn read_i16(&mut self) -> std::io::Result<i16>;
	fn read_u32(&mut self) -> std::io::Result<u32>;
	fn read_u32_be(&mut self) -> std::io::Result<u32>;
	fn read_f64(&mut self) -> std::io::Result<f64>;
	fn read_struct<T>(&mut self) -> std::io::Result<T>;
}
impl <T> BinaryLoaderUtils for T where T: std::io::prelude::Read
{
	fn read_u8(&mut self) -> std::io::Result<u8>
	{
		let mut bytes = [0u8];
		self.read_exact(&mut bytes).map(|()| bytes[0])
	}
	fn read_u16(&mut self) -> std::io::Result<u16>
	{
		let mut short_be = 0u16;
		let mut bytes = unsafe { std::slice::from_raw_parts_mut(std::mem::transmute::<_, *mut u8>(&mut short_be), std::mem::size_of::<u16>()) };
		self.read_exact(&mut bytes).map(|()| u16::from_be(short_be))
	}
	fn read_i16(&mut self) -> std::io::Result<i16>
	{
		let mut short_be = 0i16;
		let mut bytes = unsafe { std::slice::from_raw_parts_mut(std::mem::transmute::<_, *mut u8>(&mut short_be), std::mem::size_of::<u16>()) };
		self.read_exact(&mut bytes).map(|()| i16::from_be(short_be))
	}
	fn read_u32(&mut self) -> std::io::Result<u32>
	{
		let mut int_be = 0u32;
		let mut bytes = unsafe { std::slice::from_raw_parts_mut(std::mem::transmute::<_, *mut u8>(&mut int_be), std::mem::size_of::<u32>()) };
		self.read_exact(&mut bytes).map(|()| u32::from_be(int_be))
	}
	fn read_u32_be(&mut self) -> std::io::Result<u32>
	{
		let mut int_be = 0u32;
		let mut bytes = unsafe { std::slice::from_raw_parts_mut(std::mem::transmute::<_, *mut u8>(&mut int_be), std::mem::size_of::<u32>()) };
		self.read_exact(&mut bytes).map(|()| int_be)
	}
	fn read_f64(&mut self) -> std::io::Result<f64>
	{
		let mut double = 0f64;
		let mut bytes = unsafe { std::slice::from_raw_parts_mut(std::mem::transmute(&mut double), 8) };
		self.read_exact(&mut bytes).map(|()| double)
	}
	fn read_struct<U>(&mut self) -> std::io::Result<U>
	{
		let mut data: U = unsafe { std::mem::uninitialized() };
		let mut buffer = unsafe { std::slice::from_raw_parts_mut(std::mem::transmute(&mut data), std::mem::size_of::<U>()) };
		self.read_exact(&mut buffer).map(|()| data)
	}
}
/// Indicates that the structure represents part of file content
trait NativeFileContent<ReturnT: std::marker::Sized = Self>
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(ReturnT, std::fs::File), PSDLoadingError>;
}
/// Indicates that the structure represents part of file content(reports number of bytes)
trait UnsizedNativeFileContent<ReturnT: std::marker::Sized = Self>
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(ReturnT, usize, std::fs::File), PSDLoadingError>;
}

/// Binary Structures of PSD
#[repr(C, packed)] struct PSDHeader
{
	signature: [u8; 4], version: u16, reserved: [u8; 6],
	channels: u16, height: u32, width: u32, depth: u16, color_mode: u16
}
impl PSDHeader
{
	fn validate(self) -> Result<Self, PSDLoadingError>
	{
		let file_signature: u32 = unsafe { std::mem::transmute(['8' as u8, 'B' as u8, 'P' as u8, 'S' as u8]) };
		let read_signature: u32 = unsafe { std::mem::transmute(self.signature) };
		if read_signature != file_signature { Err(PSDLoadingError::SignatureMismatching("PSDHeader")) }
		else if u16::from_be(self.version) != 1 { Err(PSDLoadingError::VersionMismatching) }
		else { Ok(self) }
	}
}
impl NativeFileContent for PSDHeader
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Self, std::fs::File), PSDLoadingError>
	{
		fp.read_struct::<Self>().map_err(PSDLoadingError::from).and_then(PSDHeader::validate).map(|header| (header, fp))
	}
}

/// Structure of PSD
#[allow(dead_code)]
pub struct PhotoshopDocument
{
	channels: usize, width: usize, height: usize, depth: usize, color_mode: PSDColorMode,
	color_data: PSDColorModeData, image_resources: Vec<PSDImageResource>, layer_masks: PSDLayerAndMaskInfo,
	combined_image_data: PSDImageData
}
impl PhotoshopDocument
{
	pub fn open<PathT: AsRef<Path>>(path: PathT) -> Result<PhotoshopDocument, PSDLoadingError>
	{
		std::fs::File::open(path).map_err(PSDLoadingError::from).and_then(|fp|
		{
			let (header, rest) = try!(PSDHeader::read_from_file(fp));
			let (color_mode_data, rest) = try!(PSDColorModeData::read_from_file(rest));
			let (image_resources, rest) = try!(PSDImageResourceSection::read_from_file(rest));
			let (layers, rest) = try!(PSDLayerAndMaskInfo::read_from_file(rest));
			let (combined, _) = try!(PSDImageData::read_from_file(rest));

			Ok(PhotoshopDocument
			{
				channels: header.channels as usize, width: header.width as usize, height: header.height as usize, depth: header.depth as usize,
				color_mode: PSDColorMode::from(u16::from_be(header.color_mode)),
				color_data: color_mode_data, image_resources: image_resources, layer_masks: layers, combined_image_data: combined
			})
		})
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn loadable_full_color()
	{
		let psd = PhotoshopDocument::open("../assets/graphs/playerbullet.psd").unwrap();
		assert_eq!(psd.color_mode, PSDColorMode::RGB);
		assert!(psd.color_data.same_type(&PSDColorModeData::None));
		assert_eq!(psd.image_resources[0].id, PSDImageResourceID::CaptionDigest);
		assert!(!psd.layer_masks.layers.is_empty());
		assert_eq!(psd.layer_masks.layers[0].additional_infos[0].key_chars, ['l' as u8, 'u' as u8, 'n' as u8, 'i' as u8]);
		assert_eq!(psd.layer_masks.globalmask_adinfo[0].key_chars, ['P' as u8, 'a' as u8, 't' as u8, 't' as u8]);
	}
}
