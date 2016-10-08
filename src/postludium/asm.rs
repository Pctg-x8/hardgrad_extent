// GPU Command Assembly Parser

use std::string::String;
use std::iter::FromIterator;
use itertools::Itertools;
use super::parsetools::ParseTools;

#[derive(Debug)]
pub enum LineParseError { SyntaxError }
pub type LineParseResult<T> = Result<T, LineParseError>;

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
