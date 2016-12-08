#![allow(dead_code)]

use std;
use std::io::prelude::*;
use super::{
	PSDChannelImageData, UnsizedNativeFileContent, NativeFileContent,
	BinaryLoaderUtils, PSDLoadingError, PascalString
};
use std::collections::HashMap;

// Flags //
#[repr(C, packed)] pub struct PSDLayerMaskFlags(u8);
impl PSDLayerMaskFlags
{
	pub fn is_relative_pos_to_layer(&self) -> bool { (self.0 & 0x01) != 0 }
	pub fn is_layer_mask_disabled(&self) -> bool { (self.0 & 0x02) != 0 }
	pub fn is_user_mask_came_from_rendering_other_data(&self) -> bool { (self.0 & 0x08) != 0 }
	pub fn has_user_or_vector_mask_parameters(&self) -> bool { (self.0 & 0x10) != 0 }
}
#[repr(C, packed)] pub struct PSDLayerMaskParameterFlags(u8);
impl PSDLayerMaskParameterFlags
{
	pub fn has_user_mask_density(&self) -> bool { (self.0 & 0x01) != 0 }
	pub fn has_user_mask_feather(&self) -> bool { (self.0 & 0x02) != 0 }
	pub fn has_vector_mask_density(&self) -> bool { (self.0 & 0x04) != 0 }
	pub fn has_vector_mask_feather(&self) -> bool { (self.0 & 0x08) != 0 }
}
#[repr(C, packed)] pub struct PSDLayerFlags(u8);
impl PSDLayerFlags
{
	pub fn is_transparency_protected(&self) -> bool { (self.0 & 0x01) != 0 }
	pub fn is_visible(&self) -> bool { (self.0 & 0x02) != 0 }
	pub fn has_bit4(&self) -> bool { (self.0 & 0x08) != 0 }
	pub fn is_irrelevant_pixels_to_appearance(&self) -> bool { (self.0 & 0x10) != 0 }
}

// Enums //
pub enum PSDBlendModeKey
{
	PassThrough, Normal, Dissolve, Darken, Multiply, ColorBurn, LinearBurn,
	DarkerColor, Lighten, Screen, ColorDodge, LinearDodge, LighterColor, Overlay,
	SoftLight, HardLight, VividLight, LinearLight, PinLight, HardMix, Difference, Exclusion,
	Subtract, Divide, Hue, Saturation, Color, Luminosity
}
impl std::convert::From<u32> for PSDBlendModeKey
{
	fn from(v: u32) -> Self
	{
		fn optimized_compare(chars: &str) -> u32
		{
			let bytes = AsRef::<[u8]>::as_ref(chars);
			u32::from_be(unsafe { std::mem::transmute([bytes[0], bytes[1], bytes[2], bytes[3]]) })
		}
			 if v == optimized_compare("pass") { PSDBlendModeKey::PassThrough }
		else if v == optimized_compare("norm") { PSDBlendModeKey::Normal }
		else if v == optimized_compare("diss") { PSDBlendModeKey::Dissolve }
		else if v == optimized_compare("dark") { PSDBlendModeKey::Darken }
		else if v == optimized_compare("mul ") { PSDBlendModeKey::Multiply }
		else if v == optimized_compare("idiv") { PSDBlendModeKey::ColorBurn }
		else if v == optimized_compare("lbrn") { PSDBlendModeKey::LinearBurn }
		else if v == optimized_compare("dkCl") { PSDBlendModeKey::DarkerColor }
		else if v == optimized_compare("lite") { PSDBlendModeKey::Lighten }
		else if v == optimized_compare("scrn") { PSDBlendModeKey::Screen }
		else if v == optimized_compare("div ") { PSDBlendModeKey::ColorDodge }
		else if v == optimized_compare("lddg") { PSDBlendModeKey::LinearDodge }
		else if v == optimized_compare("lgCl") { PSDBlendModeKey::LighterColor }
		else if v == optimized_compare("over") { PSDBlendModeKey::Overlay }
		else if v == optimized_compare("sLit") { PSDBlendModeKey::SoftLight }
		else if v == optimized_compare("vLit") { PSDBlendModeKey::VividLight }
		else if v == optimized_compare("lLit") { PSDBlendModeKey::LinearLight }
		else if v == optimized_compare("pLit") { PSDBlendModeKey::PinLight }
		else if v == optimized_compare("hMix") { PSDBlendModeKey::HardMix }
		else if v == optimized_compare("diff") { PSDBlendModeKey::Difference }
		else if v == optimized_compare("smud") { PSDBlendModeKey::Exclusion }
		else if v == optimized_compare("fsub") { PSDBlendModeKey::Subtract }
		else if v == optimized_compare("fdiv") { PSDBlendModeKey::Divide }
		else if v == optimized_compare("hue ") { PSDBlendModeKey::Hue }
		else if v == optimized_compare("sat ") { PSDBlendModeKey::Saturation }
		else if v == optimized_compare("colr") { PSDBlendModeKey::Color }
		else if v == optimized_compare("lum ") { PSDBlendModeKey::Luminosity }
		else { unreachable!() }
	}
}
pub enum PSDLayerClipping { Base, NonBase }
impl std::convert::From<u8> for PSDLayerClipping
{
	fn from(v: u8) -> Self
	{
		match v
		{
			0 => PSDLayerClipping::Base,
			1 => PSDLayerClipping::NonBase,
			_ => unreachable!()
		}
	}
}
#[repr(u8)] pub enum PSDGlobalLayerMaskKind { ColorSelected = 0, ColorProtected = 1, UseValueStoredPerLayer = 128 }

