
use structures::*;
use nalgebra::*;
use LogicalInputTypes;
use interlude;
use std;

pub struct Player<'a>
{
	uniform_memory: &'a mut CVector4, instance_memory: &'a mut [CVector4; 2],
	living_secs: f32
}
impl<'a> Player<'a>
{
	pub fn new(uniform_ref: &'a mut CVector4, instance_ref: &'a mut [CVector4; 2]) -> Self
	{
		let u_quaternion = UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32));
		let quaternion_ref = u_quaternion.quaternion();

		instance_ref[0] = [quaternion_ref.i, quaternion_ref.j, quaternion_ref.k, quaternion_ref.w];
		instance_ref[1] = [quaternion_ref.i, quaternion_ref.j, quaternion_ref.k, quaternion_ref.w];
		*uniform_ref = [0.0f32, 38.0f32, 0.0f32, 0.0f32];

		Player
		{
			uniform_memory: uniform_ref, instance_memory: instance_ref, living_secs: 0.0f32
		}
	}
	pub fn update(&mut self, frame_delta: f32, input: &interlude::InputSystem<LogicalInputTypes>)
	{
		let u_quaternions = [
			UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32).normalize() * (260.0f32 * self.living_secs as f32).to_radians()),
			UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32).normalize() * (-260.0f32 * self.living_secs as f32 + 13.0f32).to_radians())
		];
		let mut quaternions = u_quaternions.iter().map(|x| x.quaternion()).map(|q| [q.i, q.j, q.k, q.w]);
		self.living_secs += frame_delta;

		self.uniform_memory[0] =
			(self.uniform_memory[0] + input[LogicalInputTypes::Horizontal] * 40.0f32 * frame_delta).max(-33.0f32).min(33.0f32);
		self.uniform_memory[1] =
			(self.uniform_memory[1] + input[LogicalInputTypes::Vertical] * 40.0f32 * frame_delta).max(1.5f32).min(45.0f32);

		self.instance_memory[0] = quaternions.next().unwrap();
		self.instance_memory[1] = quaternions.next().unwrap();
	}

	pub fn left(&self) -> f32 { self.uniform_memory[0] }
	pub fn top(&self) -> f32 { self.uniform_memory[1] }
}

pub enum PlayerBullet<'a>
{
	Free, Entity { block_index: u32, offs_sincos_ref: &'a mut CVector4 }, Garbage(u32)
}
impl<'a> PlayerBullet<'a>
{
	pub fn init(init_left: f32, init_top: f32, init_angle: f32, block_index: u32, offs_sincos_ref: &'a mut CVector4) -> Self
	{
		offs_sincos_ref[0] = init_left;
		offs_sincos_ref[1] = init_top;
		let (s, c) = init_angle.to_radians().sin_cos();
		offs_sincos_ref[2] = s; offs_sincos_ref[3] = c;

		PlayerBullet::Entity { block_index: block_index, offs_sincos_ref: offs_sincos_ref }
	}
	pub fn update(&mut self, delta_time: f32)
	{
		let died_index = match self
		{
			&mut PlayerBullet::Entity { block_index: block, offs_sincos_ref: ref mut offs_sincos } =>
			{
				offs_sincos[0] += offs_sincos[2] * 8.0 * 14.0 * delta_time;
				offs_sincos[1] -= offs_sincos[3] * 8.0 * 14.0 * delta_time;
				if offs_sincos[0].abs() > 33.0 || !(0.0 <= offs_sincos[1] && offs_sincos[1] <= 50.0)
				{
					offs_sincos[0] = std::f32::MAX;
					offs_sincos[1] = std::f32::MAX;
					Some(block)
				}
				else { None }
			}, _ => None
		};
		
		if let Some(bindex) = died_index { *self = PlayerBullet::Garbage(bindex); }
	}
	pub fn is_garbage(&self) -> bool { match self { &PlayerBullet::Garbage(_) => true, _ => false } }
}
