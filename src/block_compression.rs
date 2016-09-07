// Block Compression(BC4/BC5) Algorithms Porting

//-------------------------------------------------------------------------------------
// BC4BC5.cpp
// BC.h
//  
// Block-compression (BC) functionality for BC4 and BC5 (DirectX 10 texture compression)
//
// THIS CODE AND INFORMATION IS PROVIDED "AS IS" WITHOUT WARRANTY OF
// ANY KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// THE IMPLIED WARRANTIES OF MERCHANTABILITY AND/OR FITNESS FOR A
// PARTICULAR PURPOSE.
//  
// Copyright (c) Microsoft Corporation. All rights reserved.
//
// http://go.microsoft.com/fwlink/?LinkId=248926
//-------------------------------------------------------------------------------------

use std;

const BLOCK_LEN: usize = 4;
const BLOCK_SIZE: usize = BLOCK_LEN * BLOCK_LEN;

// returns (pX, pY)
fn optimize_alpha_u(points: &[f32], steps: usize) -> (f32, f32)
{
	static C6: [f32; 6] = [5.0 / 5.0, 4.0 / 5.0, 3.0 / 5.0, 2.0 / 5.0, 1.0 / 5.0, 0.0 / 5.0];
	static D6: [f32; 6] = [0.0 / 5.0, 1.0 / 5.0, 2.0 / 5.0, 3.0 / 5.0, 4.0 / 5.0, 5.0 / 5.0];
	static C8: [f32; 8] = [7.0 / 7.0, 6.0 / 7.0, 5.0 / 7.0, 4.0 / 7.0, 3.0 / 7.0, 2.0 / 7.0, 1.0 / 7.0, 0.0 / 7.0];
	static D8: [f32; 8] = [0.0 / 7.0, 1.0 / 7.0, 2.0 / 7.0, 3.0 / 7.0, 4.0 / 7.0, 5.0 / 7.0, 6.0 / 7.0, 7.0 / 7.0];

	let (c, d) = if steps == 6 { (&C6[..], &D6[..]) } else { (&C8[..], &D8[..]) };

	const MAX_VALUE: f32 = 1.0;
	const MIN_VALUE: f32 = 0.0;

	// Find Min and Max points, as starting point
	let (mut fx, mut fy) = if steps == 8
	{
		(points.iter().fold(MAX_VALUE, |acc, &x| acc.min(x)),
		points.iter().fold(MIN_VALUE, |acc, &x| acc.max(x)))
	}
	else
	{
		(points.iter().fold(MAX_VALUE, |acc, &x| if x < acc && x > MIN_VALUE { x } else { acc }),
		points.iter().fold(MIN_VALUE, |acc, &x| if x > acc && x < MAX_VALUE { x } else { acc }))
	};
	fy = if steps == 6 && fx == fy { MAX_VALUE } else { fy };

	// Use Newton's Method to find local minima of sum-of-squares error
	let f_steps = steps - 1;
	for _ in 0 .. 8
	{
		if fy - fx < 1.0f32 / 256.0f32 { break; }
		let f_scale = f_steps as f32 / (fy - fx);

		// Calculate new steps
		let mut p_steps: Vec<f32> = (0 .. steps).map(|n| c[n] * fx + d[n] * fy).collect();
		if steps == 6
		{
			p_steps.push(MIN_VALUE); p_steps.push(MAX_VALUE);
		}

		// Evaluate function, and derivatives
		let (mut dx, mut dy, mut d2x, mut d2y) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
		for n in 0 .. BLOCK_SIZE
		{
			let f_dot = (points[n] - fx) * f_scale;
			let i_step = if f_dot <= 0.0
			{
				if steps == 6 && points[n] <= fx * 0.5 { 6 } else { 0 }
			}
			else if f_dot >= f_steps as f32
			{
				if steps == 6 && points[n] >= (fy + 1.0) * 0.5 { 7 } else { steps - 1 }
			}
			else { (f_dot + 0.5) as usize };

			if i_step < steps
			{
				// D3DX had this computation backwards (points[y][x] - steps[i_step])
				// this fix improves RMS of the alpha component
				let diff = p_steps[i_step] - points[n];
				dx += c[i_step] * diff;
				d2x += c[i_step] * c[i_step];
				dy += d[i_step] * diff;
				d2y += d[i_step] * d[i_step];
			}
		}

		// Move endpoints
		if d2x > 0.0 { fx -= dx / d2x; }
		if d2y > 0.0 { fy -= dy / d2y; }
		if fx > fy { std::mem::swap(&mut fx, &mut fy); }
		if dx * dx < 1.0 / 64.0 && dy * dy < 1.0 / 64.0 { break; }
	}

	(fx.max(MIN_VALUE).min(MAX_VALUE), fy.max(MIN_VALUE).min(MAX_VALUE))
}

