// GPU Command Assembly Parser

use std;
use std::string::String;
use super::parsetools::ParseTools;
use super::lazylines::*;
use itertools::Itertools;

#[derive(Debug)]
pub enum ParseError
{
	SyntaxError, UnclosedDelimiter, UnknownCommand, MissingArgument,
	UnknownCommandType, UnknownLabelAttribute
}
pub type ParseResult<T> = Result<T, ParseError>;
trait WithLine<T>
{
	fn unwrap_on_line(self, line: usize) -> T;
}
impl<T> WithLine<T> for ParseResult<T>
{
	fn unwrap_on_line(self, line: usize) -> T
	{
		match self
		{
			Ok(t) => t,
			Err(e) => panic!("{:?} on line {}", e, line)
		}
	}
}
pub struct ParserChainData<'a, T>(ParseResult<T>, &'a [char]);
impl<'a, T> ParserChainData<'a, T>
{
	fn skip_spaces(self) -> Self
	{
		if self.0.is_ok() { ParserChainData(self.0, self.1.skip_while(is_space)) }
		else { self }
	}
	fn syntax_char(self, ch: char) -> Self
	{
		if self.0.is_err() { self }
		else if self.1.is_front_of(ch) { ParserChainData(self.0, self.1.drop(1)) }
		else { ParserChainData(Err(ParseError::SyntaxError), self.1) }
	}
	fn syntax_char_e(self, ch: char, err: ParseError) -> Self
	{
		if self.0.is_err() { self }
		else if self.1.is_front_of(ch) { ParserChainData(self.0, self.1.drop(1)) }
		else { ParserChainData(Err(err), self.1) }
	}
	fn action<U, F>(self, act: F) -> ParserChainData<'a, U> where F: FnOnce(T, &'a [char]) -> ParserChainData<'a, U>
	{
		match self.0
		{
			Ok(e) => act(e, self.1),
			Err(e) => ParserChainData(Err(e), self.1)
		}
	}
	fn reduce<U, F>(self, red: F) -> ParserChainData<'a, U> where F: FnOnce(T) -> U
	{
		match self.0
		{
			Ok(e) => ParserChainData(Ok(red(e)), self.1),
			Err(e) => ParserChainData(Err(e), self.1)
		}
	}
	fn recurse<F>(self, rec: F) -> Self where F: Fn(T, &'a [char]) -> (Self, bool)
	{
		fn recursive<'a, T, F>(i: ParserChainData<'a, T>, rec: F) -> ParserChainData<'a, T>
			where F: Fn(T, &'a [char]) -> (ParserChainData<'a, T>, bool)
		{
			match i.0
			{
				Ok(p) =>
				{
					let (ns, brk) = rec(p, i.1);
					if brk { ns } else { recursive(ns, rec) }
				},
				_ => i
			}
		}
		recursive(self, rec)
	}
}
impl<'a, T> std::convert::From<(ParseResult<T>, &'a [char])> for ParserChainData<'a, T>
{
	fn from(tup: (ParseResult<T>, &'a [char])) -> Self
	{
		ParserChainData(tup.0, tup.1)
	}
}

#[derive(Debug, PartialEq)]
pub enum ExpressionNode<'a>
{
	Number(u64), Floating(f64), ConstantRef(&'a [char]), InjectionArgRef(u64),
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
#[derive(Debug, PartialEq)]
pub enum CommandNode<'a>
{
	// Graphics Binders //
	BindPipelineState(ExpressionNode<'a>),
	BindDescriptorSet(ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>),
	BindVertexBuffer(ExpressionNode<'a>, ExpressionNode<'a>),
	BindIndexBuffer(ExpressionNode<'a>),
	PushConstant(ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>),
	// Graphics Drawers //
	Draw(ExpressionNode<'a>, ExpressionNode<'a>),
	DrawIndexed(ExpressionNode<'a>, ExpressionNode<'a>),
	// Memory Barriers //
	BufferBarrier(ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>),
	ImageBarrier(ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>),
	// Assembly Intrinsics //
	InjectCommands(&'a [char], Vec<ExpressionNode<'a>>)
}
#[derive(Debug, PartialEq)]
pub enum LabelType { Primary, Secondary, Injected }
#[derive(PartialEq, Debug)]
pub enum RenderedSubpass<'a> { Pre, Post, Sub(ExpressionNode<'a>) }
#[derive(PartialEq, Debug)]
pub enum LabelRenderedFB<'a>
{
	Swapchain(RenderedSubpass<'a>),
	Backbuffer(ExpressionNode<'a>, RenderedSubpass<'a>)
}
#[derive(Debug, PartialEq)]
pub struct LabelAttribute<'a>
{
	cmdtype: LabelType, rendered_framebuffer: LabelRenderedFB<'a>, is_transfer: bool
}

fn is_space(chr: char) -> bool { chr == ' ' || chr == '\t' }
fn split_of_ident(ch: char) -> bool
{
	is_space(ch) || ch == '\n' || ch == '#' || ch == ',' || ch == '.'
}
pub fn parse_define(line: &[char]) -> ParserChainData<(&[char], ExpressionNode)>
{
	if line[..7] != ['.', 'd', 'e', 'f', 'i', 'n', 'e']
	{
		ParserChainData(Err(ParseError::SyntaxError), line)
	}
	else if line[7] != ' ' && line[7] != '\t'
	{
		ParserChainData(Err(ParseError::SyntaxError), line)
	}
	else
	{
		let (name, rest) = (&line[7..]).skip_while(is_space).take_until(is_space);
		parse_expression(rest.skip_while(is_space)).reduce(|value| (name, value))
	}
}
pub fn parse_primary_terms(input: &[char]) -> ParserChainData<ExpressionNode>
{
	if input.is_front_of('(')
	{
		// Nested Expression
		parse_expression(input.drop(1).skip_while(is_space))
			.skip_spaces().syntax_char_e(')', ParseError::UnclosedDelimiter)
			.reduce(|ner| ner)
	}
	else if input.is_front_of('-')
	{
		// Negated
		parse_expression(input.drop(1).skip_while(is_space))
			.reduce(|ner| ExpressionNode::Negated(Box::new(ner)))
	}
	else if input.is_front_of('@')
	{
		// Injection Argument Ref
		let mut num_ipart = 0;
		let mut rest = &input[1..];
		while rest.is_front(|&c| '0' <= c && c <= '9')
		{
			num_ipart = num_ipart * 10 + rest[0].to_digit(10).unwrap() as u64;
			rest = &rest[1..];
		}
		ParserChainData(Ok(ExpressionNode::InjectionArgRef(num_ipart)), rest)
	}
	else if input.is_front(|&c| '0' <= c && c <= '9')
	{
		// Numeric
		let (ipart, rest) = input.take_while(|c| c.is_digit(10));
		if rest.is_front_of('.')
		{
			// fp
			let (fpart, rest) = rest.drop(1).take_while(|c| c.is_digit(10));
			ParserChainData(Ok(ExpressionNode::Floating((ipart.clone_as_string() + "." + fpart.clone_as_string().as_ref()).parse().unwrap())), rest)
		}
		else { ParserChainData(Ok(ExpressionNode::Number(ipart.clone_as_string().parse().unwrap())), rest) }
	}
	else
	{
		// ConstantRef
		let (cref_name, rest) = input.take_until(split_of_ident);
		ParserChainData(if cref_name.is_empty() { Err(ParseError::SyntaxError) }
		else { Ok(ExpressionNode::ConstantRef(cref_name)) }, rest)
	}
}
macro_rules!CombinateBinaryExpressionParser
{
	($name: ident = $parent_term: path { $($op: expr => $node_variant: path),* }) =>
	{
		pub fn $name(input: &[char]) -> ParserChainData<ExpressionNode>
		{
			$parent_term(input).recurse(|current_expr, rest|
			{
				let rest = rest.skip_while(is_space);
				$(
					if rest.is_front_of($op)
					{
						let rest = rest.drop(1).skip_while(is_space);
						($parent_term(rest).reduce(|rhs| $node_variant(Box::new(current_expr), Box::new(rhs))), false)
					}
				)else*
				else { (ParserChainData(Ok(current_expr), rest), true) }
			})
		}
	}
}
CombinateBinaryExpressionParser!(parse_muldiv_expr = parse_primary_terms
{
	'*' => ExpressionNode::Mul, '/' => ExpressionNode::Div, '%' => ExpressionNode::Mod
});
CombinateBinaryExpressionParser!(parse_addsub_expr = parse_muldiv_expr
{
	'+' => ExpressionNode::Add, '-' => ExpressionNode::Sub
});
CombinateBinaryExpressionParser!(parse_bit_expr = parse_addsub_expr
{
	'&' => ExpressionNode::And, '|' => ExpressionNode::Or, '^' => ExpressionNode::Xor
});
pub fn parse_expression(input: &[char]) -> ParserChainData<ExpressionNode>
{
	parse_bit_expr(input)
}

fn take_arg<'a, F>(input: &'a [char], reducer: F) -> ParserChainData<CommandNode<'a>>
	where F: FnOnce(ExpressionNode<'a>) -> CommandNode<'a>
{
	parse_expression(input).reduce(reducer)
}
fn take_2_args<'a, F>(input: &'a [char], reducer: F) -> ParserChainData<CommandNode<'a>>
	where F: FnOnce(ExpressionNode<'a>, ExpressionNode<'a>) -> CommandNode<'a>
{
	parse_expression(input)
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|a, rest| parse_expression(rest).reduce(|b| reducer(a, b)))
}
fn take_3_args<'a, F>(input: &'a [char], reducer: F) -> ParserChainData<CommandNode<'a>>
	where F: FnOnce(ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>) -> CommandNode<'a>
{
	parse_expression(input)
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|a, rest| parse_expression(rest).reduce(|b| (a, b)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b), rest| parse_expression(rest).reduce(|c| reducer(a, b, c)))
}
fn take_6_args<'a, F>(input: &'a [char], reducer: F) -> ParserChainData<CommandNode<'a>>
	where F: FnOnce(ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>) -> CommandNode<'a>
{
	parse_expression(input)
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|a, rest| parse_expression(rest).reduce(|b| (a, b)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b), rest| parse_expression(rest).reduce(|c| (a, b, c)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b, c), rest| parse_expression(rest).reduce(|d| (a, b, c, d)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b, c, d), rest| parse_expression(rest).reduce(|e| (a, b, c, d, e)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b, c, d, e), rest| parse_expression(rest).reduce(|f| reducer(a, b, c, d, e, f)))
}
fn take_8_args<'a, F>(input: &'a [char], reducer: F) -> ParserChainData<CommandNode<'a>>
	where F: FnOnce(ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>,
		ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>, ExpressionNode<'a>) -> CommandNode<'a>
{
	parse_expression(input)
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|a, rest| parse_expression(rest).reduce(|b| (a, b)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b), rest| parse_expression(rest).reduce(|c| (a, b, c)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b, c), rest| parse_expression(rest).reduce(|d| (a, b, c, d)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b, c, d), rest| parse_expression(rest).reduce(|e| (a, b, c, d, e)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b, c, d, e), rest| parse_expression(rest).reduce(|f| (a, b, c, d, e, f)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b, c, d, e, f), rest| parse_expression(rest).reduce(|g| (a, b, c, d, e, f, g)))
		.skip_spaces().syntax_char_e(',', ParseError::MissingArgument).skip_spaces()
		.action(|(a, b, c, d, e, f, g), rest| parse_expression(rest).reduce(|h| reducer(a, b, c, d, e, f, g, h)))
}
pub fn parse_command(input: &[char]) -> ParserChainData<CommandNode>
{
	let (instruction_ref, rest) = input.take_until(is_space);
	let instruction = instruction_ref.clone_as_string().to_uppercase();
	let args = rest.skip_while(is_space);
	match instruction.as_ref()
	{
		// bindps [ps_index]
		"BINDPS" | "BPS" => take_arg(args, CommandNode::BindPipelineState),
		// bindds [pl_index], [slot_index], [ds_index]
		"BINDDS" | "BDS" => take_3_args(args, CommandNode::BindDescriptorSet),
		// bindvb [slot_index], [vb_index]
		"BINDVB" | "BVB" => take_2_args(args, CommandNode::BindVertexBuffer),
		// bindib [ib_index]
		"BINDIB" | "BIB" => take_arg(args, CommandNode::BindIndexBuffer),
		// push [pl_index], [slot_index], [value]
		"PUSH" => take_3_args(args, CommandNode::PushConstant),
		// draw [vertex_count], [instance_count]
		"DRAW" => take_2_args(args, CommandNode::Draw),
		// drawindexed [vertex_count], [instance_count]
		"DRAWINSTANCED" | "IDXDRAW" | "DIX" => take_2_args(args, CommandNode::DrawIndexed),
		// bufferbarrier [srcstagemask], [dststagemask], [offs], [size], [srcusage], [dstusage]
		"BUFFERBARRIER" | "BUFBARRIER" | "BUFB" => take_6_args(args, CommandNode::BufferBarrier),
		// imagebarrier [srcstagemask], [dststagemask], [imgres], [imgsubres], [srcusage], [dstusage], [srclayout], [dstlayout]
		"IMAGEBARRIER" | "IMGBARRIER" | "IMGB" => take_8_args(args, CommandNode::ImageBarrier),
		// inject [label_name], [args]...
		"INJECT" =>
		{
			let (lname, rest) = args.take_until(split_of_ident);
			let rest = rest.skip_while(is_space);
			if rest.is_front_of(',')
			{
				ParserChainData(Ok(Vec::new()), rest.drop(1).skip_while(is_space))
					.recurse(|mut args, rest| match parse_expression(rest)
					{
						ParserChainData(Ok(arg), rest) =>
						{
							args.push(arg);
							let next = rest.skip_while(is_space);
							if next.is_front_of(',') { (ParserChainData(Ok(args), next.drop(1).skip_while(is_space)), false) }
							else { (ParserChainData(Ok(args), next), true) }
						},
						ParserChainData(Err(e), rest) => (ParserChainData(Err(e), rest), true)
					})
					.reduce(|args| CommandNode::InjectCommands(lname, args))
			}
			else { ParserChainData(Ok(CommandNode::InjectCommands(lname, Vec::new())), rest) }
		},
		_ => ParserChainData(Err(ParseError::UnknownCommand), rest)
	}
}
pub fn parse_label<'a, LinesT: LazyLines + 'a>(mut lines: LinesT) -> LabelAttribute<'a>
{
	let (mut cmd_type, mut rendered_fb, mut is_transfer) = (LabelType::Primary, None, false);
	while if let Some(&(_, ref l)) = lines.next() { l.starts_with(".") } else { false }
	{
		let (n, l) = lines.pop().unwrap();
		let chars = l.chars().skip(1).collect_vec();
		let (attr_name, rest) = (&chars).take_until(is_space);
		let rest = rest.skip_while(is_space);
		match attr_name.clone_as_string().to_uppercase().as_ref()
		{
			"TYPE" => cmd_type = match rest.take_until(is_space).0.clone_as_string().to_uppercase().as_ref()
			{
				"PRIMARY" | "PRI" | "A" => Ok(LabelType::Primary),
				"SECONDARY" | "SEC" | "B" => Ok(LabelType::Secondary),
				"INJECTED" | "INJ" | "I" => Ok(LabelType::Injected),
				_ => Err(ParseError::UnknownCommandType)
			}.unwrap_on_line(n),
			"SC_RENDERPASS" => rendered_fb = Some(LabelRenderedFB::Swapchain(match parse_expression(rest).0
			{
				Ok(ExpressionNode::ConstantRef(e)) if e.clone_as_string().to_uppercase() == "PRE" => Ok(RenderedSubpass::Pre),
				Ok(ExpressionNode::ConstantRef(e)) if e.clone_as_string().to_uppercase() == "POST" => Ok(RenderedSubpass::Post),
				Ok(e) => Ok(RenderedSubpass::Sub(e)),
				Err(e) => Err(e)
			}.unwrap_on_line(n))),
			"RENDERPASS" => rendered_fb = Some(match parse_expression(rest)
				.action(|fbi, rest| parse_expression(rest).reduce(move |si| (fbi, si)))
			{
				ParserChainData(Ok((fb_index, subpass_index)), _) => Ok(LabelRenderedFB::Backbuffer(fb_index, match subpass_index
				{
					ExpressionNode::ConstantRef(e) if e.clone_as_string().to_uppercase() == "PRE" => RenderedSubpass::Pre,
					ExpressionNode::ConstantRef(e) if e.clone_as_string().to_uppercase() == "POST" => RenderedSubpass::Post,
					e => RenderedSubpass::Sub(e),
				})),
				ParserChainData(Err(e), _) => Err(e)
			}.unwrap_on_line(n)),
			"TRANSFER" => is_transfer = true,
			_ => Err(ParseError::UnknownLabelAttribute).unwrap_on_line(n)
		}
	}
	unimplemented!();
}

