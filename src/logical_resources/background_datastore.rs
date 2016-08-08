
use rand; use time;
use rand::distributions::*;
use structures;

pub struct BackgroundDatastore;
impl BackgroundDatastore
{
	pub fn new() -> BackgroundDatastore
	{
		BackgroundDatastore
	}
	pub fn update(&self, uniform_memory_ref: &mut structures::UniformMemory, instance_memory_ref: &mut structures::InstanceMemory, mut randomizer: &mut rand::Rng, delta_time: time::Duration, appear: bool)
	{
		let delta_sec = delta_time.num_microseconds().unwrap_or(0) as f32 / (1000.0f32 * 1000.0f32);
        let mut require_appear = appear;
        let mut left_range = rand::distributions::Range::new(-14.0f32, 14.0f32);
		let mut count_range = rand::distributions::Range::new(2, 10);
		let mut scale_range = rand::distributions::Range::new(1.0f32, 3.0f32);
		for (i, m) in instance_memory_ref.background_instance_mult.iter_mut().enumerate()
		{
			if *m == 0
			{
				// instantiate randomly
				if require_appear
				{
					let scale = scale_range.sample(&mut randomizer);
					*m = 1;
					uniform_memory_ref.background_instance_data[i].offset = [left_range.sample(&mut randomizer), -20.0f32, -20.0f32, count_range.sample(&mut randomizer) as f32];
					uniform_memory_ref.background_instance_data[i].scale = [scale, scale, 1.0f32, 1.0f32];
                    require_appear = false;
				}
			}
			else
			{
				uniform_memory_ref.background_instance_data[i].offset[1] += delta_sec * 22.0f32;
				*m = if uniform_memory_ref.background_instance_data[i].offset[1] >= 20.0f32 { 0 } else { 1 };
			}
		}
	}
}
