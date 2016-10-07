
use std::string::String;
use std::iter::FromIterator;
use itertools::Itertools;

#[derive(Debug)]
pub enum LineParseError { SyntaxError }
pub type LineParseResult<T> = Result<T, LineParseError>;

fn is_space(chr: char) -> bool { chr == ' ' || chr == '\t' }
fn not_space(chr: char) -> bool { !is_space(chr) }
fn skip_while<F>(input: &[char], pred: F) -> &[char] where F: Fn(char) -> bool { if !input.is_empty() && pred(input[0]) { skip_while(&input[1..], pred) } else { input } }
fn take_while<F>(input: &[char], pred: F) -> (&[char], &[char]) where F: Fn(char) -> bool { let len = take_while_impl(input, 0, pred); (&input[..len], &input[len..]) }
fn take_while_impl<F>(input: &[char], len: usize, pred: F) -> usize where F: Fn(char) -> bool { if !input.is_empty() && pred(input[0]) { take_while_impl(&input[1..], len + 1, pred) } else { len } }
pub fn parse_define(line: &[char]) -> LineParseResult<(String, String)>
{
	if line[..7] != ['.', 'd', 'e', 'f', 'i', 'n', 'e'] { Err(LineParseError::SyntaxError) }
	else if line[8] != ' ' && line[8] != '\t' { Err(LineParseError::SyntaxError) }
	else
	{
		let rest = skip_while(&line[7..], is_space);
		let (name, rest) = take_while(rest, not_space);
		let name = name.iter().cloned().collect::<String>();
		let value = skip_while(rest, is_space).iter().cloned().collect::<String>();
		Ok((name, value))
	}
}

#[cfg(test)]
mod test
{
	use itertools::Itertools;

	#[test] fn parse_define()
	{
		let testcase = ".define DEFAULT_BITS	2";
		assert_eq!(super::parse_define(&testcase.chars().collect_vec()).unwrap(), ("DEFAULT_BITS", "2"));
	}
}
