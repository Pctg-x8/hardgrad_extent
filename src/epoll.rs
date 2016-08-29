// epoll syscalls

use {prelude, std, libc};
use evdev::*;
use std::os::unix::io::AsRawFd;
use std::cell::RefCell;
use std::rc::Rc;

mod ffi
{
/* Copyright (C) 2002-2016 Free Software Foundation, Inc.
   Original file(sys/epoll.h) is part of the GNU C Library.

   The GNU C Library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public
   License as published by the Free Software Foundation; either
   version 2.1 of the License, or (at your option) any later version.

   The GNU C Library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with the GNU C Library; if not, see
   <http://www.gnu.org/licenses/>.  */

	#![allow(dead_code, non_camel_case_types)]
	use libc::{c_int, c_void};

	pub const EPOLL_CTL_ADD: i32 = 1;
	pub const EPOLL_CTL_DEL: i32 = 2;
	pub const EPOLL_CTL_MOD: i32 = 3;

	// EPOLL_EVENTS //
	pub const EPOLLIN: u32 = 0x001;
	pub const EPOLLPRI: u32 = 0x002;

	pub type epoll_data_t = *mut c_void;		// union value
	#[repr(C)] #[derive(Clone)]
	pub struct epoll_event
	{
		pub events: u32, pub data: epoll_data_t
	}

	extern
	{
		pub fn epoll_create1(flags: c_int) -> c_int;
		pub fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut epoll_event) -> c_int;
		pub fn epoll_wait(epfd: c_int, events: *mut epoll_event, max_events: c_int, timeout: c_int) -> c_int;
	}
}

pub struct EPoll
{
	internal_fd: libc::c_int,
	targets_ref: Vec<Rc<RefCell<EventDevice>>>,
	event_receiver: Vec<ffi::epoll_event>
}
impl EPoll
{
	pub fn new(targets: &[Rc<RefCell<EventDevice>>]) -> Result<Self, prelude::EngineError>
	{
		let ifd = unsafe { ffi::epoll_create1(0) };
		if ifd == -1 { Err(prelude::EngineError::from(std::io::Error::last_os_error())) }
		else
		{
			for (n, f) in targets.iter().enumerate()
			{
				let mut event_data = ffi::epoll_event
				{
					events: ffi::EPOLLIN, data: unsafe { std::mem::transmute(n) }
				};
				unsafe { ffi::epoll_ctl(ifd, ffi::EPOLL_CTL_ADD, f.borrow().as_raw_fd(), &mut event_data) };
			}
			Ok(EPoll
			{
				event_receiver: vec![ffi::epoll_event { events: 0, data: std::ptr::null_mut() }; targets.len()],
				internal_fd: ifd, targets_ref: Vec::from(targets)
			})
		}
	}
	pub fn wait(&mut self) -> Result<Vec<Rc<RefCell<EventDevice>>>, prelude::EngineError>
	{
		let epres = unsafe { ffi::epoll_wait(self.internal_fd, self.event_receiver.as_mut_ptr(), self.event_receiver.len() as libc::c_int, -1) };
		if epres == -1 { Err(prelude::EngineError::from(std::io::Error::last_os_error())) } else
		{
			Ok(self.event_receiver[..epres as usize].iter().map(|e|
				self.targets_ref[unsafe { std::mem::transmute::<_, u64>(e.data) } as usize].clone()
			).collect())
		}
	}
	#[allow(dead_code)]
	pub fn check(&mut self) -> Result<Vec<Rc<RefCell<EventDevice>>>, prelude::EngineError>
	{
		let epres = unsafe { ffi::epoll_wait(self.internal_fd, self.event_receiver.as_mut_ptr(), self.event_receiver.len() as libc::c_int, 0) };
		if epres == -1 { Err(prelude::EngineError::from(std::io::Error::last_os_error())) } else
		{
			Ok(self.event_receiver[..epres as usize].iter().map(|e|
				self.targets_ref[unsafe { std::mem::transmute::<_, u64>(e.data) } as usize].clone()
			).collect())
		}
	}
}
