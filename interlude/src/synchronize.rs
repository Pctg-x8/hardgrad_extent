// Prelude: Synchronize Primitives(Fence and QueueFence(Semaphore))

use super::internals::*;
use vk;

pub trait QueueFenceInternals
{
	fn new(sem: vk::Semaphore) -> Self;
}
pub trait FenceInternals
{
	fn new(fen: vk::Fence) -> Self;
}

pub struct QueueFence { internal: vk::Semaphore }
pub struct Fence { internal: vk::Fence }

impl InternalExports<vk::Semaphore> for QueueFence { fn get_internal(&self) -> &vk::Semaphore { &self.internal } }
impl InternalExports<vk::Fence> for Fence { fn get_internal(&self) -> &vk::Fence { &self.internal } }

impl QueueFenceInternals for QueueFence
{
	fn new(sem: vk::Semaphore) -> Self { QueueFence { internal: sem } }
}
impl FenceInternals for Fence
{
	fn new(fen: vk::Fence) -> Self { Fence { internal: fen } }
}

impl Fence
{
	pub fn get_status(&self) -> Result<(), EngineError>
	{
		self.internal.get_status().map_err(EngineError::from)
	}
	pub fn clear(&self) -> Result<(), EngineError>
	{
		self.internal.reset().map_err(EngineError::from)
	}
}