// returns (endpoint0, endpoint1)
fn find_endpoints_bc4u(texels: &[f32]) -> (u8, u8)
{
	// The boundary of codec for signed/unsigned format
	const MIN_NORM: f32 = 0.0;
	const MAX_NORM: f32 = 1.0;

	// Find max.min of input texels
	let block_max = texels.iter().fold(texels[0], |acc, &x| acc.max(x));
	let block_min = texels.iter().fold(texels[0], |acc, &x| acc.min(x));

	// If there are boundary values in input texels, Should use 4 block-coded to guarantee
	// the exact code of the boundary values
	let using_4_block_codec = MIN_NORM == block_min || MAX_NORM == block_max;

	// Using optimize
	if using_4_block_codec
	{
		let (start, end) = optimize_alpha_u(texels, 8);
		((start * 255.0) as u8, (end * 255.0) as u8)
	}
	else
	{
		let (start, end) = optimize_alpha_u(texels, 6);
		((start * 255.0) as u8, (end * 255.0) as u8)
	}
}
fn bc4_decode_from_index(r0: u8, r1: u8, index: usize) -> f32
{
	match index
	{
		0 => r0 as f32 / 255.0, 1 => r1 as f32 / 255.0,
		6 if r0 <= r1 => 0.0,
		7 if r0 <= r1 => 1.0,
		_ => if r0 > r1
		{
			((r0 as f32 / 255.0) * (8 - index) as f32 + (r1 as f32 / 255.0) * (index - 1) as f32) / 7.0
		}
		else
		{
			((r0 as f32 / 255.0) * (6 - index) as f32 + (r1 as f32 / 255.0) * (index - 1) as f32) / 5.0
		}
	}
}
// returns indices
fn find_closest_unorm(r0: u8, r1: u8, texels: &[f32]) -> Vec<u8>
{
	let gradients = (0 .. 8).map(|n| bc4_decode_from_index(r0, r1, n)).collect::<Vec<_>>();
	(0 .. BLOCK_SIZE).map(|n|
	{
		let (best_index, _) = gradients.iter().enumerate().fold((0, 100000.0f32), |(bi, bd), (i, &g)|
		{
			let current_delta = (g - texels[n]).abs();
			if current_delta < bd { (i, current_delta) } else { (bi, bd) }
		});
		best_index as u8
	}).collect()
}
fn unorm(v: u8) -> f32 { v as f32 / 255.0 }
fn v8_to_u64_encode(src: &[u8]) -> u64
{
	assert!(src.len() < 64 / 3);
	src.iter().enumerate().fold(0u64, |acc, (n, &x)|
	{
		assert!(x < 8);
		acc | (((x & 0b111) as u64) << (n * 3))
	})
}

#[repr(C)] struct CompressedBlockData
{
	r0: u8, r1: u8, indices: [u8; 6]
}
#[repr(C)] struct CompressedBlockData2
{
	r: CompressedBlockData, g: CompressedBlockData
}

fn encode_block_single(src: &[u8], pitch: usize, bx: usize, by: usize) -> CompressedBlockData
{
	let block_normalized = (by .. by + 4).flat_map(|y| (bx .. bx + 4).map(move |x| unorm(src[x + y * pitch]))).collect::<Vec<_>>();
	let (r0, r1) = find_endpoints_bc4u(&block_normalized);
	let indices_t = find_closest_unorm(r0, r1, &block_normalized);
	let indices = v8_to_u64_encode(&indices_t);
	let mut cb = CompressedBlockData { r0: r0, r1: r1, indices: [0; 6] };
	cb.indices.copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 8]>(indices) }[..6]);
	cb
}
fn encode_block_double(src: &[u8], pitch: usize, bx: usize, by: usize) -> CompressedBlockData2
{
	let block_normalized_r = (by .. by + 4).flat_map(|y| (bx .. bx + 4).map(move |x| unorm(src[(x + y * pitch) * 2]))).collect::<Vec<_>>();
	let block_normalized_g = (by .. by + 4).flat_map(|y| (bx .. bx + 4).map(move |x| unorm(src[(x + y * pitch) * 2 + 1]))).collect::<Vec<_>>();
	let (r0, r1) = find_endpoints_bc4u(&block_normalized_r);
	let (g0, g1) = find_endpoints_bc4u(&block_normalized_g);
	let (indices_r, indices_g) = (
		v8_to_u64_encode(&find_closest_unorm(r0, r1, &block_normalized_r)),
		v8_to_u64_encode(&find_closest_unorm(g0, g1, &block_normalized_g))
	);
	let mut cbs = CompressedBlockData2
	{
		r: CompressedBlockData { r0: r0, r1: r1, indices: [0; 6] },
		g: CompressedBlockData { r0: g0, r1: g1, indices: [0; 6] }
	};
	cbs.r.indices.copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 8]>(indices_r) }[..6]);
	cbs.g.indices.copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 8]>(indices_g) }[..6]);
	cbs
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

		let compressed_blocks = (0 .. size.1 / 4).map(|y| y * 4).flat_map(|y| (0 .. size.0 / 4).map(|x| x * 4).map(move |x| (x, y))).map(|(bx, by)|
		{
			encode_block_single(&source, size.0, bx, by)
		}).collect::<Vec<_>>();
		unsafe { std::slice::from_raw_parts(compressed_blocks.as_ptr() as *const u8, compressed_blocks.len() * 8) }.into()
	}
}
impl CompressionAlgorithm for BC5
{
	fn compress(source: &[u8], size: (usize, usize)) -> Vec<u8>
	{
		assert_eq!(size.0 * size.1 * 2, source.len());		// size matching
		assert!(size.0 % 4 == 0 && size.1 % 4 == 0);		// alignment matching

		let compressed_blocks = (0 .. size.1 / 4).map(|y| y * 4).flat_map(|y| (0 .. size.0 / 4).map(|x| x * 4).map(move |x| (x, y))).map(|(bx, by)|
		{
			encode_block_double(&source, size.0, bx, by)
		}).collect::<Vec<_>>();
		unsafe { std::slice::from_raw_parts(compressed_blocks.as_ptr() as *const u8, compressed_blocks.len() * 16) }.into()
	}
}
