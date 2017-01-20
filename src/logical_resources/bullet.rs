
use utils::*;
use constants::*;
use structures::*;
use interlude::CVector4;
use rayon::prelude::*;
use std;
use thread_scoped;

pub struct BulletDatastore<'a>
{
	instance_memory_ref: &'a mut [BulletInstance; MAX_BULLETS],
	memory_block_manager: MemoryBlockManager
}
impl<'a> BulletDatastore<'a>
{
	pub fn new(im_ref: &'a mut [BulletInstance; MAX_BULLETS]) -> Self
	{
		BulletDatastore { instance_memory_ref: im_ref, memory_block_manager: MemoryBlockManager::new(MAX_BULLETS as u32) }
	}
	pub fn allocate(&mut self) -> Option<u32>
	{
		let index = self.memory_block_manager.allocate();
		if let Some(index) = index { self.enable_instance(index); }
		index
	}
	pub fn free(&mut self, index: u32)
	{
		self.memory_block_manager.free(index);
		self.disable_instance(index);
	}

	fn enable_instance(&mut self, index: u32)
	{
		self.instance_memory_ref[index as usize].available = 1.0;
	}
	fn disable_instance(&mut self, index: u32)
	{
		self.instance_memory_ref[index as usize].available = 0.0;
	}

	pub fn init_lifetime(&mut self, index: u32) { self.instance_memory_ref[index as usize].lifetime = 0.0; }
	pub fn increase_all_lifetime(&mut self, amount: f32) { self.instance_memory_ref.par_iter_mut().for_each(|l| l.lifetime += amount); }
}

pub enum Bullet<'a>
{
	Free, Garbage(u32),
	Linear { block_index: u32, movec: [f32; 2], translation: &'a mut CVector4 }
}
unsafe impl<'a> Send for Bullet<'a> {}
impl<'a> Bullet<'a>
{
	pub fn init_linear(block_index: u32, tref: &'a mut CVector4, from: &CVector4, angle: f32, speed: f32) -> Self
	{
		*tref = *from;
		let (s, c) = angle.sin_cos();

		Bullet::Linear
		{
			block_index: block_index, translation: tref, 
			movec: [s * speed, c * speed]
		}
	}
	pub fn update(&mut self, delta_time: f32)
	{
		let died_index = match self
		{
			&mut Bullet::Linear { block_index, movec, ref mut translation } =>
			{
				// Linear motion
				translation[0] += movec[0] * delta_time;
				translation[1] += movec[1] * delta_time;
				if translation[0].abs() * 0.9 > SCREEN_HSIZE || !(-1.0 <= translation[1] && translation[1] <= SCREEN_VSIZE + 1.0)
				{
					Some(block_index)
				}
				else { None }
			},
			_ => None
		};

		if let Some(di) = died_index { *self = Bullet::Garbage(di); }
	}

	pub fn is_garbage(&self) -> bool { match self { &Bullet::Garbage(_) => true, _ => false } }
}

pub enum FireRequest
{
	Linears(Vec<(CVector4, f32, f32)>)
}
unsafe impl Send for FireRequest {}
unsafe impl Sync for FireRequest {}

pub struct BulletSpawner<'a>
{
	handle: thread_scoped::JoinGuard<'a, ()>, cancel_bus: std::sync::mpsc::Sender<()>
}
impl<'a> Drop for BulletSpawner<'a>
{
	fn drop(&mut self)
	{
		self.cancel_bus.send(()).unwrap();
		std::mem::replace(&mut self.handle, unsafe { std::mem::uninitialized() }).join();
	}
}
impl<'a> BulletSpawner<'a>
{
	pub fn new<F>(func: F) -> Self where F: FnOnce(std::sync::mpsc::Receiver<()>) + Send + 'a
	{
		let (send, recv) = std::sync::mpsc::channel();

		BulletSpawner
		{
			handle: unsafe { thread_scoped::scoped(move || func(recv)) }, cancel_bus: send
		}
	}
	pub fn cancel(mut self) { std::mem::drop(self); }
	pub fn detach(self) { std::mem::forget(self); }
}
