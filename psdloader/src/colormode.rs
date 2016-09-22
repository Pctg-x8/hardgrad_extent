use std;
use super::{NativeFileContent, PSDLoadingError, BinaryLoaderUtils};
use std::io::prelude::*;

pub type PSDPaletteElement = [u8; 3];	// ColorModeData element in Indexed Color Mode
pub enum PSDColorModeData
{
	None,
	IndexedPalette([PSDPaletteElement; 256]),	// 768 bytes,
	DuotonePalette(Vec<u8>)						// Unknown bytes(undocumented)
}
impl std::fmt::Debug for PSDColorModeData
{
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		match self
		{
			&PSDColorModeData::None => write!(formatter, "PSDColorModeData::None"),
			&PSDColorModeData::IndexedPalette(_) => write!(formatter, "PSDColorModeData::IndexedPalette(...)"),
			&PSDColorModeData::DuotonePalette(_) => write!(formatter, "PSDColorModeData::DuotonePalette(...)")
		}
	}
}
impl PSDColorModeData
{
	pub fn same_type(&self, other: &PSDColorModeData) -> bool
	{
		let is_comparing_with_none = match other { &PSDColorModeData::None => true, _ => false };
		match self
		{
			&PSDColorModeData::None if is_comparing_with_none => true,
			&PSDColorModeData::IndexedPalette(_) => if let &PSDColorModeData::IndexedPalette(_) = other { true } else { false },
			&PSDColorModeData::DuotonePalette(_) => if let &PSDColorModeData::DuotonePalette(_) = other { true } else { false },
			_ => false
		}
	}
}
#[repr(u8)] #[derive(PartialEq, Eq, Debug)]
pub enum PSDColorMode
{
	Bitmap = 0, Grayscale = 1, Indexed = 2, RGB = 3, CMYK = 4,
	Multichannel = 7, Duotone = 8, Lab = 9
}
impl std::convert::From<u16> for PSDColorMode
{
	fn from(v: u16) -> Self
	{
		match v
		{
			0 => PSDColorMode::Bitmap,
			1 => PSDColorMode::Grayscale,
			2 => PSDColorMode::Indexed,
			3 => PSDColorMode::RGB,
			4 => PSDColorMode::CMYK,
			7 => PSDColorMode::Multichannel,
			8 => PSDColorMode::Duotone,
			9 => PSDColorMode::Lab,
			_ => unreachable!()
		}
	}
}

impl NativeFileContent for PSDColorModeData
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(PSDColorModeData, std::fs::File), PSDLoadingError>
	{
		fp.read_u32().map_err(PSDLoadingError::from).and_then(|section_length| match section_length
		{
			0 => Ok((PSDColorModeData::None, fp)),
			768 =>
			{
				let mut indexed_palette_data = [[0; 3]; 256];
				let mut bytes = unsafe { std::slice::from_raw_parts_mut(std::mem::transmute::<_, *mut u8>(&mut indexed_palette_data), 3 * 256) };
				fp.read_exact(&mut bytes).map_err(PSDLoadingError::from).map(|()| (PSDColorModeData::IndexedPalette(indexed_palette_data), fp))
			},
			_ =>
			{
				let mut duotone_data = vec![0u8; section_length as usize];
				fp.read_exact(&mut duotone_data).map_err(PSDLoadingError::from).map(|()| (PSDColorModeData::DuotonePalette(duotone_data), fp))
			}
		})
	}
}