#[cfg(test)]
mod test
{
	use itertools::Itertools;
	use super::super::lazylines::*;

	#[test] fn parse_define()
	{
		let testcase = ".define DEFAULT_BITS	2";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_define(&testcase_collect);
		assert_eq!(res.0.unwrap(), (&testcase_collect[8..20], super::ExpressionNode::Number(2)));
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
		let testcase = "@30";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_primary_terms(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::ExpressionNode::InjectionArgRef(30));
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
	#[test] fn parse_command()
	{
		let testcase = "bps 0";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_command(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::CommandNode::BindPipelineState(super::ExpressionNode::Number(0)));
		let testcase = "BindDS 0, 0, GLOBAL_UNIFORM_DS";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_command(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::CommandNode::BindDescriptorSet(
			super::ExpressionNode::Number(0),
			super::ExpressionNode::Number(0),
			super::ExpressionNode::ConstantRef(&testcase_collect[13..30])
		));
		let testcase = "iNjecT push_wire_colors, 0.25, 0.9875, 1.5, 1.0";
		let testcase_collect = testcase.chars().collect_vec();
		let res = super::parse_command(&testcase_collect);
		assert_eq!(res.0.unwrap(), super::CommandNode::InjectCommands(
			&testcase_collect[7..23], vec![
				super::ExpressionNode::Floating(0.25),
				super::ExpressionNode::Floating(0.9875),
				super::ExpressionNode::Floating(1.5),
				super::ExpressionNode::Floating(1.0)
			]
		));
	}
	#[test] fn parse_label()
	{
		let testcase = ".type injected
.args 4
push_wire_colors:
	push	0, 0, @0
	push	0, 1, @1
	push	0, 2, @2
	push	0, 3, @3".to_owned();
		let testcase_lines = LazyLinesStr::new(&testcase);
		let res = super::parse_label(testcase_lines);
	}
}
