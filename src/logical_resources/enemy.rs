
use utils::*;
use constants::*;
use structures::*;
use nalgebra::*;
use interlude::*;
use rand;
use rand::distributions::*;
use super::bullet::*;
use std;
use std::cell::RefCell;
use std::rc::*;

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

pub struct SpawnGroupRef { livings: Weak<spawn_group::EntityLivings>, local_index: usize }
pub enum Enemy<'a>
{
	Free, Entity
	{
		block_index: u32, spawngroup: SpawnGroupRef, uniform_ref: &'a mut CharacterLocation, rezonator_iref: &'a mut CVector4,
		left: f32, living_secs: f32, rezonator_left: u32, next: Continuous, next_raised: f32
	}, Garbage(u32)
}
unsafe impl<'a> Send for Enemy<'a> {}
impl<'a> Enemy<'a>
{
	pub fn init(init_left: f32, block_index: u32, livings: &Rc<spawn_group::EntityLivings>, sglx: usize,
		uref: &'a mut CharacterLocation, iref_rez: &'a mut CVector4) -> Self
	{
		uref.center_tf = [init_left, 0.0, 0.0, 0.0];
		store_quaternion(&mut uref.rotq[0], UnitQuaternion::new(Vector3::new(0.0, 0.0, 0.0)).quaternion());
		store_quaternion(&mut uref.rotq[1], UnitQuaternion::new(Vector3::new(0.0, 0.0, 0.0)).quaternion());
		*iref_rez = [3.0, 0.0, 0.0, 0.0];

		// let (frsender, uref_th) = unsafe { (fr_sender.clone(), CharacterLocationUnsafePtr(std::mem::transmute(uref))) };
		Enemy::Entity
		{
			block_index: block_index, spawngroup: SpawnGroupRef { livings: Rc::downgrade(livings), local_index: sglx }, uniform_ref: uref, rezonator_iref: iref_rez,
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
			&mut Enemy::Entity
			{
				block_index, ref spawngroup, ref mut uniform_ref, ref mut rezonator_iref, ref mut living_secs,
				rezonator_left, ref mut next, ref mut next_raised, ..
			} => {
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
					if let Some(lv) = spawngroup.livings.upgrade()
					{
						println!("dead... {}", spawngroup.local_index);
						lv.zako_mut().die(spawngroup.local_index);
					}
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
		if let Some(gb) = gb_index
		{
			*self = Enemy::Garbage(gb);
		}
		np
	}
	pub fn is_garbage(&self) -> bool
	{
		match self { &Enemy::Garbage(_) => true, _ => false }
	}
}

/// Enemy Spawn Group
pub mod spawn_group
{
	use std::cell::*;
	use std::rc::Rc;
	use std::mem::transmute;

	/// fn (x, y, livings, local_index) -> Option<block_index>
	pub type UnsafeAppearFnRef = *mut FnMut(f32, f32, &Rc<EntityLivings>, usize) -> Option<u32>;
	pub type ExecuteBoxFn = Box<Fn(&Rc<EntityLivings>, UnsafeAppearFnRef) -> ExecuteState>;
	/// An execution state of Enemy Spawning Group
	pub enum ExecuteState
	{
		Term,
		Update(ExecuteBoxFn),
		Delay(f32, ExecuteBoxFn), WaitForAll(ExecuteBoxFn)
	}

	/// Enemy Spawning Strategies
	pub mod strategies
	{
		use std::rc::Rc;
		use super::{EntityLivings, UnsafeAppearFnRef, ExecuteState};
		pub trait Strategy { fn begin(&self) -> ExecuteState; }

		pub struct RandomFall(pub f32, pub u32);
		impl Strategy for RandomFall
		{
			fn begin(&self) -> ExecuteState
			{
				use super::ExecuteState::*;

				fn recursive(l: &Rc<EntityLivings>, fun: UnsafeAppearFnRef, wait: f32, counter: u32) -> ExecuteState
				{
					if counter > 0
					{
						// println!("Enemy::Fall");
						let next_index = l.zako.borrow().nextlife();
						if let Some(b) = unsafe { (&mut *fun)(0.0, 0.0, l, next_index) }
						{
							l.zako.borrow_mut().newlife(b);
						}
						Delay(wait, Box::new(move |l, f| recursive(l, f, wait, counter - 1)))
					}
					else { WaitForAll(Box::new(|_, _| Term)) }
				}
				let &RandomFall(w, c) = self;
				Update(Box::new(move |l, f| recursive(l, f, w, c)))
			}
		}
	}
	pub use self::strategies::Strategy as SpawnStrategy;

	pub struct EnemySpawnGroupExecute(f32, ExecuteState);
	impl EnemySpawnGroupExecute
	{
		fn new(c: ExecuteState) -> Self { EnemySpawnGroupExecute(0.0, c) }
		fn update(&mut self, delta_time: f32, livings: &Rc<EntityLivings>, args: UnsafeAppearFnRef) -> bool
		{
			use self::ExecuteState::*;

			self.0 += delta_time;
			let newcont = match self.1
			{
				Update(ref f) => Some(f(livings, args)),
				Delay(d, ref f) => if self.0 >= d { Some(f(livings, args)) } else { None },
				WaitForAll(ref f) => if livings.zako.borrow().lefts == 0 { Some(f(livings, args)) } else { None },
				_ => None
			};
			if let Some(nc) = newcont
			{
				self.0 = 0.0;
				self.1 = nc;
			}
			if let Term = self.1 { false } else { true }
		}
	}
	pub enum LivingState { Left, Dead }
	pub struct ClassifiedLivingStates { list: Vec<(u32, LivingState)>, lefts: usize }
	impl ClassifiedLivingStates
	{
		pub fn new() -> Self { ClassifiedLivingStates { list: Vec::new(), lefts: 0 } }
		pub fn newlife(&mut self, index: u32) { self.list.push((index, LivingState::Left)); self.lefts += 1; }
		pub fn die(&mut self, local_index: usize) { self.list[local_index].1 = LivingState::Dead; self.lefts -= 1; }
		pub fn nextlife(&self) -> usize { self.list.len() }
	}
	pub struct EntityLivings
	{
		zako: RefCell<ClassifiedLivingStates>
	}
	impl EntityLivings
	{
		pub fn zako_mut(&self) -> RefMut<ClassifiedLivingStates> { self.zako.borrow_mut() }
	}
	pub struct EnemyGroup
	{
		engine: EnemySpawnGroupExecute, livings: Rc<EntityLivings>
	}
	impl EnemyGroup
	{
		pub fn new<Strategy: SpawnStrategy>(st: Strategy) -> Self
		{
			EnemyGroup
			{
				engine: EnemySpawnGroupExecute::new(st.begin()), livings: Rc::new(EntityLivings
				{
					zako: RefCell::new(ClassifiedLivingStates::new())
				})
			}
		}
		pub fn update<F: FnMut(f32, f32, &Rc<EntityLivings>, usize) -> Option<u32>>(&mut self, delta_time: f32, mut appear: F) -> bool
		{
			let cont = self.engine.update(delta_time, &self.livings,
				unsafe { transmute::<&mut FnMut(f32, f32, &Rc<EntityLivings>, usize) -> Option<u32>, _>(&mut appear) });
			if !cont
			{
				println!("End of Execution");
			}
			cont
		}
	}
}
pub use self::spawn_group::EnemyGroup;
