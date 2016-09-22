use super::{PSDLoadingError, UnsizedNativeFileContent, NativeFileContent, BinaryLoaderUtils, PascalString};
use std;
use std::io::prelude::*;

/// PSDImageResourceBlock(Variable Structure)
/// Signature: [u8; 4], ID: u16, NameLength: u16, Name: [u8], ResourceLength: u32, ResourceData: ?
#[allow(dead_code)]
pub struct PSDImageResource
{
	pub id: PSDImageResourceID, pub name: Vec<u8>, data: Vec<u8>
}
#[derive(PartialEq, Eq, Debug)]
pub enum PSDImageResourceID
{
	NumOfChannelsRowsColumnsDepthAndMode,
	MacintoshPrintManagerInfoRecord, MacintoshPageFormatInfo,
	IndexedColorTable, ResolutionInfo, AlphaChannelName, DisplayInfo, Caption,
	BorderInfo, BackgroundColor, PrintFlags, GrayscaleHalftoneInfo, ColorHalftoneInfo,
	DuotoneHalftoneInfo, GrayscaleTransferFunc, ColorTransferFunc, DuotoneTransferFunc,
	DuotoneImageInfo, EffectiveBlackOrWhiteValues, EPSOptions, QuickMaskInfo, LayerStateInfo,
	WorkingPath, LayersGroupInfo, IPTCNAARecord, ImageModeForRawFormatFiles, JPEGQualityData,
	// Photoshop 4.0 //
	GridAndGuidesInfo, ThumbnailResource, CopyrightFlag, URL,
	// Photoshop 5.0 //
	ThumbnailResource2, GlobalAngle, ColorSamplers, ICCProfile, Watermark, ICCUntaggedProfile,
	EffectVisibility, SpotHalftone, IDSeedNumber, AlphaNameUnicode,
	// Photoshop 6.0 //
	IndexedColorTableCount, TransparencyIndex, GlobalAltitude, Slices, WorkflowURL,
	JumpToXPEP, AlphaIdentifiers, URLList, VersionInfo,
	// Photoshop 7.0 //
	EXIFData1, EXIFData3, XMPMetadata, CaptionDigest, PrintScale,
	// Photoshop CS //
	PixelAspectRatio, LayerComps, AlternateDuotoneColors, AlternateSpotColors,
	// Photoshop CS2 //
	LayerSelectionID, HDRToningInformation, PrintInfo, LayerGroupEnabledID,
	// Photoshop CS3 //
	ColorSamplersResource, MeasurementScale, TimelineInfo, SheetDisclosure, DisplayInfoFP, OnionSkins,
	// Photoshop CS4 //
	CountInfo,
	// Photoshop CS5 //
	PrintInfo5, PrintStyle, MacintoshNSPrintInfoStruct, WindowsDevModeStruct,
	// Photoshop CS6 //
	AutoSavePath, AutoSaveFormat,
	// Photoshop CC //
	PathSelectionState, PathInformation(u16), NameOfClippingPath,
	OriginPathInfo, PluginResources(u16),
	ImageReadyVariables, ImageReadyDataSets, ImageReadyDefaultSelectedState,
	ImageReady7RolloverExpandedState, ImageReadyRolloverExpandedState,
	ImageReadySaveLayerSettings, ImageReadyVersion, LightroomWorkflow,
	PrintFlags2,
	Unknown(u16)		// Followed target's endian
}
impl std::convert::From<u16> for PSDImageResourceID
{
	fn from(v: u16) -> Self
	{
		match v
		{
			1000 => PSDImageResourceID::NumOfChannelsRowsColumnsDepthAndMode,
			1001 => PSDImageResourceID::MacintoshPrintManagerInfoRecord,
			1002 => PSDImageResourceID::MacintoshPageFormatInfo,
			1003 => PSDImageResourceID::IndexedColorTable,
			1005 => PSDImageResourceID::ResolutionInfo,
			1006 => PSDImageResourceID::AlphaChannelName,
			1007 => PSDImageResourceID::DisplayInfo,
			1008 => PSDImageResourceID::Caption,
			1009 => PSDImageResourceID::BorderInfo,
			1010 => PSDImageResourceID::BackgroundColor,
			1011 => PSDImageResourceID::PrintFlags,
			1012 => PSDImageResourceID::GrayscaleHalftoneInfo,
			1013 => PSDImageResourceID::ColorHalftoneInfo,
			1014 => PSDImageResourceID::DuotoneHalftoneInfo,
			1015 => PSDImageResourceID::GrayscaleTransferFunc,
			1016 => PSDImageResourceID::ColorTransferFunc,
			1017 => PSDImageResourceID::DuotoneTransferFunc,
			1018 => PSDImageResourceID::DuotoneImageInfo,
			1019 => PSDImageResourceID::EffectiveBlackOrWhiteValues,
			1021 => PSDImageResourceID::EPSOptions,
			1022 => PSDImageResourceID::QuickMaskInfo,
			1024 => PSDImageResourceID::LayerStateInfo,
			1025 => PSDImageResourceID::WorkingPath,
			1026 => PSDImageResourceID::LayersGroupInfo,
			1028 => PSDImageResourceID::IPTCNAARecord,
			1029 => PSDImageResourceID::ImageModeForRawFormatFiles,
			1030 => PSDImageResourceID::JPEGQualityData,
			1032 => PSDImageResourceID::GridAndGuidesInfo,
			1033 => PSDImageResourceID::ThumbnailResource,
			1034 => PSDImageResourceID::CopyrightFlag,
			1035 => PSDImageResourceID::URL,
			1036 => PSDImageResourceID::ThumbnailResource2,
			1037 => PSDImageResourceID::GlobalAngle,
			1038 => PSDImageResourceID::ColorSamplers,
			1039 => PSDImageResourceID::ICCProfile,
			1040 => PSDImageResourceID::Watermark,
			1041 => PSDImageResourceID::ICCUntaggedProfile,
			1042 => PSDImageResourceID::EffectVisibility,
			1043 => PSDImageResourceID::SpotHalftone,
			1044 => PSDImageResourceID::IDSeedNumber,
			1045 => PSDImageResourceID::AlphaNameUnicode,
			1046 => PSDImageResourceID::IndexedColorTableCount,
			1047 => PSDImageResourceID::TransparencyIndex,
			1049 => PSDImageResourceID::GlobalAltitude,
			1050 => PSDImageResourceID::Slices,
			1051 => PSDImageResourceID::WorkflowURL,
			1052 => PSDImageResourceID::JumpToXPEP,
			1053 => PSDImageResourceID::AlphaIdentifiers,
			1054 => PSDImageResourceID::URLList,
			1057 => PSDImageResourceID::VersionInfo,
			1058 => PSDImageResourceID::EXIFData1,
			1059 => PSDImageResourceID::EXIFData3,
			1060 => PSDImageResourceID::XMPMetadata,
			1061 => PSDImageResourceID::CaptionDigest,
			1062 => PSDImageResourceID::PrintScale,
			1064 => PSDImageResourceID::PixelAspectRatio,
			1065 => PSDImageResourceID::LayerComps,
			1066 => PSDImageResourceID::AlternateDuotoneColors,
			1067 => PSDImageResourceID::AlternateSpotColors,
			1069 => PSDImageResourceID::LayerSelectionID,
			1070 => PSDImageResourceID::HDRToningInformation,
			1071 => PSDImageResourceID::PrintInfo,
			1072 => PSDImageResourceID::LayerGroupEnabledID,
			1073 => PSDImageResourceID::ColorSamplersResource,
			1074 => PSDImageResourceID::MeasurementScale,
			1075 => PSDImageResourceID::TimelineInfo,
			1076 => PSDImageResourceID::SheetDisclosure,
			1077 => PSDImageResourceID::DisplayInfoFP,
			1078 => PSDImageResourceID::OnionSkins,
			1080 => PSDImageResourceID::CountInfo,
			1082 => PSDImageResourceID::PrintInfo5,
			1083 => PSDImageResourceID::PrintStyle,
			1084 => PSDImageResourceID::MacintoshNSPrintInfoStruct,
			1085 => PSDImageResourceID::WindowsDevModeStruct,
			1086 => PSDImageResourceID::AutoSavePath,
			1087 => PSDImageResourceID::AutoSaveFormat,
			1088 => PSDImageResourceID::PathSelectionState,
			2000 ... 2997 => PSDImageResourceID::PathInformation(v - 2000),
			2999 => PSDImageResourceID::NameOfClippingPath,
			3000 => PSDImageResourceID::OriginPathInfo,
			4000 ... 4999 => PSDImageResourceID::PluginResources(v - 4000),
			7000 => PSDImageResourceID::ImageReadyVariables,
			7001 => PSDImageResourceID::ImageReadyDataSets,
			7002 => PSDImageResourceID::ImageReadyDefaultSelectedState,
			7003 => PSDImageResourceID::ImageReady7RolloverExpandedState,
			7004 => PSDImageResourceID::ImageReadyRolloverExpandedState,
			7005 => PSDImageResourceID::ImageReadySaveLayerSettings,
			7006 => PSDImageResourceID::ImageReadyVersion,
			8000 => PSDImageResourceID::LightroomWorkflow,
			10000 => PSDImageResourceID::PrintFlags2,
			_ => PSDImageResourceID::Unknown(v)
		}
	}
}
pub enum PSDImageResourceSection {}

