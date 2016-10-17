// Expression Resolvers

use std;
use std::collections::{HashMap, HashSet, LinkedList};
use super::syntree::ExpressionNode;
use super::BuilderArguments;
use interlude::ffi::*;
use itertools::Itertools;
use postludium::parsetools::ParseTools;

pub enum ResolveError<'a>
{
	SelfRecurse(&'a [char]), UndefinedRef(&'a [char]), InjectionArgNotAllowed
}
impl<'a> std::fmt::Display for ResolveError<'a>
{
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		match self
		{
			&ResolveError::SelfRecurse(d) => write!(fmt, "{}: Self-recursed definition", d.clone_as_string()),
			&ResolveError::UndefinedRef(d) => write!(fmt, "{}: Referencing undefined definition", d.clone_as_string()),
			&ResolveError::InjectionArgNotAllowed => write!(fmt, "InjectionArg is not allowed here")
		}
	}
}
impl<'a> std::fmt::Debug for ResolveError<'a> { fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result { std::fmt::Display::fmt(self, fmt) } }
type ResolvingResult<'a, T> = Result<T, ResolveError<'a>>;

pub enum DefinitionResolver { }
impl DefinitionResolver
{
	pub fn resolve<'a>(args: &'a BuilderArguments, src: HashMap<&'a [char], ExpressionNode<'a>>) -> ResolvingResult<'a, HashMap<&'a [char], f64>>
	{
		let mut defdeps = HashMap::new();
		for (&n, x) in src.iter()
		{
			defdeps.insert(n, Self::find_deps(x));
		}
		let mut added_names = HashSet::new();
		let deflist_vl = try!(defdeps.iter().map(|(d, dd)| Self::build_deflist(&mut added_names, &defdeps, d, dd)).collect::<Result<Vec<_>, _>>());
		let deflist = deflist_vl.into_iter().flatten().collect_vec();
		// println!("{:?}", deflist);

		let mut dst = HashMap::new();
		for n in deflist
		{
			let xv = try!(Self::resolve_expression(args, &dst, src.get(&n).unwrap()));
			dst.insert(n, xv);
		}
		Ok(dst)
	}

	fn builtin_defs<'a>(name: &'a [char]) -> Option<u32>
	{
		if name == ['I', 'N', 'D', 'E', 'X', '_', 'R', 'E', 'A', 'D'] { Some(VK_ACCESS_INDEX_READ_BIT) }
		else if name == ['V', 'E', 'R', 'T', 'E', 'X', '_', 'A', 'T', 'T', 'R', 'I', 'B', 'U', 'T', 'E', '_', 'R', 'E', 'A', 'D'] { Some(VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT) }
		else if name == ['U', 'N', 'I', 'F', 'O', 'R', 'M', '_', 'R', 'E', 'A', 'D'] { Some(VK_ACCESS_UNIFORM_READ_BIT) }
		else { None }
	}
	
	fn find_deps<'a>(current: &ExpressionNode<'a>) -> Vec<&'a [char]>
	{
		match current
		{
			&ExpressionNode::ConstantRef(refn) => if Self::builtin_defs(refn).is_none() { vec![refn] } else { Vec::new() },
			&ExpressionNode::ExternalU32(ref x) | &ExpressionNode::Negated(ref x) => Self::find_deps(x),
			&ExpressionNode::Add(ref x, ref y) | &ExpressionNode::Sub(ref x, ref y) |
			&ExpressionNode::Mul(ref x, ref y) | &ExpressionNode::Div(ref x, ref y) | &ExpressionNode::Mod(ref x, ref y) |
			&ExpressionNode::And(ref x, ref y) | &ExpressionNode::Or(ref x, ref y) | &ExpressionNode::Xor(ref x, ref y) =>
			{
				Self::find_deps(x).into_iter().chain(Self::find_deps(y)).collect_vec()
			},
			_ => Vec::new()
		}
	}
	fn build_deflist<'a>(added: &mut HashSet<&'a [char]>, tree: &HashMap<&'a [char], Vec<&'a [char]>>, current: &'a [char], current_deps: &Vec<&'a [char]>) -> ResolvingResult<'a, LinkedList<&'a [char]>>
	{
		let deps = try!(current_deps.iter().map(|d| if *d == current { Err(ResolveError::SelfRecurse(d)) } else if let Some(dd) = tree.get(d) { Ok((d, dd)) }
			else { Err(ResolveError::UndefinedRef(d)) }).collect::<Result<Vec<_>, _>>());
		let mut ll = try!(deps.into_iter().map(|(d, dd)| Self::build_deflist(added, tree, d, dd)).collect::<Result<Vec<_>, _>>().map(|lv| lv.into_iter().flatten().collect::<LinkedList<_>>()));
		if !added.contains(&current)
		{
			added.insert(current);
			ll.push_back(current);
		}
		Ok(ll)
	}
	fn resolve_expression<'a>(args: &'a BuilderArguments, dst: &HashMap<&'a [char], f64>, current: &ExpressionNode<'a>) -> ResolvingResult<'a, f64>
	{
		match current
		{
			&ExpressionNode::ConstantRef(refn) => Ok(if let Some(v) = Self::builtin_defs(refn) { v as f64 } else { *dst.get(&refn).expect(&format!("unwrapping {}", refn.clone_as_string())) }),
			&ExpressionNode::Number(num) => Ok(num as f64),
			&ExpressionNode::Floating(num) => Ok(num),
			&ExpressionNode::ExternalU32(ref idx) => Self::resolve_expression(args, dst, idx).map(|v| args.external_u32s[v as usize] as f64),
			&ExpressionNode::Negated(ref x) => Self::resolve_expression(args, dst, x).map(|v| -v),
			&ExpressionNode::Add(ref l, ref r) => Ok(try!(Self::resolve_expression(args, dst, l)) + try!(Self::resolve_expression(args, dst, r))),
			&ExpressionNode::Sub(ref l, ref r) => Ok(try!(Self::resolve_expression(args, dst, l)) - try!(Self::resolve_expression(args, dst, r))),
			&ExpressionNode::Mul(ref l, ref r) => Ok(try!(Self::resolve_expression(args, dst, l)) * try!(Self::resolve_expression(args, dst, r))),
			&ExpressionNode::Div(ref l, ref r) => Ok(try!(Self::resolve_expression(args, dst, l)) / try!(Self::resolve_expression(args, dst, r))),
			&ExpressionNode::Mod(ref l, ref r) => Ok(try!(Self::resolve_expression(args, dst, l)) % try!(Self::resolve_expression(args, dst, r))),
			&ExpressionNode::And(ref l, ref r) => Ok((try!(Self::resolve_expression(args, dst, l)) as u64 & try!(Self::resolve_expression(args, dst, r)) as u64) as f64),
			&ExpressionNode::Or(ref l, ref r) => Ok((try!(Self::resolve_expression(args, dst, l)) as u64 | try!(Self::resolve_expression(args, dst, r)) as u64) as f64),
			&ExpressionNode::Xor(ref l, ref r) => Ok((try!(Self::resolve_expression(args, dst, l)) as u64 ^ try!(Self::resolve_expression(args, dst, r)) as u64) as f64),
			&ExpressionNode::InjectionArgRef(_) => Err(ResolveError::InjectionArgNotAllowed)
		}
	}
}
