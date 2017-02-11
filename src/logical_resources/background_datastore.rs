
use rand;
use rand::distributions::*;
use structures;
use constants::*;
use GameUpdateArgs;

pub struct BackgroundDatastore<'a>
{
	buffer_data: &'a mut [structures::BackgroundInstance; MAX_BK_COUNT],
	instance_data: &'a mut [u32; MAX_BK_COUNT]
}
impl <'a> BackgroundDatastore<'a>
{
	pub fn new(buffer_data_ref: &'a mut [structures::BackgroundInstance; MAX_BK_COUNT], instance_data_ref: &'a mut [u32; MAX_BK_COUNT]) -> Self
	{
		BackgroundDatastore
		{
			buffer_data: buffer_data_ref,
			instance_data: instance_data_ref
		}
	}
	pub fn update(&mut self, update_args: &mut GameUpdateArgs, appear: bool)
	{
		let mut require_appear = appear;
		let mut left_range = rand::distributions::Range::new(-14.0f32, 14.0f32);
		let mut count_range = rand::distributions::Range::new(2, 10);
		let mut scale_range = rand::distributions::Range::new(1.0f32, 3.0f32);
		for (i, m) in self.instance_data.iter_mut().enumerate()
		{
			if *m == 0
			{
				// instantiate randomly
				if require_appear
				{
					let scale = scale_range.sample(&mut update_args.randomizer);
					*m = 1;
					self.buffer_data[i].offset = [left_range.sample(&mut update_args.randomizer), -20.0f32, -20.0f32,
						count_range.sample(&mut update_args.randomizer) as f32];
					self.buffer_data[i].scale = [scale, scale, 1.0f32, 1.0f32];
					require_appear = false;
				}
			}
			else
			{
				self.buffer_data[i].offset[1] += update_args.delta_time * 22.0f32;
				*m = if self.buffer_data[i].offset[1] >= 20.0f32 { 0 } else { 1 };
			}
		}
	}
}