impl UnsizedNativeFileContent for PSDImageResource
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(PSDImageResource, usize, std::fs::File), PSDLoadingError>
	{
		try!(fp.read_u32_be().map_err(PSDLoadingError::from).and_then(|sig|
			if sig != unsafe { std::mem::transmute(['8' as u8, 'B' as u8, 'I' as u8, 'M' as u8]) }
			{
				Err(PSDLoadingError::SignatureMismatching("PSDImageResource"))
			}
			else { Ok(()) }
		));
		let resource_id = try!(fp.read_u16().map(PSDImageResourceID::from));
		let reading_bytes = 4 + 2;
		let (name, bytes, mut fp) = try!(PascalString::read_from_file(fp, 2));
		let reading_bytes = reading_bytes + bytes;
		let (data, bytes) = try!(fp.read_u32().and_then(|len| if len > 0
		{
			let mut data_bytes = vec![0u8; len as usize];
			fp.read_exact(&mut data_bytes).map(|()| (data_bytes, len as usize + 4))
		}
		else { Ok((Vec::new(), 1)) }));
		let additional_reads = if bytes % 2 != 0 { try!(fp.read_u8()); 1 } else { 0 };
		let reading_bytes = reading_bytes + bytes + additional_reads;

		Ok((PSDImageResource
		{
			id: resource_id, name: name, data: data
		}, reading_bytes, fp))
	}
}
impl NativeFileContent<Vec<PSDImageResource>> for PSDImageResourceSection
{
	fn read_from_file(mut fp: std::fs::File) -> Result<(Vec<PSDImageResource>, std::fs::File), PSDLoadingError>
	{
		fp.read_u32().map_err(PSDLoadingError::from).and_then(|section_length|
		{
			fn read_recursive(fp: std::fs::File, mut resources: Vec<PSDImageResource>, left_bytes: usize)
				-> Result<(Vec<PSDImageResource>, std::fs::File), PSDLoadingError>
			{
				if left_bytes == 0 { Ok((resources, fp)) }
				else
				{
					let (res, bytes, rest_fp) = try!(PSDImageResource::read_from_file(fp));
					resources.push(res);
					read_recursive(rest_fp, resources, left_bytes - bytes)
				}
			}
			read_recursive(fp, Vec::new(), section_length as usize)
		})
	}
}
