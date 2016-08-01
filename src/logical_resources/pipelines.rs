use vkffi::*;
use render_vk::wrap as vk;
use std;
use device_resources;
use traits::*;
use vertex_formats::*;

impl std::default::Default for VkPipelineRasterizationStateCreateInfo
{
	fn default() -> Self
	{
		VkPipelineRasterizationStateCreateInfo
		{
			sType: VkStructureType::Pipeline_RasterizationStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			polygonMode: VkPolygonMode::Fill, cullMode: VK_CULL_MODE_NONE, frontFace: VkFrontFace::CounterClockwise,
			rasterizerDiscardEnable: false as VkBool32, depthClampEnable: false as VkBool32, depthBiasEnable: false as VkBool32,
			lineWidth: 1.0f32, depthBiasConstantFactor: 0.0f32, depthBiasClamp: 0.0f32, depthBiasSlopeFactor: 0.0f32
		}
	}
}
impl std::default::Default for VkPipelineMultisampleStateCreateInfo
{
	fn default() -> Self
	{
		VkPipelineMultisampleStateCreateInfo
		{
			sType: VkStructureType::Pipeline_MultisampleStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			rasterizationSamples: VK_SAMPLE_COUNT_1_BIT, sampleShadingEnable: false as VkBool32,
			alphaToCoverageEnable: false as VkBool32, alphaToOneEnable: false as VkBool32,
			pSampleMask: std::ptr::null(), minSampleShading: 0.0f32
		}
	}
}
impl std::default::Default for VkGraphicsPipelineCreateInfo
{
	fn default() -> Self
	{
		VkGraphicsPipelineCreateInfo
		{
			sType: VkStructureType::GraphicsPipelineCreateInfo, pNext: std::ptr::null(), flags: 0,
			stageCount: 0, pStages: std::ptr::null(),
			pVertexInputState: std::ptr::null(), pInputAssemblyState: std::ptr::null(),
			pTessellationState: std::ptr::null(), pDepthStencilState: std::ptr::null(),
			pViewportState: std::ptr::null(), pRasterizationState: std::ptr::null(), pMultisampleState: std::ptr::null(),
			pColorBlendState: std::ptr::null(), pDynamicState: std::ptr::null(),
			layout: std::ptr::null_mut(), renderPass: std::ptr::null_mut(), subpass: 0,
			basePipelineHandle: std::ptr::null_mut(), basePipelineIndex: 0
		}
	}
}
enum ShaderStage {}
impl ShaderStage
{
	fn geometry(module: &VkShaderModule, entry: &std::ffi::CString, specialization: Option<&VkSpecializationInfo>) -> VkPipelineShaderStageCreateInfo
	{
		VkPipelineShaderStageCreateInfo
		{
			sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
			stage: VK_SHADER_STAGE_GEOMETRY_BIT, module: *module, pName: entry.as_ptr(),
			pSpecializationInfo: specialization.map(|x| x as *const VkSpecializationInfo).unwrap_or(std::ptr::null())
		}
	}
	fn fragment(module: &VkShaderModule, entry: &std::ffi::CString, specialization: Option<&VkSpecializationInfo>) -> VkPipelineShaderStageCreateInfo
	{
		VkPipelineShaderStageCreateInfo
		{
			sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
			stage: VK_SHADER_STAGE_FRAGMENT_BIT, module: *module, pName: entry.as_ptr(),
			pSpecializationInfo: specialization.map(|x| x as *const VkSpecializationInfo).unwrap_or(std::ptr::null())
		}
	}
}
enum InputAssemblyState {}
impl InputAssemblyState
{
	fn new(topo: VkPrimitiveTopology, primitive_restart: bool) -> VkPipelineInputAssemblyStateCreateInfo
	{
		VkPipelineInputAssemblyStateCreateInfo
		{
			sType: VkStructureType::Pipeline_InputAssemblyStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			topology: topo, primitiveRestartEnable: primitive_restart as VkBool32
		}
	}
}
struct ViewportState
{
	#[allow(dead_code)] viewports: Box<[VkViewport]>, #[allow(dead_code)] scissors: Box<[VkRect2D]>,
	data: VkPipelineViewportStateCreateInfo
}
impl ViewportState
{
	fn new(vports: Box<[VkViewport]>, scissors: Box<[VkRect2D]>) -> Self
	{
		ViewportState
		{
			data: VkPipelineViewportStateCreateInfo
			{
				sType: VkStructureType::Pipeline_ViewportStateCreateInfo, pNext: std::ptr::null(), flags: 0,
				viewportCount: vports.len() as u32, pViewports: vports.as_ptr(),
				scissorCount: scissors.len() as u32, pScissors: scissors.as_ptr()
			}, viewports: vports, scissors: scissors
		}
	}
}
impl std::ops::Deref for ViewportState
{
	type Target = VkPipelineViewportStateCreateInfo;
	fn deref(&self) -> &Self::Target { &self.data }
}

