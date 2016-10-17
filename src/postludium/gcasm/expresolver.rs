// Expression Resolvers

use std;
use std::collections::{HashMap, HashSet, LinkedList};
use super::syntree::*;
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

pub enum ResolvedCommand
{
	// Graphics Binders //
	BindPipelineState(u32),
	BindDescriptorSet(u32, u32, u32),
	BindVertexBuffer(u32, u32),
	BindIndexBuffer(u32),
	PushConstant(u32, u32, f64),
	// Graphics Drawers //
	Draw(u32, u32),
	DrawIndexed(u32, u32),
	// Memory Barriers //
	BufferBarrier(u32, u32, u32, i32, u32, u32),
	ImageBarrier(u32, u32, u32, u32, u32, u32, u32, u32),
	// Copying Commands //
	CopyBuffer(u32, u32, u32, i32, u32)
}

fn builtin_defs<'a>(name: &'a [char]) -> Option<u32>
{
	if name == ['I', 'N', 'D', 'E', 'X', '_', 'R', 'E', 'A', 'D'] { Some(VK_ACCESS_INDEX_READ_BIT) }
	else if name == ['V', 'E', 'R', 'T', 'E', 'X', '_', 'A', 'T', 'T', 'R', 'I', 'B', 'U', 'T', 'E', '_', 'R', 'E', 'A', 'D'] { Some(VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT) }
	else if name == ['U', 'N', 'I', 'F', 'O', 'R', 'M', '_', 'R', 'E', 'A', 'D'] { Some(VK_ACCESS_UNIFORM_READ_BIT) }
	else { None }
}
fn resolve_expression<'a>(args: &BuilderArguments, defs: &HashMap<&'a [char], f64>, current: &ExpressionNode<'a>) -> ResolvingResult<'a, f64>
{
	match current
	{
		&ExpressionNode::ConstantRef(refn) => Ok(if let Some(v) = builtin_defs(refn) { v as f64 } else { *defs.get(&refn).expect(&format!("unwrapping {}", refn.clone_as_string())) }),
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

pub enum DefinitionResolver { }
impl DefinitionResolver
{
	pub fn resolve<'a>(args: &BuilderArguments, src: HashMap<&'a [char], ExpressionNode<'a>>) -> ResolvingResult<'a, HashMap<&'a [char], f64>>
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
}

pub enum CommandResolver {}
impl CommandResolver
{
	pub fn resolve<'a>(args: &BuilderArguments, defs: &HashMap<&'a [char], f64>, mut src: HashMap<&'a [char], LabelBlock<'a>>)
	{
		while !src.is_empty()
		{
			let (name, lblk) = src.drain().take(1);
			Self::resolve_in_label(args, defs, &src, lblk.commands)
		}
	}

	fn resolve_in_label<'a>(args: &BuilderArguments, defs: &HashMap<&'a [char], f64>, labels: &HashMap<&'a [char], LabelBlock<'a>>, lb: LinkedList<CommandNode<'a>>) -> LinkedList<ResolvedCommand<'a>>
	{
		let resolved = LinkedList::new();

		for cmd in lb
		{
			match cmd
			{
				CommandNode::BindPipelineState(psid) => resolve_expression(args, defs, psid)
					.map(|psid| resolved.push_back(ResolvedCommand::BindPipelineState(psid as u32))),
				CommandNode::BindDescriptorSet(dlid, slot, dsid) => resolve_expression(args, defs, dlid)
					.and_then(|dlid| resolve_expression(args, defs, slot).map(|slot| (dlid, slot)))
					.and_then(|(dlid, slot)| resolve_expression(args, defs, dsid).map(|dsid| (dlid, slot, dsid)))
					.map(|(dlid, slot, dsid)| resolved.push_back(ResolvedCommand::BindDescriptorSet(dlid as u32, slot as u32, dsid as u32))),
				CommandNode::BindVertexBuffer(slot, vbid) => resolve_expression(args, defs, slot)
					.and_then(|slot| resolve_expression(args, defs, vbid).map(|vbid| (slot, vbid)))
					.map(|(slot, vbid)| resolved.push_back(ResolvedCommand::BindVertexBuffer(slot as u32, vbid as u32))),
				CommandNode::BindIndexBuffer(ibid) => resolve_expression(args, defs, ibid)
					.map(|ibid| resolved.push_back(ResolvedCommand::BindIndexBuffer(ibid as u32))),
				CommandNode::PushConstant(dlid, pcid, val) => resolve_expression(args, defs, dlid)
					.and_then(|dlid| resolve_expression(args, defs, pcid).map(|psid| (dlid, psid)))
					.and_then(|(dlid, psid)| resolve_expression(args, defs, val).map(|val| (dlid, psid, val)))
					.map(|(dlid, psid, val)| resolved.push_back(ResolvedCommand::PushConstant(dlid as u32, psid as u32, val))),
				CommandNode::Draw(vc, ic) => resolve_expression(args, defs, vc)
					.and_then(|vc| resolve_expression(args, defs, ic).map(|ic| (vc, ic)))
					.map(|(vc, ic)| resolved.push_back(ResolvedCommand::Draw(vc as u32, ic as u32))),
				CommandNode::DrawIndexed(vc, ic) => resolve_expression(args, defs, vc)
					.and_then(|vc| resolve_expression(args, defs, ic).map(|ic| (vc, ic)))
					.map(|(vc, ic)| resolved.push_back(ResolvedCommand::DrawIndexed(vc as u32, ic as u32))),
				CommandNode::BufferBarrier(sps, dps, offs, size, smu, dmu) => resolve_expression(args, defs, sps)
					.and_then(|sps| resolve_expression(args, defs, dps).map(move |dps| (sps, dps)))
					.and_then(|(sps, dps)| resolve_expression(args, defs, offs).map(move |offs| (sps, dps, offs)))
					.and_then(|(sps, dps, offs)| resolve_expression(args, defs, size).map(move |size| (sps, dps, offs, size)))
					.and_then(|(sps, dps, offs, size)| resolve_expression(args, defs, smu).map(move |smu| (sps, dps, offs, size, smu)))
					.and_then(|(sps, dps, offs, size, smu)| resolve_expression(args, defs, dmu).map(move |dmu| (sps, dps, offs, size, smu, dmu)))
					.map(|(sps, dps, offs, size, smu, dmu)| resolved.push_back(ResolvedCommand::BufferBarrier(sps as u32, dps as u32, offs as u32, size as i32, smu as u32, dmu as u32))),
				CommandNode::ImageBarrier(sps, dps, img, ims, smu, dmu, sil, dil) => resolve_expression(args, defs, sps)
					.and_then(|sps| resolve_expression(args, defs, dps).map(move |dps| (sps, dps)))
					.and_then(|(sps, dps)| resolve_expression(args, defs, img).map(move |img| (sps, dps, img)))
					.and_then(|(sps, dps, img)| resolve_expression(args, defs, ims).map(move |ims| (sps, dps, img, ims)))
					.and_then(|(sps, dps, img, ims)| resolve_expression(args, defs, smu).map(move |smu| (sps, dps, img, ims, smu)))
					.and_then(|(sps, dps, img, ims, smu)| resolve_expression(args, defs, dmu).map(move |dmu| (sps, dps, img, ims, smu, dmu)))
					.and_then(|(sps, dps, img, ims, smu, dmu)| resolve_expression(args, defs, sil).map(move |sil| (sps, dps, img, ims, smu, dmu, sil)))
					.and_then(|(sps, dps, img, ims, smu, dmu, sil)| resolve_expression(args, defs, dil).map(move |dil| (sps, dps, img, ims, smu, dmu, sil, dil)))
					.map(|(sps, dps, img, ims, smu, dmu, sil, dil)| resolved.push_back(ResolvedCommand::ImageBarrier(sps as u32, dps as u32, img as u32, ims as u32, smu as u32, dmu as u32, sil as u32, dil as u32))),
				CommandNode::CopyBuffer(sbid, dbid, sof, size, dof) => resolve_expression(args, defs, sbid)
					.and_then(|sbid| resolve_expression(args, defs, dbid).map(move |dbid| (sbid, dbid)))
					.and_then(|(sbid, dbid)| resolve_expression(args, defs, sof).map(move |sof| (sbid, dbid, sof)))
					.and_then(|(sbid, dbid, sof)| resolve_expression(args, defs, size).map(move |size| (sbid, dbid, sof, size)))
					.and_then(|(sbid, dbid, sof, size)| resolve_expression(args, defs, dof).map(move |dof| (sbid, dbid, sof, size, dof)))
					.map(|(sbid, dbid, sof, size, dof)| resolved.push_back(ResolvedCommand::CopyBuffer(sbid as u32, dbid as u32, sof as u32, size as i32, dof as u32))),
				CommandNode::InjectCommands(name, args) => resolved.append(resolve_for_injection(args, defs, args, labels, labels.get(name)))
			}
		}
	}
	fn resolve_for_injection<'a>(args: &BuilderArguments, defs: &HashMap<&'a [char], f64>, args: Vec<f64>, labels: &HashMap<&'a [char], LabelBlock<'a>>, lb: &LinkedList<CommandNode<'a>>) -> LinkedList<ResolvedCommand<'a>>
	{
		let resolved = LinkedList::new();

		for cmd in lb
		{
			match cmd
			{
				CommandNode::BindPipelineState(psid) => resolve_expression(args, defs, psid)
					.map(|psid| resolved.push_back(ResolvedCommand::BindPipelineState(psid as u32))),
				CommandNode::BindDescriptorSet(dlid, slot, dsid) => resolve_expression(args, defs, dlid)
					.and_then(|dlid| resolve_expression(args, defs, slot).map(|slot| (dlid, slot)))
					.and_then(|(dlid, slot)| resolve_expression(args, defs, dsid).map(|dsid| (dlid, slot, dsid)))
					.map(|(dlid, slot, dsid)| resolved.push_back(ResolvedCommand::BindDescriptorSet(dlid as u32, slot as u32, dsid as u32))),
				CommandNode::BindVertexBuffer(slot, vbid) => resolve_expression(args, defs, slot)
					.and_then(|slot| resolve_expression(args, defs, vbid).map(|vbid| (slot, vbid)))
					.map(|(slot, vbid)| resolved.push_back(ResolvedCommand::BindVertexBuffer(slot as u32, vbid as u32))),
				CommandNode::BindIndexBuffer(ibid) => resolve_expression(args, defs, ibid)
					.map(|ibid| resolved.push_back(ResolvedCommand::BindIndexBuffer(ibid as u32))),
				CommandNode::PushConstant(dlid, pcid, val) => resolve_expression(args, defs, dlid)
					.and_then(|dlid| resolve_expression(args, defs, pcid).map(|psid| (dlid, psid)))
					.and_then(|(dlid, psid)| resolve_expression(args, defs, val).map(|val| (dlid, psid, val)))
					.map(|(dlid, psid, val)| resolved.push_back(ResolvedCommand::PushConstant(dlid as u32, psid as u32, val))),
				CommandNode::Draw(vc, ic) => resolve_expression(args, defs, vc)
					.and_then(|vc| resolve_expression(args, defs, ic).map(|ic| (vc, ic)))
					.map(|(vc, ic)| resolved.push_back(ResolvedCommand::Draw(vc as u32, ic as u32))),
				CommandNode::DrawIndexed(vc, ic) => resolve_expression(args, defs, vc)
					.and_then(|vc| resolve_expression(args, defs, ic).map(|ic| (vc, ic)))
					.map(|(vc, ic)| resolved.push_back(ResolvedCommand::DrawIndexed(vc as u32, ic as u32))),
				CommandNode::BufferBarrier(sps, dps, offs, size, smu, dmu) => resolve_expression(args, defs, sps)
					.and_then(|sps| resolve_expression(args, defs, dps).map(move |dps| (sps, dps)))
					.and_then(|(sps, dps)| resolve_expression(args, defs, offs).map(move |offs| (sps, dps, offs)))
					.and_then(|(sps, dps, offs)| resolve_expression(args, defs, size).map(move |size| (sps, dps, offs, size)))
					.and_then(|(sps, dps, offs, size)| resolve_expression(args, defs, smu).map(move |smu| (sps, dps, offs, size, smu)))
					.and_then(|(sps, dps, offs, size, smu)| resolve_expression(args, defs, dmu).map(move |dmu| (sps, dps, offs, size, smu, dmu)))
					.map(|(sps, dps, offs, size, smu, dmu)| resolved.push_back(ResolvedCommand::BufferBarrier(sps as u32, dps as u32, offs as u32, size as i32, smu as u32, dmu as u32))),
				CommandNode::ImageBarrier(sps, dps, img, ims, smu, dmu, sil, dil) => resolve_expression(args, defs, sps)
					.and_then(|sps| resolve_expression(args, defs, dps).map(move |dps| (sps, dps)))
					.and_then(|(sps, dps)| resolve_expression(args, defs, img).map(move |img| (sps, dps, img)))
					.and_then(|(sps, dps, img)| resolve_expression(args, defs, ims).map(move |ims| (sps, dps, img, ims)))
					.and_then(|(sps, dps, img, ims)| resolve_expression(args, defs, smu).map(move |smu| (sps, dps, img, ims, smu)))
					.and_then(|(sps, dps, img, ims, smu)| resolve_expression(args, defs, dmu).map(move |dmu| (sps, dps, img, ims, smu, dmu)))
					.and_then(|(sps, dps, img, ims, smu, dmu)| resolve_expression(args, defs, sil).map(move |sil| (sps, dps, img, ims, smu, dmu, sil)))
					.and_then(|(sps, dps, img, ims, smu, dmu, sil)| resolve_expression(args, defs, dil).map(move |dil| (sps, dps, img, ims, smu, dmu, sil, dil)))
					.map(|(sps, dps, img, ims, smu, dmu, sil, dil)| resolved.push_back(ResolvedCommand::ImageBarrier(sps as u32, dps as u32, img as u32, ims as u32, smu as u32, dmu as u32, sil as u32, dil as u32))),
				CommandNode::CopyBuffer(sbid, dbid, sof, size, dof) => resolve_expression(args, defs, sbid)
					.and_then(|sbid| resolve_expression(args, defs, dbid).map(move |dbid| (sbid, dbid)))
					.and_then(|(sbid, dbid)| resolve_expression(args, defs, sof).map(move |sof| (sbid, dbid, sof)))
					.and_then(|(sbid, dbid, sof)| resolve_expression(args, defs, size).map(move |size| (sbid, dbid, sof, size)))
					.and_then(|(sbid, dbid, sof, size)| resolve_expression(args, defs, dof).map(move |dof| (sbid, dbid, sof, size, dof)))
					.map(|(sbid, dbid, sof, size, dof)| resolved.push_back(ResolvedCommand::CopyBuffer(sbid as u32, dbid as u32, sof as u32, size as i32, dof as u32))),
				CommandNode::InjectCommands(name, args) =>
			}
		}
	}
}