// Primitives //
#[repr(C, packed)] #[derive(Debug)] pub struct PSDLayerRect { pub top: i32, pub left: i32, pub bottom: i32, pub right: i32 }
impl PSDLayerRect
{
	pub fn lines(&self) -> i32 { self.bottom - self.top }
	pub fn width(&self) -> i32 { self.right - self.left }
}
pub struct PSDMaskParameterPair { pub density: Option<u8>, pub feather: Option<f64> }
#[repr(C, packed)] pub struct PSDLayerBlendingRange { src: u32, dest: u32 }
#[repr(C, packed)] struct PSDChannelInfo { id: i16, length: u32 }
#[allow(dead_code)] pub struct PSDChannel { pub id: i16, pub data: PSDChannelImageData }

// Large Records //
pub enum PSDLayerMask
{
	Empty, Short(PSDLayerRect, u8),
	Full(PSDLayerRect, u8, PSDMaskParameterPair, PSDMaskParameterPair, u8, PSDLayerRect),
	WithoutParameter(PSDLayerRect, u8, u8, PSDLayerRect)
}
impl UnsizedNativeFileContent for PSDLayerMask
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Self, usize, std::fs::File), PSDLoadingError>
	{
		let structure_size = try!(fp.read_u32());
		match structure_size
		{
			0 => Ok((PSDLayerMask::Empty, 4, fp)),
			20 =>
			{
				let mask_enclosing_rect = try!(fp.read_struct::<PSDLayerRect>());
				let default_color = try!(fp.read_u8());
				let flags = try!(fp.read_struct::<PSDLayerMaskFlags>());
				if flags.has_user_or_vector_mask_parameters()
				{
					Err(PSDLoadingError::StructureSizeMismatching)
				}
				else
				{
					try!(fp.read_u16());
					Ok((PSDLayerMask::Short(mask_enclosing_rect, default_color), 24, fp))
				}
			},
			_ =>
			{
				let mask_enclosing_rect = try!(fp.read_struct::<PSDLayerRect>());
				let default_color = try!(fp.read_u8());
				let flags = try!(fp.read_struct::<PSDLayerMaskFlags>());
				if flags.has_user_or_vector_mask_parameters()
				{
					let provided_parameters = try!(fp.read_struct::<PSDLayerMaskParameterFlags>());
					let user_params = PSDMaskParameterPair
					{
						density: if provided_parameters.has_user_mask_density() { Some(try!(fp.read_u8())) } else { None },
						feather: if provided_parameters.has_user_mask_feather() { Some(try!(fp.read_f64())) } else { None }
					};
					let vector_params = PSDMaskParameterPair
					{
						density: if provided_parameters.has_vector_mask_density() { Some(try!(fp.read_u8())) } else { None },
						feather: if provided_parameters.has_vector_mask_feather() { Some(try!(fp.read_f64())) } else { None }
					};
					let _/*real_flags*/ = try!(fp.read_struct::<PSDLayerMaskFlags>());
					let real_background = try!(fp.read_u8());
					let real_enclosing_rect = try!(fp.read_struct::<PSDLayerRect>());
					Ok((PSDLayerMask::Full(mask_enclosing_rect, default_color, user_params, vector_params, real_background, real_enclosing_rect), structure_size as usize + 4, fp))
				}
				else
				{
					let _/*real_flags*/ = try!(fp.read_struct::<PSDLayerMaskFlags>());
					let real_background = try!(fp.read_u8());
					let real_enclosing_rect = try!(fp.read_struct::<PSDLayerRect>());
					Ok((PSDLayerMask::WithoutParameter(mask_enclosing_rect, default_color, real_background, real_enclosing_rect), structure_size as usize + 4, fp))
				}
			}
		}
	}
}
#[allow(dead_code)]
pub struct PSDLayerBlendingRanges
{
	pub gray: PSDLayerBlendingRange, pub channels: Vec<PSDLayerBlendingRange>
}
impl UnsizedNativeFileContent<Option<PSDLayerBlendingRanges>> for PSDLayerBlendingRanges
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Option<Self>, usize, std::fs::File), PSDLoadingError>
	{
		let structure_size = try!(fp.read_u32());
		if structure_size == 0 { Ok((None, 4, fp)) }
		else if structure_size < 8 || (structure_size & 0x07) != 0 { Err(PSDLoadingError::StructureSizeMismatching) }
		else
		{
			let gray_pair = try!(fp.read_struct::<PSDLayerBlendingRange>());
			let mut color_pairs = Vec::with_capacity((structure_size >> 3) as usize - 1);
			for _ in 0 .. (structure_size >> 3) - 1
			{
				color_pairs.push(try!(fp.read_struct::<PSDLayerBlendingRange>()));
			}
			Ok((Some(PSDLayerBlendingRanges
			{
				gray: gray_pair, channels: color_pairs
			}), structure_size as usize + 4, fp))
		}
	}
}
#[allow(dead_code)]
pub struct PSDAdditionalLayerInfo
{
	pub key_chars: [u8; 4], pub data: Vec<u8>
}
impl PSDAdditionalLayerInfo
{
	fn check_signature(sin: u32, bytes: u64) -> Result<(), PSDLoadingError>
	{
		static SIM: [u8; 4] = ['8' as u8, 'B' as u8, 'I' as u8, 'M' as u8];
		static S64: [u8; 4] =  ['8' as u8, 'B' as u8, '6' as u8, '4' as u8];

		if sin == u32::from_be(unsafe { std::mem::transmute(SIM) }) || sin == u32::from_be(unsafe { std::mem::transmute(S64) })
		{
			Ok(())
		}
		else { Err(PSDLoadingError::SignatureMismatchingF(format!("PSDAdditionalLayerInfo: {:08x} at {:x}", sin, bytes))) }
	}
}
impl UnsizedNativeFileContent for PSDAdditionalLayerInfo
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Self, usize, std::fs::File), PSDLoadingError>
	{
		try!(fp.read_u32().map_err(PSDLoadingError::from).and_then(|x| Self::check_signature(x, fp.seek(std::io::SeekFrom::Current(0)).unwrap())));
		let mut key = [0u8; 4];
		try!(fp.read_exact(&mut key));
		let data_length = try!(fp.read_u32()) as usize;
		let mut data = vec![0u8; data_length];
		try!(fp.read_exact(&mut data));

		Ok((PSDAdditionalLayerInfo
		{
			key_chars: key, data: data
		}, data_length + 12, fp))
	}
}
#[allow(dead_code)]
struct PSDLayerRecord
{
	content_rect: PSDLayerRect,
	channel_info: Vec<PSDChannelInfo>,
	blend_mode_key: PSDBlendModeKey,
	opacity: u8, clipping: PSDLayerClipping, flags: PSDLayerFlags,
	layer_masks: PSDLayerMask, blending_ranges: Option<PSDLayerBlendingRanges>,
	name: Vec<u8>, additional_infos: Vec<PSDAdditionalLayerInfo>
}
impl UnsizedNativeFileContent for PSDLayerRecord
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Self, usize, std::fs::File), PSDLoadingError>
	{
		let content_rect = try!(fp.read_struct::<PSDLayerRect>());
		let channels = try!(fp.read_u16());
		let mut channel_informations = Vec::with_capacity(channels as usize);
		for _ in 0 .. channels
		{
			channel_informations.push(try!(fp.read_struct::<PSDChannelInfo>()));
		}
		try!(fp.read_u32().map_err(PSDLoadingError::from).and_then(|sig|
			if sig != u32::from_be(unsafe { std::mem::transmute(['8' as u8, 'B' as u8, 'I' as u8, 'M' as u8]) })
			{
				Err(PSDLoadingError::SignatureMismatching("PSDLayerRec"))
			} else { Ok(()) }
		));
		let blend_mode_key = PSDBlendModeKey::from(try!(fp.read_u32()));
		let opacity = try!(fp.read_u8());
		let clipping = PSDLayerClipping::from(try!(fp.read_u8()));
		let flags = try!(fp.read_struct::<PSDLayerFlags>());
		try!(fp.read_u8());
		let extra_bytes = try!(fp.read_u32()) as usize;
		let (layer_mask_data, lmsize, frest) = try!(PSDLayerMask::read_from_file(fp));
		try!(if lmsize >= extra_bytes { Err(PSDLoadingError::StructureSizeMismatching) } else { Ok(()) });
		let (layer_blending_ranges, lbsize, frest) = try!(PSDLayerBlendingRanges::read_from_file(frest));
		try!(if (lbsize + lmsize) >= extra_bytes { Err(PSDLoadingError::StructureSizeMismatching) } else { Ok(()) });
		let (layer_name, lnsize, frest) = try!(PascalString::read_from_file(frest, 4));

		let fixed_bytes = std::mem::size_of::<PSDLayerRect>() + 2 + channels as usize * 6 + 4 + 4 + 4 + 4;
		let bytes_here = fixed_bytes + lmsize + lbsize + lnsize;

		let layerrec_size = fixed_bytes + extra_bytes;
		let mut last_bytes = extra_bytes - (bytes_here - fixed_bytes);
		let mut additional_infos = Vec::new();
		let mut frest = frest;
		while last_bytes > 0
		{
			let (ainfo, sz, fr) = try!(PSDAdditionalLayerInfo::read_from_file(frest));
			additional_infos.push(ainfo);
			last_bytes -= sz;
			frest = fr;
		}

		Ok((PSDLayerRecord
		{
			content_rect: content_rect, channel_info: channel_informations,
			blend_mode_key: blend_mode_key, opacity: opacity, clipping: clipping,
			flags: flags, layer_masks: layer_mask_data, blending_ranges: layer_blending_ranges,
			name: layer_name, additional_infos: additional_infos
		}, layerrec_size, frest))
	}
}
#[allow(dead_code)]
pub struct PSDLayer
{
	pub content_rect: PSDLayerRect,
	pub channels: HashMap<i16, PSDChannelImageData>,
	pub blend_mode_key: PSDBlendModeKey,
	pub opacity: u8, pub clipping: PSDLayerClipping, pub flags: PSDLayerFlags,
	pub layer_masks: PSDLayerMask, pub blending_ranges: Option<PSDLayerBlendingRanges>,
	pub name: Vec<u8>, pub additional_infos: Vec<PSDAdditionalLayerInfo>
}
pub enum PSDLayerInfo {}
impl UnsizedNativeFileContent<Vec<PSDLayer>> for PSDLayerInfo
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Vec<PSDLayer>, usize, std::fs::File), PSDLoadingError>
	{
		let structure_size = try!(fp.read_u32()) as usize;
		let layer_count = try!(fp.read_i16()).abs() as usize;
		let mut layer_records = Vec::with_capacity(layer_count);
		let mut frest = fp;
		let mut left_bytes = structure_size - 2;
		for _ in 0 .. layer_count
		{
			try!(if left_bytes <= 0 { Err(PSDLoadingError::StructureSizeMismatching) } else { Ok(()) });
			let (rec, rec_bytes, fr) = try!(PSDLayerRecord::read_from_file(frest));
			layer_records.push(rec);
			left_bytes -= rec_bytes;
			frest = fr;
		}
		let mut layers = Vec::with_capacity(layer_count);
		for l in layer_records.into_iter()
		{
			let mut channels = HashMap::with_capacity(l.channel_info.len());
			for ch in l.channel_info.into_iter()
			{
				try!(if left_bytes <= 0 { Err(PSDLoadingError::StructureSizeMismatching) } else { Ok(()) });
				let (rec, fr) = try!(PSDChannelImageData::read_from_file(frest, u32::from_be(ch.length) as usize));
				channels.insert(i16::from_be(ch.id), rec);
				left_bytes -= u32::from_be(ch.length) as usize;
				frest = fr;
			}
			layers.push(PSDLayer
			{
				content_rect: PSDLayerRect
				{
					left: i32::from_be(l.content_rect.left), right: i32::from_be(l.content_rect.right),
					bottom: i32::from_be(l.content_rect.bottom), top: i32::from_be(l.content_rect.top)
				}, channels: channels,
				blend_mode_key: l.blend_mode_key,
				opacity: l.opacity, clipping: l.clipping, flags: l.flags,
				layer_masks: l.layer_masks, blending_ranges: l.blending_ranges, name: l.name,
				additional_infos: l.additional_infos
			});
		}

		if left_bytes != 0 { frest.seek(std::io::SeekFrom::Current(left_bytes as i64)).unwrap(); }

		Ok((layers, structure_size + 4, frest))
	}
}
#[repr(C, packed)]
pub struct PSDGlobalLayerMaskInfo
{
	overlay_color_space: u16, color_components: [u16; 4],
	opacity: u16, kind: PSDGlobalLayerMaskKind
}
impl UnsizedNativeFileContent<Option<PSDGlobalLayerMaskInfo>> for PSDGlobalLayerMaskInfo
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Option<Self>, usize, std::fs::File), PSDLoadingError>
	{
		let section_length = try!(fp.read_u32()) as usize;
		if section_length > 0
		{
			let mut mapped_str: PSDGlobalLayerMaskInfo = unsafe { std::mem::uninitialized() };
			let mut bytes = unsafe { std::slice::from_raw_parts_mut(std::mem::transmute(&mut mapped_str), std::mem::size_of::<PSDGlobalLayerMaskInfo>()) };
			try!(fp.read_exact(&mut bytes));
			let mut filler_bytes = vec![0u8; section_length - bytes.len()];
			try!(fp.read_exact(&mut filler_bytes));
			Ok((Some(mapped_str), section_length + 4, fp))
		}
		else { Ok((None, 4, fp)) }
	}
}
#[allow(dead_code)]
pub struct PSDLayerAndMaskInfo
{
	pub layers: Vec<PSDLayer>, pub global_mask: Option<PSDGlobalLayerMaskInfo>, pub globalmask_adinfo: Vec<PSDAdditionalLayerInfo>
}
impl NativeFileContent for PSDLayerAndMaskInfo
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Self, std::fs::File), PSDLoadingError>
	{
		let section_length = try!(fp.read_u32()) as usize;
		let (layers, lr_size, frest) = try!(PSDLayerInfo::read_from_file(fp));
		try!(if section_length <= lr_size { Err(PSDLoadingError::StructureSizeMismatching) } else { Ok(()) });
		let (gm, gm_size, frest) = try!(PSDGlobalLayerMaskInfo::read_from_file(frest));
		try!(if section_length < (lr_size + gm_size) { Err(PSDLoadingError::StructureSizeMismatching) } else { Ok(()) });

		let mut left_bytes = section_length - lr_size - gm_size;
		let mut frest = frest;
		let mut gminfo = Vec::new();
		while left_bytes > 0
		{
			let (rec, recsize, fr) = try!(PSDAdditionalLayerInfo::read_from_file(frest));
			gminfo.push(rec);
			left_bytes -= recsize;
			frest = fr;
		}

		Ok((PSDLayerAndMaskInfo { layers: layers, global_mask: gm, globalmask_adinfo: gminfo }, frest))
	}
}

macro_rules! WeakEnums
{
	(pub enum $name: ident: $base_type: ty { $($vname: ident = $v: expr),* }) =>
	{
		#[allow(non_snake_case)] pub mod PSDChannelIndices
		{
			#![allow(non_upper_case_globals)]

			$(
				pub const $vname: $base_type = $v;
			)*
		}
	}
}

WeakEnums!(pub enum PSDChannelIndices: i16
{
	Alpha = -1, Red = 0, Green = 1, Blue = 2, UserLayerMask = -2
});
