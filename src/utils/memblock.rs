// Memory Block Manager

use std;

type BlockIndexRange = std::ops::Range<u32>;
enum FreeOperation
{
	ConcatenateBlock(usize),
	AppendBack(usize), AppendFront(usize),
	InsertNew(usize), AppendNew
}

pub struct MemoryBlockManager
{
	block_count: u32, freelist: std::collections::LinkedList<BlockIndexRange>
}
impl MemoryBlockManager
{
	pub fn new(block_count: u32) -> Self
	{
		let mut fl = std::collections::LinkedList::<BlockIndexRange>::new();
		fl.push_back(0 .. block_count - 1);

		MemoryBlockManager { block_count: block_count, freelist: fl }
	}
	pub fn free_all(&mut self) { self.freelist.clear(); self.freelist.push_back(0 .. self.block_count - 1); }
	pub fn allocate(&mut self) -> Option<u32>
	{
		match self.freelist.pop_front()
		{
			Some(v) =>
			{
				let head = v.start;
				if v.start + 1 <= v.end { self.freelist.push_front(v.start + 1 .. v.end) };
				Some(head)
			}, None => None
		}
	}
	pub fn free(&mut self, index: u32)
	{
		let search = { self.free_search(index) };
		match search
		{
			FreeOperation::ConcatenateBlock(i) =>
			{
				let mut backlist = self.freelist.split_off(i + 1);
				self.freelist.back_mut().unwrap().end = backlist.pop_front().unwrap().end;
				self.freelist.append(&mut backlist);
			},
			FreeOperation::AppendBack(i) =>
			{
				let mut iter = self.freelist.iter_mut();
				iter.nth(i).unwrap().end += 1;
			}
			FreeOperation::AppendFront(i) =>
			{
				let mut iter = self.freelist.iter_mut();
				iter.nth(i).unwrap().start -= 1;
			}
			FreeOperation::InsertNew(i) =>
			{
				let mut backlist = self.freelist.split_off(i);
				self.freelist.push_back(index .. index);
				self.freelist.append(&mut backlist);
			},
			FreeOperation::AppendNew => self.freelist.push_back(index .. index)
		}
	}

	// Privates //
	fn free_search(&self, index: u32) -> FreeOperation
	{
		fn recursive<'a, IterT>(mut iter: IterT, target: u32) -> FreeOperation
			where IterT: std::iter::Iterator<Item = (usize, &'a BlockIndexRange)>
		{
			if let Some((i, b)) = iter.next()
			{
				if target == b.end + 1
				{
					if let Some((_, b2)) = iter.next()
					{
						if target == b2.start - 1 { FreeOperation::ConcatenateBlock(i) }
						else { FreeOperation::AppendBack(i) }
					}
					else { FreeOperation::AppendBack(i) }
				}
				else if b.start > 0 && target == b.start - 1 { FreeOperation::AppendFront(i) }
				else if target < b.start { FreeOperation::InsertNew(i) }
				else { recursive(iter, target) }
			}
			else { FreeOperation::AppendNew }
		}

		recursive(self.freelist.iter().enumerate(), index)
	}
	#[allow(dead_code)]
	fn dump_freelist(&self)
	{
		println!("== Freelist ==");
		for r in self.freelist.iter()
		{
			println!("-- {} .. {}", r.start, r.end);
		}
	}
}
#[allow(dead_code)]
pub fn memory_management_test()
{
	use rand; use time;
	use rand::Rng;

	let mut mb = MemoryBlockManager::new(128);
	mb.dump_freelist();
	let mut list = [0; 16];
	for i in 0 .. 16
	{
		let b1 = mb.allocate().unwrap();
		println!("Allocated Memory Block: {}", b1);
		list[i] = b1;
	};
	mb.dump_freelist();
	let mut rng = rand::thread_rng();
	rng.shuffle(&mut list);
	for index in &list
	{
		println!("Freeing Index {}...", index);
		mb.free(*index);
		mb.dump_freelist();
	}

	println!("== Sequential Deallocation Performance ==");
	let seq_time =
	{
		let mut list = [0; 100];
		for i in 0 .. 100
		{
			list[i] = mb.allocate().unwrap();
		}
		let start_time = time::PreciseTime::now();
		for i in 0 .. 100
		{
			mb.free(list[i]);
		}
		start_time.to(time::PreciseTime::now()).num_nanoseconds().unwrap()
	};
	println!("x100 {}(avg. {}) ns", seq_time, seq_time / 100);
	mb.free_all();
	println!("== Random Deallocation Performance ==");
	let s_rand_time =
	{
		let mut rng = rand::thread_rng();
		let mut list = [0; 100];
		let mut dur_total = time::Duration::zero();
		for _ in 0 .. 10
		{
			for i in 0 .. 100
			{
				list[i] = mb.allocate().unwrap();
			}
			rng.shuffle(&mut list);
			let start_time = time::PreciseTime::now();
			for i in 0 .. 100 { mb.free(list[i]); }
			dur_total = dur_total + start_time.to(time::PreciseTime::now());
		}
		dur_total.num_nanoseconds().unwrap()
	};
	println!("x1000 {}(avg. {}) ns", s_rand_time, s_rand_time / 1000);
}