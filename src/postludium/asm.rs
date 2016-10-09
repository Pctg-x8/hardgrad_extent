// GPU Command Assembly Parser

use std::string::String;
use super::parsetools::ParseTools;

#[derive(Debug)]
pub enum ParseError { SyntaxError, DelimiterNotEnclosed }
pub type ParseResult<T> = Result<T, ParseError>;
#[derive(Debug, PartialEq)]
pub enum ExpressionNode<'a>
{
	Number(u64), Floating(f64), ConstantRef(&'a [char]),
	Negated(Box<ExpressionNode<'a>>),
	Add(Box<ExpressionNode<'a>>, Box<ExpressionNode<'a>>),
	Sub(Box<ExpressionNode<'a>>, Box<ExpressionNode<'a>>),
	Mul(Box<ExpressionNode<'a>>, Box<ExpressionNode<'a>>),
	Div(Box<ExpressionNode<'a>>, Box<ExpressionNode<'a>>),
	Mod(Box<ExpressionNode<'a>>, Box<ExpressionNode<'a>>),
	And(Box<ExpressionNode<'a>>, Box<ExpressionNode<'a>>),
	Or(Box<ExpressionNode<'a>>, Box<ExpressionNode<'a>>),
	Xor(Box<ExpressionNode<'a>>, Box<ExpressionNode<'a>>)
}

fn is_space(chr: char) -> bool { chr == ' ' || chr == '\t' }
fn split_of_ident(ch: char) -> bool
{
	is_space(ch) || ch == '\n' || ch == '#' || ch == ',' || ch == '.'
}
pub fn parse_define(line: &[char]) -> ParseResult<(String, String)>
{
	if line[..7] != ['.', 'd', 'e', 'f', 'i', 'n', 'e'] { Err(ParseError::SyntaxError) }
	else if line[7] != ' ' && line[7] != '\t' { Err(ParseError::SyntaxError) }
	else
	{
		let (name, rest) = (&line[7..]).skip_while(is_space).take_until(is_space);
		let value = rest.skip_while(is_space);
		Ok((name.clone_as_string(), value.clone_as_string()))
	}
}
pub fn parse_primary_terms(input: &[char]) -> (ParseResult<ExpressionNode>, &[char])
{
	if input.is_front_of('(')
	{
		// Nested Expression
		let (inner, rest) = parse_expression(input.drop(1).skip_while(is_space));
		match inner
		{
			Err(_) => (inner, rest),
			Ok(_) =>
			{
				let rest = rest.skip_while(is_space);
				if rest.is_front_of(')') { (inner, &rest[1..]) }
				else { (Err(ParseError::DelimiterNotEnclosed), rest) }
			}
		}
	}
	else if input.is_front_of('-')
	{
		// Negated
		let (inner, rest) = parse_primary_terms(input.drop(1).skip_while(is_space));
		(inner.map(|ner| ExpressionNode::Negated(Box::new(ner))), rest)
	}
	else if input.is_front(|&c| '0' <= c && c <= '9')
	{
		// Numeric
		let mut num_ipart = input[0] as u64 - '0' as u64;
		let mut rest = &input[1..];
		while rest.is_front(|&c| '0' <= c && c <= '9')
		{
			num_ipart = num_ipart * 10 + (rest[0] as u64 - '0' as u64);
			rest = &rest[1..];
		}
		(if rest.is_front_of('.')
		{
			// fp
			let mut num_fpart = 0.0f64;
			let mut divs_fpart = 10.0f64;
			rest = &rest[1..];
			while rest.is_front(|&c| '0' <= c && c <= '9')
			{
				num_fpart += (rest[0] as u64 - '0' as u64) as f64 / divs_fpart;
				divs_fpart *= 10.0;
				rest = &rest[1..];
			}
			Ok(ExpressionNode::Floating(num_ipart as f64 + num_fpart))
		}
		else { Ok(ExpressionNode::Number(num_ipart)) }, rest)
	}
	else
	{
		// ConstantRef
		let (cref_name, rest) = input.take_until(split_of_ident);
		(if cref_name.is_empty() { Err(ParseError::SyntaxError) }
		else { Ok(ExpressionNode::ConstantRef(cref_name)) }, rest)
	}
}
pub fn parse_muldiv_expr(input: &[char]) -> (ParseResult<ExpressionNode>, &[char])
{
	let (lhs, rest) = parse_primary_terms(input);
	match lhs
	{
		Err(_) => (lhs, rest),
		Ok(e) =>
		{
			fn recursive<'a>(current_expr: ExpressionNode<'a>, rest: &'a [char])
				-> (ParseResult<ExpressionNode<'a>>, &'a [char])
			{
				let rest = rest.skip_while(is_space);
				if rest.is_front_of('*')
				{
					let rest = rest.drop(1).skip_while(is_space);
					let (rhs, rest_r) = parse_primary_terms(rest);
					match rhs
					{
						Err(_) => (rhs, rest_r),
						Ok(er) => recursive(ExpressionNode::Mul(Box::new(current_expr), Box::new(er)), rest_r)
					}
				}
				else if rest.is_front_of('/')
				{
					let rest = rest.drop(1).skip_while(is_space);
					let (rhs, rest_r) = parse_primary_terms(rest);
					match rhs
					{
						Err(_) => (rhs, rest_r),
						Ok(er) => recursive(ExpressionNode::Div(Box::new(current_expr), Box::new(er)), rest_r)
					}
				}
				else if rest.is_front_of('%')
				{
					let rest = rest.drop(1).skip_while(is_space);
					let (rhs, rest_r) = parse_primary_terms(rest);
					match rhs
					{
						Err(_) => (rhs, rest_r),
						Ok(er) => recursive(ExpressionNode::Mod(Box::new(current_expr), Box::new(er)), rest_r)
					}
				}
				else { (Ok(current_expr), rest) }
			}
			recursive(e, rest)
		}
	}
}
pub fn parse_addsub_expr(input: &[char]) -> (ParseResult<ExpressionNode>, &[char])
{
	let (lhs, rest) = parse_muldiv_expr(input);
	match lhs
	{
		Err(_) => (lhs, rest),
		Ok(e) =>
		{
			fn recursive<'a>(current_expr: ExpressionNode<'a>, rest: &'a [char])
				-> (ParseResult<ExpressionNode<'a>>, &'a [char])
			{
				let rest = rest.skip_while(is_space);
				if rest.is_front_of('+')
				{
					let rest = rest.drop(1).skip_while(is_space);
					let (rhs, rest_r) = parse_muldiv_expr(rest);
					match rhs
					{
						Err(_) => (rhs, rest_r),
						Ok(er) => recursive(ExpressionNode::Add(Box::new(current_expr), Box::new(er)), rest_r)
					}
				}
				else if rest.is_front_of('-')
				{
					let rest = rest.drop(1).skip_while(is_space);
					let (rhs, rest_r) = parse_muldiv_expr(rest);
					match rhs
					{
						Err(_) => (rhs, rest_r),
						Ok(er) => recursive(ExpressionNode::Sub(Box::new(current_expr), Box::new(er)), rest_r)
					}
				}
				else { (Ok(current_expr), rest) }
			}
			recursive(e, rest)
		}
	}
}
pub fn parse_bit_expr(input: &[char]) -> (ParseResult<ExpressionNode>, &[char])
{
	let (lhs, rest) = parse_addsub_expr(input);
	match lhs
	{
		Err(_) => (lhs, rest),
		Ok(e) =>
		{
			fn recursive<'a>(current_expr: ExpressionNode<'a>, rest: &'a [char])
				-> (ParseResult<ExpressionNode<'a>>, &'a [char])
			{
				let rest = rest.skip_while(is_space);
				if rest.is_front_of('&')
				{
					let rest = rest.drop(1).skip_while(is_space);
					let (rhs, rest_r) = parse_addsub_expr(rest);
					match rhs
					{
						Err(_) => (rhs, rest_r),
						Ok(er) => recursive(ExpressionNode::And(Box::new(current_expr), Box::new(er)), rest_r)
					}
				}
				else if rest.is_front_of('|')
				{
					let rest = rest.drop(1).skip_while(is_space);
					let (rhs, rest_r) = parse_addsub_expr(rest);
					match rhs
					{
						Err(_) => (rhs, rest_r),
						Ok(er) => recursive(ExpressionNode::Or(Box::new(current_expr), Box::new(er)), rest_r)
					}
				}
				else if rest.is_front_of('^')
				{
					let rest = rest.drop(1).skip_while(is_space);
					let (rhs, rest_r) = parse_addsub_expr(rest);
					match rhs
					{
						Err(_) => (rhs, rest_r),
						Ok(er) => recursive(ExpressionNode::Xor(Box::new(current_expr), Box::new(er)), rest_r)
					}
				}
				else { (Ok(current_expr), rest) }
			}
			recursive(e, rest)
		}
	}
}
pub fn parse_expression(input: &[char]) -> (ParseResult<ExpressionNode>, &[char])
{
	parse_bit_expr(input)
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
	#[test] fn parse_primary_terms()
	{
		let testcase = "PS_RENDER_BACKGROUND,";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_primary_terms(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::ExpressionNode::ConstantRef(&testcase_collect[..20]));
		assert_eq!(res.1, &testcase_collect[20..]);
		let testcase = "2.0";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_primary_terms(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::ExpressionNode::Floating(2.0));
		let testcase = "-6";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_primary_terms(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::ExpressionNode::Negated(Box::new(super::ExpressionNode::Number(6))));
	}
	#[test] fn parse_expression()
	{
		let testcase = "2 + 3";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_expression(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::ExpressionNode::Add(Box::new(super::ExpressionNode::Number(2)), Box::new(super::ExpressionNode::Number(3))));
		let testcase = "TOP | TRANSFER + 2, ";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_expression(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::ExpressionNode::Or(
			Box::new(super::ExpressionNode::ConstantRef(&testcase_collect[..3])),
			Box::new(super::ExpressionNode::Add(
				Box::new(super::ExpressionNode::ConstantRef(&testcase_collect[6..14])),
				Box::new(super::ExpressionNode::Number(2))
			))
		));
		assert_eq!(res.1, &testcase_collect[18..]);
	}
}
