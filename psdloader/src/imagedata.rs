use std;
use std::io::prelude::*;
use super::{BinaryLoaderUtils, PSDLoadingError, NativeFileContent};

/// Image Data for each channels
pub enum PSDChannelImageData
{
	Uncompressed(Vec<u8>), RunLengthCompressed(Vec<u8>),
	ZipWithoutPrediction(Vec<u8>), ZipWithPrediction(Vec<u8>)
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
}

pub enum PSDImageData
{
	Uncompressed(Vec<u8>), RunLengthCompressed(Vec<u8>),
	ZipWithoutPrediction(Vec<u8>), ZipWithPrediction(Vec<u8>)
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
