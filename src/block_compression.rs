// Block Compression(BC4/BC5) Algorithms

use std;

// Compress 4x4 block of single component bytes
fn compress_bytes(byte_iter: &[&u8]) -> Vec<u8>
{
	let max_value = byte_iter.iter().fold(0u8, |acc, &&x| std::cmp::max(x, acc));
	let min_value = byte_iter.iter().fold(0xffu8, |acc, &&x| std::cmp::min(x, acc));
	let interpolated = if max_value > min_value
	{
		// 4 interpolated values
		[
			min_value as f32, max_value as f32,
			(4.0f32 * min_value as f32 + 1.0f32 * max_value as f32) / 5.0f32,
			(3.0f32 * min_value as f32 + 2.0f32 * max_value as f32) / 5.0f32,
			(2.0f32 * min_value as f32 + 3.0f32 * max_value as f32) / 5.0f32,
			(1.0f32 * min_value as f32 + 4.0f32 * max_value as f32) / 5.0f32,
			0.0f32, 255.0f32
		]
	}
	else
	{
		// 6 interpolated values
		[
			min_value as f32, max_value as f32,
			(6.0f32 * min_value as f32 + 1.0f32 * max_value as f32) / 7.0f32,
			(5.0f32 * min_value as f32 + 2.0f32 * max_value as f32) / 7.0f32,
			(4.0f32 * min_value as f32 + 3.0f32 * max_value as f32) / 7.0f32,
			(3.0f32 * min_value as f32 + 4.0f32 * max_value as f32) / 7.0f32,
			(2.0f32 * min_value as f32 + 5.0f32 * max_value as f32) / 7.0f32,
			(1.0f32 * min_value as f32 + 6.0f32 * max_value as f32) / 7.0f32
		]
	};
	let encoded = byte_iter.into_iter().map(|&&v| interpolated.iter().enumerate().map(|(n, &i)| (n, (v as f32 - i).abs()))
			.fold((0, 255.0f32), |(nc, minv), (n, v)| if minv > v { (n, v) } else { (nc, minv) })).map(|(n, _)| n as u8).collect::<Vec<_>>();
	
	vec![
		min_value, max_value,
		(encoded[0] & 0b111) | ((encoded[1] & 0b111) << 3) | ((encoded[2] & 0b011) << 6),
		((encoded[2] & 0b100) >> 2) | ((encoded[3] & 0b111) << 1) | ((encoded[4] & 0b111) << 4) | ((encoded[5] & 0b001) << 7),
		((encoded[5] & 0b110) >> 1) | ((encoded[6] & 0b111) << 2) | ((encoded[7] & 0b111) << 5),
		(encoded[8] & 0b111) | ((encoded[9] & 0b111) << 3) | ((encoded[10] & 0b011) << 6),
		((encoded[10] & 0b100) >> 2) | ((encoded[11] & 0b111) << 1) | ((encoded[12] & 0b111) << 4) | ((encoded[13] & 0b001) << 7),
		((encoded[13] & 0b110) >> 1) | ((encoded[14] & 0b111) << 2) | ((encoded[15] & 0b111) << 5)
	]
}

pub fn compress_test()
{
	let source_block = [
		0u8, 1, 2, 3,
		4, 5, 6, 7,
		8, 9, 10, 11,
		12, 13, 14, 15
	];
	let encoded_data = compress_bytes(&source_block.iter().collect::<Vec<_>>());
	println!("Compression Result: {}, {}, {:b}-{:b}-{:b}-{:b}-{:b}-{:b}", encoded_data[0], encoded_data[1],
		encoded_data[2], encoded_data[3], encoded_data[4], encoded_data[5], encoded_data[6], encoded_data[7]);
	assert_eq!(encoded_data[0], 0);
	assert_eq!(encoded_data[1], 15);
}

pub struct BC4;
pub struct BC5;
pub trait CompressionAlgorithm
{
	fn compress(source: &[u8], size: (usize, usize)) -> Vec<u8>;
}
impl CompressionAlgorithm for BC4
{
	fn compress(source: &[u8], size: (usize, usize)) -> Vec<u8>
	{
		assert_eq!(size.0 * size.1, source.len());		// size matching
		assert!(size.0 % 4 == 0 && size.1 % 4 == 0);	// alignment matching

		let block_indices = (0 .. size.0 / 4).map(|x| x * 4).flat_map(|bx| (0 .. size.1 / 4).map(|x| x * 4).map(move |by| (bx, by)));
		block_indices.flat_map(|(bx, by)| compress_bytes(&(by .. by + 4).flat_map(|y| source[bx + y * size.0 .. (bx + 4) + y * size.0].iter()).collect::<Vec<_>>())).collect()
	}
}
impl CompressionAlgorithm for BC5
{
	fn compress(source: &[u8], size: (usize, usize)) -> Vec<u8>
	{
		assert_eq!(size.0 * size.1 * 2, source.len());		// size matching
		assert!(size.0 % 4 == 0 && size.1 % 4 == 0);		// alignment matching

		let block_indices = (0 .. size.0 / 4).map(|x| x * 4).flat_map(|bx| (0 .. size.1 / 4).map(|x| x * 4).map(move |by| (bx, by)));
		block_indices.flat_map(|(bx, by)|
		{
			let pixel_slice = (by .. by + 4).flat_map(|y| source[(bx + y * size.0) * 2 .. ((bx + 4) + y * size.0) * 2].iter());
			let (xslice, yslice): (Vec<_>, Vec<_>) = pixel_slice.enumerate().partition(|&(n, _)| n % 2 == 0);
			let mut x_comp = compress_bytes(&xslice.iter().map(|&(_, x)| x).collect::<Vec<_>>());
			let mut y_comp = compress_bytes(&yslice.iter().map(|&(_, y)| y).collect::<Vec<_>>());
			x_comp.append(&mut y_comp);
			x_comp
		}).collect()
	}
}
