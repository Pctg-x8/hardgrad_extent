
use rand;
use utils::MemoryBlockManager;
use constants::*;
use structures::*;
use rand::distributions::*;
use std::collections::LinkedList;

pub struct LineBurstParticles<'a>
{
	memory: MemoryBlockManager,
	iref: &'a mut [LineBurstParticleGroup; MAX_LBPARTICLE_GROUPS],
	uref: &'a mut [LineBurstParticle; MAX_LBPARTICLES],
	estimated_lifetimes: LinkedList<(usize, f32)>,
	randomizer: rand::ThreadRng
}
impl<'a> LineBurstParticles<'a>
{
	pub fn new(iref: &'a mut [LineBurstParticleGroup; MAX_LBPARTICLE_GROUPS], uref: &'a mut [LineBurstParticle; MAX_LBPARTICLES]) -> Self
	{
		LineBurstParticles
		{
			memory: MemoryBlockManager::new(MAX_LBPARTICLE_GROUPS as u32),
			iref: iref, uref: uref,
			estimated_lifetimes: LinkedList::new(),
			randomizer: rand::thread_rng()
		}
	}
	pub fn spawn(&mut self, count: u32, x: f32, y: f32, lifestart_sec: f32)
	{
		let angle_distr = rand::distributions::Range::new(0.0f32, 360.0);
		let length_range = rand::distributions::Range::new(0.25, 2.0);
		let lifetime_mult_range = rand::distributions::Range::new(2.0, 6.0);
		let memindex = self.memory.allocate();
		if let Some(mindex) = memindex
		{
			self.iref[mindex as usize].count = count;
			self.iref[mindex as usize].start_point = [x, y];
			let mut estimated_lifetime = 0.0f32;
			for n in 0 .. count
			{
				let (s, c) = angle_distr.ind_sample(&mut self.randomizer).to_radians().sin_cos();
				let length = length_range.ind_sample(&mut self.randomizer);
				let lifetime_mult = lifetime_mult_range.ind_sample(&mut self.randomizer);

				self.uref[mindex as usize * MAX_LBPARTICLES_PER_GROUP + n as usize].sincos_xx = [s, c, 0.0, 0.0];
				self.uref[mindex as usize * MAX_LBPARTICLES_PER_GROUP + n as usize].length_colrel_lifestart_lifemult = [length, 0.0, lifestart_sec, lifetime_mult];
				estimated_lifetime = estimated_lifetime.max(lifetime_mult.recip());
			}
			self.estimated_lifetimes.push_front((mindex as usize, lifestart_sec + estimated_lifetime));
		}
		else { warn!("Memory for Line Burst Particles is Full!!"); }
	}
	pub fn garbage_collect(mut self, current_time: f32) -> Self
	{
		let (collected, survive): (LinkedList<_>, LinkedList<_>) = self.estimated_lifetimes.into_iter().partition(|&(_, n)| current_time >= n);
		for (i, _) in collected.into_iter()
		{
			self.memory.free(i as u32);
			self.iref[i].count = 0;
		}
		self.estimated_lifetimes = survive;
		self
	}
}
