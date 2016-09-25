use std;
use std::io::prelude::*;
use super::{BinaryLoaderUtils, PSDLoadingError, NativeFileContent, PSDLayerRect};

/// Image Data for each channels
pub enum PSDChannelImageData
{
	Uncompressed(Vec<u8>), RunLengthCompressed(Vec<u8>),
	ZipWithoutPrediction(Vec<u8>), ZipWithPrediction(Vec<u8>)
}
pub struct DecompressedChannelImageData<'a> { data: Vec<u8>, content_rect: &'a PSDLayerRect }
impl<'a> DecompressedChannelImageData<'a>
{
	pub fn fetch(&self, x: usize, y: usize) -> u8 { self.data[x + y * self.content_rect.width() as usize] }
}
impl PSDChannelImageData
{
	pub fn read_from_file(mut fp: std::fs::File, length: usize) -> Result<(Self, std::fs::File), PSDLoadingError>
	{
		let dtype = try!(fp.read_u16());
		let mut buf = vec![0u8; length - 2];
		fp.read_exact(&mut buf).map_err(PSDLoadingError::from).map(|()| match dtype
		{
			0 => (PSDChannelImageData::Uncompressed(buf), fp),
			1 => (PSDChannelImageData::RunLengthCompressed(buf), fp),
			2 => (PSDChannelImageData::ZipWithoutPrediction(buf), fp),
			3 => (PSDChannelImageData::ZipWithPrediction(buf), fp),
			_ => unreachable!()
		})
	}
	pub fn decompress<'a>(&self, content_rect: &'a PSDLayerRect) -> DecompressedChannelImageData<'a>
	{
		match self
		{
			&PSDChannelImageData::Uncompressed(ref b) =>
			{
				DecompressedChannelImageData { data: b.clone(), content_rect: content_rect }
			},
			&PSDChannelImageData::RunLengthCompressed(ref b) =>
			{
				DecompressedChannelImageData { data: unpackbits(b, content_rect.lines() as usize), content_rect: content_rect }
			},
			_ => unimplemented!()
		}
	}
}

pub enum PSDImageData
{
	Uncompressed(Vec<u8>), RunLengthCompressed(Vec<u8>),
	ZipWithoutPrediction(Vec<u8>), ZipWithPrediction(Vec<u8>)
}
pub struct DecompressedPSDImagePlane<'a>
{
	dref: &'a DecompressedPSDImageData, x: usize, y: usize
}
impl<'a> std::ops::Index<usize> for DecompressedPSDImagePlane<'a>
{
	type Output = u8;
	fn index(&self, c: usize) -> &u8 { &self.dref.data[self.x + self.y * self.dref.width + c * (self.dref.width * self.dref.height)] }
}
pub struct DecompressedPSDImageData { data: Vec<u8>, pub width: usize, pub height: usize, pub channels: usize }
impl DecompressedPSDImageData
{
	pub fn fetch(&self, x: usize, y: usize, c: usize) -> u8 { self.data[x + y * self.width + c * (self.width * self.height)] }
	pub fn pixel(&self, x: usize, y: usize) -> DecompressedPSDImagePlane { DecompressedPSDImagePlane { dref: self, x: x, y: y } }
}
impl NativeFileContent for PSDImageData
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Self, std::fs::File), PSDLoadingError>
	{
		let dtype = try!(fp.read_u16());
		let mut buffer = Vec::new();
		fp.read_to_end(&mut buffer).map_err(PSDLoadingError::from).map(|_| match dtype
		{
			0 => (PSDImageData::Uncompressed(buffer), fp),
			1 => (PSDImageData::RunLengthCompressed(buffer), fp),
			2 => (PSDImageData::ZipWithoutPrediction(buffer), fp),
			3 => (PSDImageData::ZipWithPrediction(buffer), fp),
			_ => unreachable!()
		})
	}
}
impl PSDImageData
{
	pub fn dump(&self)
	{
		match self
		{
			&PSDImageData::Uncompressed(ref b) => println!("ImageData:Uncompressed: {}", b.len()),
			&PSDImageData::RunLengthCompressed(ref b) => println!("ImageData:RunLengthCompressed: {}", b.len()),
			&PSDImageData::ZipWithoutPrediction(ref b) => println!("ImageData:ZipWOPrediction: {}", b.len()),
			&PSDImageData::ZipWithPrediction(ref b) => println!("ImageData:ZipWPrediction: {}", b.len())
		}
	}
	pub fn decompress(&self, cols: usize, rows: usize, channels: usize) -> DecompressedPSDImageData
	{
		match self
		{
			&PSDImageData::Uncompressed(ref b) =>
			{
				DecompressedPSDImageData { data: b.clone(), width: cols, height: rows, channels: channels }
			},
			&PSDImageData::RunLengthCompressed(ref b) =>
			{
				DecompressedPSDImageData { data: unpackbits(b, channels * rows), width: cols, height: rows, channels: channels }
			},
			_ => unimplemented!()
		}
	}
}

// PackBits algorithm in Macintosh ROM
fn unpackbits(input: &[u8], scanlines: usize) -> Vec<u8>
{
	let bytes_per_line: Vec<_> = input[0 .. scanlines * 2].chunks(2).map(|b| ((b[0] as u16) << 8) | b[1] as u16).collect();
	let mut current_slice = &input[scanlines * 2..];
	let mut unpacked = Vec::new();
	for b in bytes_per_line
	{
		let mut line_slice = &current_slice[..b as usize];
		while !line_slice.is_empty()
		{
			line_slice = match line_slice[0]
			{
				0x80 => &line_slice[1..], /* nop */
				0x00 ... 0x7f =>
				{
					for n in 1 .. line_slice[0] + 2 { unpacked.push(line_slice[n as usize]); }
					&line_slice[line_slice[0] as usize + 2..]
				},
				_ =>
				{
					let repeats = 1 - line_slice[0] as i8;
					for _ in 0 .. repeats { unpacked.push(line_slice[1]); }
					&line_slice[2..]
				}
			};
		}
		current_slice = &current_slice[b as usize..];
	}
	unpacked
}
