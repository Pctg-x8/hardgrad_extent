// Peeking with Cache

use std;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

pub trait LazyLines
{
	fn next(&mut self) -> Option<&(usize, String)>;
	fn pop(&mut self) -> Option<(usize, String)>;
}
#[allow(dead_code)]
pub struct LazyLinesStr<'a>
{
	iter: std::iter::Enumerate<std::str::Lines<'a>>, cache: Option<(usize, String)>
}
impl<'a> LazyLinesStr<'a>
{
	#[cfg(test)]
	pub fn new(source: &'a String) -> Self
	{
		LazyLinesStr { iter: source.lines().enumerate(), cache: None }
	}
}
impl<'a> LazyLines for LazyLinesStr<'a>
{
	fn next(&mut self) -> Option<&(usize, String)>
	{
		if self.cache.is_none() { self.cache = self.iter.next().map(|(u, s)| (u + 1, s.to_owned())); }
		self.cache.as_ref()
	}
	fn pop(&mut self) -> Option<(usize, String)>
	{
		if self.cache.is_none() { self.iter.next().map(|(u, s)| (u + 1, s.to_owned())) }
		else { std::mem::replace(&mut self.cache, None) }
	}
}
pub struct LazyLinesBR
{
	iter: std::iter::Enumerate<std::io::Lines<BufReader<File>>>, cache: Option<(usize, String)>
}
impl LazyLinesBR
{
	pub fn new(reader: BufReader<File>) -> Self { LazyLinesBR { iter: reader.lines().enumerate(), cache: None } }
}
impl LazyLines for LazyLinesBR
{
	fn next(&mut self) -> Option<&(usize, String)>
	{
		if self.cache.is_none() { self.cache = self.iter.next().map(|(u, s)| (u + 1, s.unwrap())); }
		self.cache.as_ref()
	}
	fn pop(&mut self) -> Option<(usize, String)>
	{
		if self.cache.is_none() { self.iter.next().map(|(u, s)| (u + 1, s.unwrap())) }
		else { std::mem::replace(&mut self.cache, None) }
	}
}
