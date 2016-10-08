
use std::string::String;
use std::iter::FromIterator;
use itertools::Itertools;

#[derive(Debug)]
pub enum LineParseError { SyntaxError }
pub type LineParseResult<T> = Result<T, LineParseError>;

pub trait ParseTools
{
	type Item;
	fn skip_while<F>(self, pred: F) -> Self where F: Fn(Self::Item) -> bool;
	fn take_while<F>(self, pred: F) -> (Self, Self) where F: Fn(Self::Item) -> bool;
	fn clone_as_string(self) -> String;
}
impl<'a> ParseTools for &'a [char]
{
	type Item = char;
	fn skip_while<F>(self, pred: F) -> Self where F: Fn(char) -> bool
	{
		if !self.is_empty() && pred(self[0]) { Self::skip_while(&self[1..], pred) } else { self }
	}
	fn take_while<F>(self, pred: F) -> (Self, Self) where F: Fn(char) -> bool
	{
		fn _impl<F>(input: &[char], counter: usize, pred: F) -> usize where F: Fn(char) -> bool
		{
			if !input.is_empty() && pred(input[0]) { _impl(&input[1..], counter + 1, pred) } else { counter }
		}
		let len = _impl(self, 0, pred);
		(&self[..len], &self[len..])
	}
	fn clone_as_string(self) -> String { self.into_iter().cloned().collect() }
}

fn is_space(chr: char) -> bool { chr == ' ' || chr == '\t' }
fn not_space(chr: char) -> bool { !is_space(chr) }
pub fn parse_define(line: &[char]) -> LineParseResult<(String, String)>
{
	if line[..7] != ['.', 'd', 'e', 'f', 'i', 'n', 'e'] { Err(LineParseError::SyntaxError) }
	else if line[7] != ' ' && line[7] != '\t' { Err(LineParseError::SyntaxError) }
	else
	{
		let (name, rest) = (&line[7..]).skip_while(is_space).take_while(not_space);
		let value = rest.skip_while(is_space);
		Ok((name.clone_as_string(), value.clone_as_string()))
	}
}

#[cfg(test)]
mod test
{
	use itertools::Itertools;

	#[test] fn parse_define()
	{
		let testcase = ".define DEFAULT_BITS	2";
		assert_eq!(super::parse_define(&testcase.chars().collect_vec()).unwrap(), ("DEFAULT_BITS".to_owned(), "2".to_owned()));
	}
}
