// epoll syscalls

use {prelude, epoll};
use evdev::*;
use std::os::unix::io::AsRawFd;
use std::cell::RefCell;
use std::rc::Rc;

pub struct EPoll
{
	internal: epoll::EpollInstance,
	targets_ref: Vec<Rc<RefCell<EventDevice>>>
}
impl EPoll
{
	pub fn new(targets: Vec<Rc<RefCell<EventDevice>>>) -> Result<Self, prelude::EngineError>
	{
		let mut ifd = try!(epoll::EpollInstance::new());
		for (n, f) in targets.iter().enumerate()
		{
			try!(ifd.add_interest(epoll::Interest::new(f.borrow().as_raw_fd(), epoll::EPOLLIN, n as u64)));
		}
		Ok(EPoll
		{
			targets_ref: targets, internal: ifd
		})
	}
	pub fn wait(&mut self) -> Result<Vec<Rc<RefCell<EventDevice>>>, prelude::EngineError>
	{
		self.internal.wait(-1, self.targets_ref.len()).map(|interests|
			interests.into_iter().map(|e| self.targets_ref[e.data() as usize].clone()).collect())
			.map_err(prelude::EngineError::from)
	}
	#[allow(dead_code)]
	pub fn check(&mut self) -> Result<Vec<Rc<RefCell<EventDevice>>>, prelude::EngineError>
	{
		self.internal.wait(0, self.targets_ref.len()).map(|interests|
			interests.into_iter().map(|e| self.targets_ref[e.data() as usize].clone()).collect())
			.map_err(prelude::EngineError::from)
	}
}
