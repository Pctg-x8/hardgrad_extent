
use std;
use utils::*;
use constants::*;
use structures::*;
use nalgebra::*;
use interlude::*;
use rand;
use rand::distributions::*;
use std::sync::mpsc::*;
use super::bullet::*;
use time;

fn store_quaternion(to: &mut CVector4, q: &Quaternion<f32>)
{
	*to = [q.i, q.j, q.k, q.w];
}

pub struct EnemyDatastore<'a>
{
	instance_memory_ref: &'a mut [u32; MAX_ENEMY_COUNT],
	memory_block_manager: MemoryBlockManager
}
impl <'a> EnemyDatastore<'a>
{
	pub fn new(im_ref: &'a mut [u32; MAX_ENEMY_COUNT]) -> Self
	{
		EnemyDatastore
		{
			instance_memory_ref: im_ref, memory_block_manager: MemoryBlockManager::new(MAX_ENEMY_COUNT as u32)
		}
	}
	pub fn allocate_block(&mut self) -> Option<u32>
	{
		let index = self.memory_block_manager.allocate();
		if let Some(i) = index { self.enable_instance(i); }
		index
	}
	pub fn free_block(&mut self, index: u32)
	{
		self.memory_block_manager.free(index);
		self.disable_instance(index);
	}
	fn enable_instance(&mut self, index: u32)
	{
		self.instance_memory_ref[index as usize] = 1;
	}
	fn disable_instance(&mut self, index: u32)
	{
		self.instance_memory_ref[index as usize] = 0;
	}
}

pub enum Continuous
{
	Cont(f32, Box<Fn((f32, f32), &mut Vec<FireRequest>) -> Continuous>), Term
}

pub enum Enemy<'a>
{
	Free, Entity
	{
		block_index: u32, uniform_ref: &'a mut CharacterLocation, rezonator_iref: &'a mut CVector4,
		left: f32, living_secs: f32, rezonator_left: u32, next: Continuous, next_raised: f32
	}, Garbage(u32)
}
unsafe impl<'a> Send for Enemy<'a> {}
impl<'a> Enemy<'a>
{
	pub fn init(init_left: f32, block_index: u32, uref: &'a mut CharacterLocation, iref_rez: &'a mut CVector4) -> Self
	{
		uref.center_tf = [init_left, 0.0, 0.0, 0.0];
		store_quaternion(&mut uref.rotq[0], UnitQuaternion::new(Vector3::new(0.0, 0.0, 0.0)).quaternion());
		store_quaternion(&mut uref.rotq[1], UnitQuaternion::new(Vector3::new(0.0, 0.0, 0.0)).quaternion());
		*iref_rez = [3.0, 0.0, 0.0, 0.0];

		// let (frsender, uref_th) = unsafe { (fr_sender.clone(), CharacterLocationUnsafePtr(std::mem::transmute(uref))) };
		Enemy::Entity
		{
			block_index: block_index, uniform_ref: uref, rezonator_iref: iref_rez,
			left: init_left, living_secs: 0.0f32, rezonator_left: 3, next: Continuous::Cont(0.0, Box::new(|pos, fq|
			{
				fn sec1(pos: (f32, f32), fq: &mut Vec<FireRequest>) -> Continuous
				{
					let mut r = rand::thread_rng();
					let ra = rand::distributions::Range::new(0.0, 360.0);
					let rs = rand::distributions::Range::new(6.0, 12.0);
					fq.push(FireRequest::Linears(vec![([pos.0, pos.1, 0.0, 0.0], ra.ind_sample(&mut r), rs.ind_sample(&mut r))]));
					Continuous::Cont(1.0, Box::new(sec1))
				}
				sec1(pos, fq)
			})), next_raised: 0.0
		}
	}
	pub fn update(&mut self, delta_time: f32, frequest_queue: &mut Vec<FireRequest>) -> Option<(f32, f32)>
	{
		// update values
		let (gb_index, np) = match self
		{
			&mut Enemy::Entity { block_index, ref mut uniform_ref, ref mut rezonator_iref, left, ref mut living_secs, rezonator_left, ref mut next, ref mut next_raised } =>
			{
				let current_y = if *living_secs < 0.875f32
				{
					15.0f32 * (1.0f32 - (1.0f32 - *living_secs / 0.875f32).powi(2)) - 3.0f32
				}
				else
				{
					15.0f32 + (*living_secs - 0.875f32) * 2.5f32 - 3.0f32
				};
				if current_y >= 51.5
				{
					rezonator_iref[0] = 0.0;
					(Some(block_index), None)
				}
				else
				{
					uniform_ref.center_tf[1] = current_y;
					store_quaternion(&mut uniform_ref.rotq[0], UnitQuaternion::new(Vector3::new(-1.0, 0.0, 0.75).normalize() * (260.0 * *living_secs).to_radians()).quaternion());
					store_quaternion(&mut uniform_ref.rotq[1], UnitQuaternion::new(Vector3::new(1.0, -1.0, 0.5).normalize() * (-260.0 * *living_secs + 13.0).to_radians()).quaternion());
					rezonator_iref[0] = rezonator_left as f32;
					rezonator_iref[1] -= 130.0f32.to_radians() * delta_time;
					rezonator_iref[2] += 220.0f32.to_radians() * delta_time;
					*living_secs += delta_time;
					let newpos = (uniform_ref.center_tf[0], uniform_ref.center_tf[1]);

					let nx = if let &mut Continuous::Cont(n, ref f) = next
					{
						if *living_secs - *next_raised >= n
						{
							*next_raised = *living_secs;
							Some(f(newpos, frequest_queue))
						}
						else { None }
					}
					else { None };
					if let Some(n) = nx { *next = n; }
					(None, Some(newpos))
				}
			},
			_ => (None, None)
		};
		if let Some(gb) = gb_index { *self = Enemy::Garbage(gb); }
		np
	}
	pub fn is_garbage(&self) -> bool
	{
		match self { &Enemy::Garbage(_) => true, _ => false }
	}
}