enum VertexInputBindingDesc {}
impl VertexInputBindingDesc
{
	fn per_vertex<T: std::marker::Sized>(binding: u32) -> VkVertexInputBindingDescription
	{
		VkVertexInputBindingDescription(binding, std::mem::size_of::<T>() as u32, VkVertexInputRate::Vertex)
	}
	fn per_instance<T: std::marker::Sized>(binding: u32) -> VkVertexInputBindingDescription
	{
		VkVertexInputBindingDescription(binding, std::mem::size_of::<T>() as u32, VkVertexInputRate::Instance)
	}
}

pub struct VertexShaderWithInputForm<'d>
{
	#[allow(dead_code)] device_ref: &'d vk::Device<'d>,
	#[allow(dead_code)] module: vk::ShaderModule<'d>,
	#[allow(dead_code)] bindings: Box<[VkVertexInputBindingDescription]>, #[allow(dead_code)] attributes: Box<[VkVertexInputAttributeDescription]>
}
impl <'d> VertexShaderWithInputForm<'d>
{
	fn new(device_ref: &'d vk::Device, shader_path: &str, bindings: Box<[VkVertexInputBindingDescription]>, attributes: Box<[VkVertexInputAttributeDescription]>) -> Self
	{
		let m = device_ref.create_shader_module_from_file(shader_path).unwrap();

		VertexShaderWithInputForm
		{
			device_ref: device_ref, module: m, bindings: bindings, attributes: attributes
		}
	}
	fn as_shader_stage(&self, entry: &std::ffi::CString, specialization: Option<&VkSpecializationInfo>) -> VkPipelineShaderStageCreateInfo
	{
		VkPipelineShaderStageCreateInfo
		{
			sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
			stage: VK_SHADER_STAGE_VERTEX_BIT, module: *self.module, pName: entry.as_ptr(),
			pSpecializationInfo: specialization.map(|x| x as *const VkSpecializationInfo).unwrap_or(std::ptr::null())
		}
	}
	fn as_vertex_input_state(&self) -> VkPipelineVertexInputStateCreateInfo
	{
		VkPipelineVertexInputStateCreateInfo
		{
			sType: VkStructureType::Pipeline_VertexInputStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			vertexBindingDescriptionCount: self.bindings.len() as u32, pVertexBindingDescriptions: self.bindings.as_ptr(),
			vertexAttributeDescriptionCount: self.attributes.len() as u32, pVertexAttributeDescriptions: self.attributes.as_ptr()
		}
	}
}
enum ColorBlendAttachmentStates {}
impl ColorBlendAttachmentStates
{
	fn no_blend() -> VkPipelineColorBlendAttachmentState
	{
		VkPipelineColorBlendAttachmentState
		{
			blendEnable: false as VkBool32,
			colorWriteMask: VK_COLOR_COMPONENT_A_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_R_BIT,
			srcColorBlendFactor: VkBlendFactor::One, dstColorBlendFactor: VkBlendFactor::One, colorBlendOp: VkBlendOp::Add,
			srcAlphaBlendFactor: VkBlendFactor::One, dstAlphaBlendFactor: VkBlendFactor::One, alphaBlendOp: VkBlendOp::Add
		}
	}
	fn premultiplied_alpha() -> VkPipelineColorBlendAttachmentState
	{
		VkPipelineColorBlendAttachmentState
		{
			blendEnable: true as VkBool32,
			colorWriteMask: VK_COLOR_COMPONENT_A_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_R_BIT,
			srcColorBlendFactor: VkBlendFactor::SrcAlpha, dstColorBlendFactor: VkBlendFactor::OneMinusSrcAlpha, colorBlendOp: VkBlendOp::Add,
			srcAlphaBlendFactor: VkBlendFactor::OneMinusDstAlpha, dstAlphaBlendFactor: VkBlendFactor::One, alphaBlendOp: VkBlendOp::Add
		}
	}
}
struct ColorBlendState
{
	#[allow(dead_code)] attachments: Box<[VkPipelineColorBlendAttachmentState]>,
	data: VkPipelineColorBlendStateCreateInfo
}
impl ColorBlendState
{
	fn new(attachments: Box<[VkPipelineColorBlendAttachmentState]>) -> ColorBlendState
	{
		ColorBlendState
		{
			data: VkPipelineColorBlendStateCreateInfo
			{
				sType: VkStructureType::Pipeline_ColorBlendStateCreateInfo, pNext: std::ptr::null(), flags: 0,
				logicOpEnable: false as VkBool32, logicOp: VkLogicOp::NOP, blendConstants: [0.0f32; 4],
				attachmentCount: attachments.len() as u32, pAttachments: attachments.as_ptr()
			}, attachments: attachments
		}
	}
}
impl std::ops::Deref for ColorBlendState
{
	type Target = VkPipelineColorBlendStateCreateInfo;
	fn deref(&self) -> &Self::Target { &self.data }
}

pub struct PipelineCommonStore<'d>
{
	device_ref: &'d vk::Device<'d>,
	pub cache: vk::PipelineCache<'d>,
	pub layout_uniform: vk::PipelineLayout<'d>,
	pub layout_ub1_s1: vk::PipelineLayout<'d>,
	default_shader_entry_point: std::ffi::CString,
	through_color_fs: vk::ShaderModule<'d>,
	alpha_applier_fs: vk::ShaderModule<'d>
}
impl <'d> PipelineCommonStore<'d>
{
	pub fn new(device: &'d vk::Device<'d>, descriptor_sets: &device_resources::DescriptorSets<'d>) -> Self
	{
		PipelineCommonStore
		{
			device_ref: device,
			cache: device.create_empty_pipeline_cache().unwrap(),
			layout_uniform: device.create_pipeline_layout(&[*descriptor_sets.set_layout_uniform_vg], &[]).unwrap(),
			layout_ub1_s1: device.create_pipeline_layout(&[
				*descriptor_sets.set_layout_uniform_vg, *descriptor_sets.set_layout_s1
			], &[]).unwrap(),
			default_shader_entry_point: std::ffi::CString::new("main").unwrap(),
			through_color_fs: device.create_shader_module_from_file("shaders/ThroughColor.spv").unwrap(),
			alpha_applier_fs: device.create_shader_module_from_file("shaders/AlphaApplier.spv").unwrap()
		}
	}
}
pub struct EnemyRenderer<'d>
{
	pub layout_ref: &'d vk::PipelineLayout<'d>,
	pub state: vk::Pipeline<'d>
}
impl <'d> EnemyRenderer<'d>
{
	pub fn new(commons: &'d PipelineCommonStore, render_pass: &vk::RenderPass<'d>, framebuffer_size: VkExtent2D) -> Self
	{
		let VkExtent2D(fb_width, fb_height) = framebuffer_size;
		let vshader_form = VertexShaderWithInputForm::new(commons.device_ref, "shaders/EnemyRenderV.spv",
			Box::new([VertexInputBindingDesc::per_vertex::<Position>(0), VertexInputBindingDesc::per_instance::<u32>(1)]),
			Box::new([VkVertexInputAttributeDescription(0, 0, VkFormat::R32G32B32A32_SFLOAT, 0), VkVertexInputAttributeDescription(1, 1, VkFormat::R32_UINT, 0)]));
		let gshader = commons.device_ref.create_shader_module_from_file("shaders/EnemyDuplicator.spv").unwrap();

		let viewports = [VkViewport(0.0f32, 0.0f32, fb_width as f32, fb_height as f32, 0.0f32, 1.0f32)];
		let scissors = [VkRect2D(VkOffset2D(0, 0), framebuffer_size)];
		let shader_specialization_map_entries =
		[
			VkSpecializationMapEntry(10, 0, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(11, (std::mem::size_of::<f32>() * 1) as u32, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(12, (std::mem::size_of::<f32>() * 2) as u32, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(13, (std::mem::size_of::<f32>() * 3) as u32, std::mem::size_of::<f32>())
		];
		let shader_specialization_data = [0.25f32, 0.9875f32, 1.5f32, 1.0f32];
		let shader_const_specialization = VkSpecializationInfo
		{
			mapEntryCount: shader_specialization_map_entries.len() as u32, pMapEntries: shader_specialization_map_entries.as_ptr(),
			dataSize: std::mem::size_of::<[f32; 4]>(), pData: unsafe { std::mem::transmute(shader_specialization_data.as_ptr()) }
		};
		let shader_stages =
		[
			vshader_form.as_shader_stage(&commons.default_shader_entry_point, None),
			ShaderStage::geometry(&gshader, &commons.default_shader_entry_point, Some(&shader_const_specialization)),
			ShaderStage::fragment(&commons.through_color_fs, &commons.default_shader_entry_point, None)
		];
		let vertex_input_state = vshader_form.as_vertex_input_state();
		let input_assembly_state = InputAssemblyState::new(VkPrimitiveTopology::LineListWithAdjacency, false);
		let viewport_state = ViewportState::new(Box::new(viewports), Box::new(scissors));
		let rasterization_state: VkPipelineRasterizationStateCreateInfo = Default::default();
		let multisample_state: VkPipelineMultisampleStateCreateInfo = Default::default();
		let blend_state = ColorBlendState::new(Box::new([ColorBlendAttachmentStates::no_blend()]));
		let pipeline_info = VkGraphicsPipelineCreateInfo
		{
			stageCount: shader_stages.len() as u32, pStages: shader_stages.as_ptr(),
			pVertexInputState: &vertex_input_state, pInputAssemblyState: &input_assembly_state,
			pViewportState: &*viewport_state, pRasterizationState: &rasterization_state,
			pMultisampleState: &multisample_state, pColorBlendState: &*blend_state,
			layout: commons.layout_uniform.get(), renderPass: render_pass.get(),
			.. Default::default()
		};

		EnemyRenderer
		{
			layout_ref: &commons.layout_uniform,
			state: commons.device_ref.create_graphics_pipelines(&commons.cache, &[pipeline_info]).unwrap().into_iter().next().unwrap()
		}
	}
}
pub struct BackgroundRenderer<'d>
{
	pub layout_ref: &'d vk::PipelineLayout<'d>,
	pub state: vk::Pipeline<'d>
}
impl <'d> BackgroundRenderer<'d>
{
	pub fn new(commons: &'d PipelineCommonStore, render_pass: &vk::RenderPass<'d>, framebuffer_size: VkExtent2D) -> Self
	{
		let VkExtent2D(fb_width, fb_height) = framebuffer_size;
		let vshader_form = VertexShaderWithInputForm::new(commons.device_ref, "shaders/RawOutput.spv",
			Box::new([VertexInputBindingDesc::per_vertex::<Position>(0), VertexInputBindingDesc::per_instance::<u32>(1)]),
			Box::new([VkVertexInputAttributeDescription(0, 0, VkFormat::R32G32B32A32_SFLOAT, 0), VkVertexInputAttributeDescription(1, 1, VkFormat::R32_UINT, 0)]));
		let gshader = commons.device_ref.create_shader_module_from_file("shaders/BackLineDuplicator.spv").unwrap();

		let viewports = [VkViewport(0.0f32, 0.0f32, fb_width as f32, fb_height as f32, 0.0f32, 1.0f32)];
		let scissors = [VkRect2D(VkOffset2D(0, 0), framebuffer_size)];
		let shader_specialization_map_entries =
		[
			VkSpecializationMapEntry(10, 0, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(11, (std::mem::size_of::<f32>() * 1) as u32, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(12, (std::mem::size_of::<f32>() * 2) as u32, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(13, (std::mem::size_of::<f32>() * 3) as u32, std::mem::size_of::<f32>())
		];
		let shader_specialization_data = [0.125f32, 0.5f32, 0.25f32, 0.75f32];
		let shader_const_specialization = VkSpecializationInfo
		{
			mapEntryCount: shader_specialization_map_entries.len() as u32, pMapEntries: shader_specialization_map_entries.as_ptr(),
			dataSize: std::mem::size_of::<[f32; 4]>(), pData: unsafe { std::mem::transmute(shader_specialization_data.as_ptr()) }
		};
		let shader_stages =
		[
			vshader_form.as_shader_stage(&commons.default_shader_entry_point, None),
			ShaderStage::geometry(&gshader, &commons.default_shader_entry_point, Some(&shader_const_specialization)),
			ShaderStage::fragment(&commons.through_color_fs, &commons.default_shader_entry_point, None)
		];
		let vertex_input_state = vshader_form.as_vertex_input_state();
		let input_assembly_state = InputAssemblyState::new(VkPrimitiveTopology::LineListWithAdjacency, false);
		let viewport_state = ViewportState::new(Box::new(viewports), Box::new(scissors));
		let rasterization_state: VkPipelineRasterizationStateCreateInfo = Default::default();
		let multisample_state: VkPipelineMultisampleStateCreateInfo = Default::default();
		let blend_state = ColorBlendState::new(Box::new([ColorBlendAttachmentStates::premultiplied_alpha()]));
		let pipeline_info = VkGraphicsPipelineCreateInfo
		{
			stageCount: shader_stages.len() as u32, pStages: shader_stages.as_ptr(),
			pVertexInputState: &vertex_input_state, pInputAssemblyState: &input_assembly_state,
			pViewportState: &*viewport_state, pRasterizationState: &rasterization_state,
			pMultisampleState: &multisample_state, pColorBlendState: &*blend_state,
			layout: commons.layout_uniform.get(), renderPass: render_pass.get(),
			.. Default::default()
		};

		BackgroundRenderer
		{
			layout_ref: &commons.layout_uniform,
			state: commons.device_ref.create_graphics_pipelines(&commons.cache, &[pipeline_info]).unwrap().into_iter().next().unwrap()
		}
	}
}
pub struct DebugRenderer<'d>
{
	pub layout_ref: &'d vk::PipelineLayout<'d>,
	pub state: vk::Pipeline<'d>, pub state_instanced: vk::Pipeline<'d>
}
impl <'d> DebugRenderer<'d>
{
	pub fn new(commons: &'d PipelineCommonStore, render_pass: &'d vk::RenderPass, framebuffer_size: VkExtent2D) -> Self
	{
		let VkExtent2D(width, height) = framebuffer_size;
		let textured_vshader_form = VertexShaderWithInputForm::new(
			commons.device_ref, "shaders/Textured.spv",
			Box::new([VertexInputBindingDesc::per_vertex::<TexturedPos>(0)]),
			Box::new([
				VkVertexInputAttributeDescription(0, 0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VkVertexInputAttributeDescription(1, 0, VkFormat::R32G32B32A32_SFLOAT, std::mem::size_of::<Position>() as u32)
			])
		);
		let numeric_render_vs_form = VertexShaderWithInputForm::new(
			commons.device_ref, "shaders/DebugTextured.spv",
			Box::new([VertexInputBindingDesc::per_vertex::<Position>(0), VertexInputBindingDesc::per_instance::<[[f32; 4]; 2]>(1)]),
			Box::new([
				VkVertexInputAttributeDescription(0, 0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VkVertexInputAttributeDescription(1, 1, VkFormat::R32G32B32A32_SFLOAT, 0),
				VkVertexInputAttributeDescription(2, 1, VkFormat::R32G32B32A32_SFLOAT, std::mem::size_of::<[f32; 4]>() as u32)
			])
		);

		let viewports = [VkViewport(0.0f32, 0.0f32, width as f32, height as f32, 0.0f32, 1.0f32)];
		let scissors = [VkRect2D(VkOffset2D(0, 0), framebuffer_size)];
		let color_constants = [1.0f32, 1.0f32, 1.0f32];
		let color_const_specialization =
		[
			VkSpecializationMapEntry(10, 0, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(11, (std::mem::size_of::<f32>() * 1) as u32, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(12, (std::mem::size_of::<f32>() * 2) as u32, std::mem::size_of::<f32>())
		];
		let color_const_specialization_info = VkSpecializationInfo
		{
			mapEntryCount: color_const_specialization.len() as u32, pMapEntries: color_const_specialization.as_ptr(),
			dataSize: std::mem::size_of::<[f32; 3]>(), pData: unsafe { std::mem::transmute(color_constants.as_ptr()) }
		};
		// Common States
		let ia_state = InputAssemblyState::new(VkPrimitiveTopology::TriangleList, false);
		let vp_state = ViewportState::new(Box::new(viewports), Box::new(scissors));
		let rasterization_state: VkPipelineRasterizationStateCreateInfo = Default::default();
		let multisample_state: VkPipelineMultisampleStateCreateInfo = Default::default();
		let blend_state = ColorBlendState::new(Box::new([ColorBlendAttachmentStates::premultiplied_alpha()]));
		// Textured Shader
		let ts_shader_stages =
		[
			textured_vshader_form.as_shader_stage(&commons.default_shader_entry_point, Some(&color_const_specialization_info)),
			ShaderStage::fragment(&commons.alpha_applier_fs, &commons.default_shader_entry_point, None)
		];
		let ts_vi_state = textured_vshader_form.as_vertex_input_state();
		let ts_pipeline_info = VkGraphicsPipelineCreateInfo
		{
			stageCount: ts_shader_stages.len() as u32, pStages: ts_shader_stages.as_ptr(),
			pVertexInputState: &ts_vi_state, pInputAssemblyState: &ia_state,
			pViewportState: &*vp_state, pRasterizationState: &rasterization_state,
			pMultisampleState: &multisample_state, pColorBlendState: &*blend_state,
			layout: *commons.layout_ub1_s1, renderPass: **render_pass,
			.. Default::default()
		};
		// Instanced-Textured Shader
		let its_shader_stages =
		[
			numeric_render_vs_form.as_shader_stage(&commons.default_shader_entry_point, Some(&color_const_specialization_info)),
			ShaderStage::fragment(&commons.alpha_applier_fs, &commons.default_shader_entry_point, None)
		];
		let its_vi_state = numeric_render_vs_form.as_vertex_input_state();
		let its_pipeline_info = VkGraphicsPipelineCreateInfo
		{
			stageCount: its_shader_stages.len() as u32, pStages: its_shader_stages.as_ptr(),
			pVertexInputState: &its_vi_state,
			.. ts_pipeline_info
		};
		let mut pipes = commons.device_ref.create_graphics_pipelines(&commons.cache, &[ts_pipeline_info, its_pipeline_info]).unwrap();
		let p2 = pipes.pop().unwrap();
		let p1 = pipes.pop().unwrap();
		assert_eq!(pipes.len(), 0);

		DebugRenderer
		{
			layout_ref: &commons.layout_ub1_s1,
			state: p1, state_instanced: p2
		}
	}
}
