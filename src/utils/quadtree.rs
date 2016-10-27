// Linear Quad-Tree

use std;

pub fn bithash(left: f32, top: f32) -> u8
{
	// yx yx yx yx
	fn bitdisc(x: u8) -> u8
	{
		let x = (x | (x << 2)) & 0x33;
		(x | (x << 1)) & 0x55
	}
	let x = (unsafe { std::mem::transmute::<_, u32>(((left / 9.0) * 2.0).trunc() as i32) }) as u8;
	let y = ((top * 2.0).trunc() as u32 >> 3) as u8;
	bitdisc(x & 0x0f) | (bitdisc(y & 0x0f) << 1)
}
