// PascalString: Pascal Formatted string(heading number of characters)

use super::{PSDLoadingError, BinaryLoaderUtils};
use std;
use std::io::prelude::*;

pub fn read_from_file(mut fp: std::fs::File, pad_align: usize) -> Result<(Vec<u8>, usize, std::fs::File), PSDLoadingError>
{
	fp.read_u8().map_err(PSDLoadingError::from).and_then(|len| if len == 0
	{
		let mut pad_buffer = vec![0u8; pad_align - 1];
		fp.read_exact(&mut pad_buffer).map_err(PSDLoadingError::from).map(|()| (Vec::new(), pad_align, fp))
	}
	else
	{
		let padded_size = (((1.0 + len as f32) / 4.0).ceil() * 4.0) as usize;
		let mut bytes = vec![0u8; len as usize];
		let mut pad_bytes = vec![0u8; padded_size - len as usize - 1];
		fp.read_exact(&mut bytes).and_then(|()| fp.read_exact(&mut pad_bytes)).map_err(PSDLoadingError::from)
			.map(|()| (bytes, padded_size, fp))
	})
}
