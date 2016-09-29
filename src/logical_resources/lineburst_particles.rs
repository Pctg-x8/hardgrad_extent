
use rand;
use utils::MemoryBlockManager;
use constants::*;
use structures::*;
use rand::distributions::*;

pub struct LineBurstParticles<'a>
{
	pub memory: MemoryBlockManager, pub iref: &'a mut [LineBurstParticleGroup; MAX_LBPARTICLE_GROUPS], randomizer: rand::ThreadRng
}
impl<'a> LineBurstParticles<'a>
{
	pub fn new(iref: &'a mut [LineBurstParticleGroup; MAX_LBPARTICLE_GROUPS]) -> Self
	{
		LineBurstParticles
		{
			memory: MemoryBlockManager::new(MAX_LBPARTICLE_GROUPS as u32),
			iref: iref, randomizer: rand::thread_rng()
		}
	}
	pub fn spawn(&mut self, count: u32, x: f32, y: f32, lifestart_sec: f32)
	{
		let angle_distr = rand::distributions::Range::new(0.0f32, 360.0);
		let length_range = rand::distributions::Range::new(0.5, 4.0);
		let lifetime_mult_range = rand::distributions::Range::new(0.5, 2.0);
		let memindex = self.memory.allocate();
		if let Some(mindex) = memindex
		{
			self.iref[mindex as usize].count = count;
			self.iref[mindex as usize].start_point = [x, y];
			for n in 0 .. count
			{
				let (s, c) = angle_distr.ind_sample(&mut self.randomizer).sin_cos();
				let length = length_range.ind_sample(&mut self.randomizer);
				let lifetime_mult = lifetime_mult_range.ind_sample(&mut self.randomizer);
				self.iref[mindex as usize].particles[n as usize].sincos_xx = [s, c, 0.0, 0.0];
				self.iref[mindex as usize].particles[n as usize].length_colrel_lifestart_lifemult = [length, 0.0, lifestart_sec, lifetime_mult];
			}
		}
		else { warn!("Memory for Line Burst Particles is Full!!"); }
	}
}
