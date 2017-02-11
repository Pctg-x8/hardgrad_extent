
use utils::*;
use constants::*;
use structures::*;
use nalgebra::*;
use interlude::*;
use rand;
use rand::distributions::*;
use super::bullet::*;
use std::rc::*;
use utils;
use {GameTime, GameUpdateArgs};

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
		left: f32, living_secs: GameTime, rezonator_left: u32, next: Continuous, next_raised: f32
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
			left: init_left, living_secs: 0.0, rezonator_left: 3, next: Continuous::Cont(0.0, Box::new(|pos, fq|
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
	pub fn update(&mut self, update_args: &GameUpdateArgs, frequest_queue: &mut Vec<FireRequest>) -> Option<(f32, f32)>
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
						// println!("dead... {}", spawngroup.local_index);
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
					rezonator_iref[1] -= 130.0f32.to_radians() * update_args.delta_time;
					rezonator_iref[2] += 220.0f32.to_radians() * update_args.delta_time;
					*living_secs += update_args.delta_time;
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
	use super::{GameTime, GameUpdateArgs};
	use std::cell::*;
	use std::rc::Rc;
	use std::mem::transmute;

	/// fn (x, y, livings, local_index) -> Option<block_index>
	pub type UnsafeAppearFnRef = *mut FnMut(f32, f32, &Rc<EntityLivings>, usize) -> Option<u32>;
	pub type ExecuteBoxFn = Box<Fn(&Rc<EntityLivings>, &mut GameUpdateArgs, UnsafeAppearFnRef) -> ExecuteState>;
	/// An execution state of Enemy Spawning Group
	pub enum ExecuteState
	{
		Term,
		Update(ExecuteBoxFn),
		Delay(GameTime, ExecuteBoxFn), WaitForAll(ExecuteBoxFn)
	}
	macro_rules! AdditionalArgsBoxed
	{
		($f: expr => $($a: expr),*) => {Box::new(|a, b, c| $f(a, b, c $(, $a)*))};
		(mv $f: expr => $($a: expr),*) => {Box::new(move |a, b, c| $f(a, b, c $(, $a)*))}
	}

	/// Enemy Spawning Strategies
	pub mod strategies
	{
		use rand;
		use rand::distributions::*;
		use std::rc::Rc;
		use super::{EntityLivings, UnsafeAppearFnRef, ExecuteState, GameTime, GameUpdateArgs};
		pub trait Strategy { fn begin(&self) -> ExecuteState; }

		pub struct RandomFall(pub GameTime, pub u32);
		impl Strategy for RandomFall
		{
			fn begin(&self) -> ExecuteState
			{
				use super::ExecuteState::*;

				fn recursive(l: &Rc<EntityLivings>, ua: &mut GameUpdateArgs, fun: UnsafeAppearFnRef, wait: GameTime, counter: u32) -> ExecuteState
				{
					let mut spawn_hrange = rand::distributions::Range::new(-25.0, 25.0);
					if counter > 0
					{
						// println!("Enemy::Fall");
						let next_index = l.zako.borrow().nextlife();
						if let Some(b) = unsafe { (&mut *fun)(spawn_hrange.sample(&mut ua.randomizer), 0.0, l, next_index) }
						{
							l.zako.borrow_mut().newlife(b);
						}
						Delay(wait, AdditionalArgsBoxed!(mv recursive => wait, counter - 1))
					}
					else { WaitForAll(Box::new(|_, _, _| Term)) }
				}
				let &RandomFall(w, c) = self;
				Update(AdditionalArgsBoxed!(mv recursive => w, c))
			}
		}
	}
	pub use self::strategies::Strategy as SpawnStrategy;

	pub struct EnemySpawnGroupExecute(GameTime, ExecuteState);
	impl EnemySpawnGroupExecute
	{
		fn new(c: ExecuteState) -> Self { EnemySpawnGroupExecute(0.0, c) }
		fn update(&mut self, update_args: &mut GameUpdateArgs, livings: &Rc<EntityLivings>, args: UnsafeAppearFnRef) -> bool
		{
			use self::ExecuteState::*;

			self.0 += update_args.delta_time;
			let newcont = match self.1
			{
				Update(ref f) => Some(f(livings, update_args, args)),
				Delay(d, ref f) => if self.0 >= d { Some(f(livings, update_args, args)) } else { None },
				WaitForAll(ref f) => if livings.zako.borrow().lefts == 0 { Some(f(livings, update_args, args)) } else { None },
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
		pub fn update(&mut self, update_args: &mut GameUpdateArgs, appear: &mut FnMut(f32, f32, &Rc<EntityLivings>, usize) -> Option<u32>) -> bool
		{
			let cont = self.engine.update(update_args, &self.livings, unsafe { transmute(appear) });
			/*if !cont
			{
				println!("End of Execution");
			}*/
			cont
		}
	}
}
pub use self::spawn_group::EnemyGroup;

pub type ManagerExecuteFn = Box<Fn(&mut EnemySquads, &mut GameUpdateArgs) -> ManagerExecuteState>;
pub enum ManagerExecuteState
{
	Terminated, Update(ManagerExecuteFn), Delay(GameTime, ManagerExecuteFn), WaitForAllSquads(ManagerExecuteFn)
}
pub struct ManagerEngine(ManagerExecuteState);
impl ManagerEngine
{
	fn begin(s: ManagerExecuteState) -> Self
	{
		ManagerEngine(s)
	}
	fn update(&mut self, update_args: &mut GameUpdateArgs, squads: &mut EnemySquads) -> bool
	{
		let next = match &mut self.0
		{
			&mut ManagerExecuteState::Update(ref f) => Some(f(squads, update_args)),
			&mut ManagerExecuteState::Delay(ref mut d, ref f) =>
			{
				*d -= update_args.delta_time;
				if *d <= 0.0 { Some(f(squads, update_args)) } else { None }
			},
			&mut ManagerExecuteState::WaitForAllSquads(ref f) => if squads.left == 0
			{
				Some(f(squads, update_args))
			} else { None },
			_ => None
		};
		if let Some(n) = next
		{
			if let ManagerExecuteState::Terminated = n { println!("Execution Terminated"); }
			self.0 = n;
		}
		if let ManagerExecuteState::Terminated = self.0 { false } else { true }
	}
}

const INITIAL_SQUADS_LIMIT: usize = 32;
pub struct EnemySquads
{
	objects: Vec<Option<EnemyGroup>>, freelist: utils::MemoryBlockManager, left: usize
}
impl EnemySquads
{
	fn new() -> Self
	{
		EnemySquads
		{
			objects: Vec::with_capacity(INITIAL_SQUADS_LIMIT), freelist: utils::MemoryBlockManager::new(0), left: 0
		}
	}
	fn spawn_squad<Strategy: spawn_group::SpawnStrategy>(&mut self, strategy: Strategy)
	{
		if let Some(n) = self.freelist.allocate()
		{
			self.objects[n as usize] = Some(EnemyGroup::new(strategy));
		}
		else
		{
			self.objects.push(Some(EnemyGroup::new(strategy)));
		}
		self.left += 1;
	}
	fn update_all(&mut self, update_args: &mut GameUpdateArgs, appear: &mut FnMut(f32, f32, &Rc<spawn_group::EntityLivings>, usize) -> Option<u32>)
	{
		for (n, gi) in self.objects.iter_mut().enumerate()
		{
			let emptiness = if let &mut Some(ref mut g) = gi
			{
				if !g.update(update_args, appear)
				{
					self.freelist.free(n as u32); true
				}
				else { false }
			}
			else { false };
			if emptiness { *gi = None; self.left -= 1; }
		}
	}
}
/// Top of all enemy squads
pub struct EnemyManager
{
	squads: EnemySquads, engine: ManagerEngine
}
impl EnemyManager
{
	pub fn new() -> Self
	{
		/*
			extern mgr: &mut EnemySquads;
			extern args: &GameUpdateArgs;

			loop
			{
				mgr.spawn_squad(spawn_group::strategies::RandomFall(0.1, 10));
				yield WaitForAllSquads;
				yield Delay(2.5);
			}
		*/
		fn eternal_loop(mgr: &mut EnemySquads, args: &mut GameUpdateArgs) -> ManagerExecuteState
		{
			mgr.spawn_squad(spawn_group::strategies::RandomFall(0.1, 10));
			ManagerExecuteState::WaitForAllSquads(Box::new(|_, _| ManagerExecuteState::Delay(2.5, Box::new(eternal_loop))))
		}
		let executions = ManagerExecuteState::Update(Box::new(eternal_loop));

		EnemyManager
		{
			squads: EnemySquads::new(), engine: ManagerEngine::begin(executions)
		}
	}
	pub fn update<AppearFn>(&mut self, update_args: &mut GameUpdateArgs, mut appear: AppearFn)
		where AppearFn: FnMut(f32, f32, &Rc<spawn_group::EntityLivings>, usize) -> Option<u32>
	{
		self.engine.update(update_args, &mut self.squads);
		self.squads.update_all(update_args, &mut appear);
	}
}
