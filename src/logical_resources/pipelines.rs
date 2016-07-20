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

pub struct PipelineCommonStore<'d>
{
	device_ref: &'d vk::Device<'d>,
	pub cache: vk::PipelineCache<'d>,
	pub layout_ub2_pc1: vk::PipelineLayout<'d>,
	pub layout_ub1_s1: vk::PipelineLayout<'d>,
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
			layout_ub2_pc1: device.create_pipeline_layout(&[
				*descriptor_sets.set_layout_ub1, *descriptor_sets.set_layout_ub1
			], &[VkPushConstantRange(VK_SHADER_STAGE_VERTEX_BIT, 0, std::mem::size_of::<u32>() as u32)]).unwrap(),
			layout_ub1_s1: device.create_pipeline_layout(&[
				*descriptor_sets.set_layout_ub1, *descriptor_sets.set_layout_s1
			], &[]).unwrap(),
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
		let vshader = commons.device_ref.create_shader_module_from_file("shaders/EnemyRenderV.spv").unwrap();

		let shader_entry = std::ffi::CString::new("main").unwrap();
		let vertex_bindings =
		[
			VkVertexInputBindingDescription(0, std::mem::size_of::<Position>() as u32, VkVertexInputRate::Vertex),
			VkVertexInputBindingDescription(1, std::mem::size_of::<u32>() as u32, VkVertexInputRate::Instance)
		];
		let vertex_inputs =
		[
			VkVertexInputAttributeDescription(0, 0, VkFormat::R32G32B32A32_SFLOAT, 0),
			VkVertexInputAttributeDescription(1, 1, VkFormat::R32_UINT, 0)
		];
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
			VkPipelineShaderStageCreateInfo
			{
				sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
				stage: VK_SHADER_STAGE_VERTEX_BIT, module: vshader.get(), pName: shader_entry.as_ptr(),
				pSpecializationInfo: &shader_const_specialization
			}, VkPipelineShaderStageCreateInfo
			{
				sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
				stage: VK_SHADER_STAGE_FRAGMENT_BIT, module: commons.through_color_fs.get(), pName: shader_entry.as_ptr(),
				pSpecializationInfo: std::ptr::null()
			}
		];
		let vertex_input_state = VkPipelineVertexInputStateCreateInfo
		{
			sType: VkStructureType::Pipeline_VertexInputStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			vertexBindingDescriptionCount: vertex_bindings.len() as u32, pVertexBindingDescriptions: vertex_bindings.as_ptr(),
			vertexAttributeDescriptionCount: vertex_inputs.len() as u32, pVertexAttributeDescriptions: vertex_inputs.as_ptr()
		};
		let input_assembly_state = VkPipelineInputAssemblyStateCreateInfo
		{
			sType: VkStructureType::Pipeline_InputAssemblyStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			topology: VkPrimitiveTopology::LineList, primitiveRestartEnable: false as VkBool32
		};
		let viewport_state = VkPipelineViewportStateCreateInfo
		{
			sType: VkStructureType::Pipeline_ViewportStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			viewportCount: viewports.len() as u32, pViewports: viewports.as_ptr(),
			scissorCount: scissors.len() as u32, pScissors: scissors.as_ptr()
		};
		let rasterization_state = VkPipelineRasterizationStateCreateInfo
		{
			depthClampEnable: false as VkBool32, .. Default::default()
		};
		let multisample_state: VkPipelineMultisampleStateCreateInfo = Default::default();
		let attachment_blend_states =
		[
			VkPipelineColorBlendAttachmentState
			{
				blendEnable: false as VkBool32,
				colorWriteMask: VK_COLOR_COMPONENT_A_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_R_BIT,
				srcColorBlendFactor: VkBlendFactor::One, dstColorBlendFactor: VkBlendFactor::One, colorBlendOp: VkBlendOp::Add,
				srcAlphaBlendFactor: VkBlendFactor::One, dstAlphaBlendFactor: VkBlendFactor::One, alphaBlendOp: VkBlendOp::Add
			}
		];
		let blend_state = VkPipelineColorBlendStateCreateInfo
		{
			sType: VkStructureType::Pipeline_ColorBlendStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			logicOpEnable: false as VkBool32, logicOp: VkLogicOp::NOP, blendConstants: [0.0f32; 4],
			attachmentCount: attachment_blend_states.len() as u32, pAttachments: attachment_blend_states.as_ptr()
		};
		let pipeline_info = VkGraphicsPipelineCreateInfo
		{
			sType: VkStructureType::GraphicsPipelineCreateInfo, pNext: std::ptr::null(), flags: 0,
			stageCount: shader_stages.len() as u32, pStages: shader_stages.as_ptr(),
			pVertexInputState: &vertex_input_state,
			pInputAssemblyState: &input_assembly_state,
			pTessellationState: std::ptr::null(),
			pDepthStencilState: std::ptr::null(),
			pViewportState: &viewport_state,
			pRasterizationState: &rasterization_state,
			pMultisampleState: &multisample_state,
			pColorBlendState: &blend_state,
			pDynamicState: std::ptr::null(),
			layout: commons.layout_ub2_pc1.get(), renderPass: render_pass.get(), subpass: 0,
			basePipelineHandle: std::ptr::null_mut(), basePipelineIndex: 0
		};

		EnemyRenderer
		{
			layout_ref: &commons.layout_ub2_pc1,
			state: commons.device_ref.create_graphics_pipelines(&commons.cache, &[pipeline_info]).unwrap().into_iter().next().unwrap()
		}
	}
}
pub struct DebugRenderer<'d>
{
	pub layout_ref: &'d vk::PipelineLayout<'d>,
	pub state: vk::Pipeline<'d>
}
impl <'d> DebugRenderer<'d>
{
	pub fn new(commons: &'d PipelineCommonStore, render_pass: &'d vk::RenderPass, framebuffer_size: VkExtent2D) -> Self
	{
		let VkExtent2D(width, height) = framebuffer_size;
		let vshader = commons.device_ref.create_shader_module_from_file("shaders/DebugTextured.spv").unwrap();

		let shader_entry = std::ffi::CString::new("main").unwrap();
		let vertex_bindings = [
			VkVertexInputBindingDescription(0, std::mem::size_of::<TexturedPos>() as u32, VkVertexInputRate::Vertex)
		];
		let vertex_attributes = [
			VkVertexInputAttributeDescription(0, 0, VkFormat::R32G32B32A32_SFLOAT, 0),
			VkVertexInputAttributeDescription(1, 0, VkFormat::R32G32B32A32_SFLOAT, std::mem::size_of::<Position>() as u32)
		];
		let viewports = [VkViewport(0.0f32, 0.0f32, width as f32, height as f32, 0.0f32, 1.0f32)];
		let scissors = [VkRect2D(VkOffset2D(0, 0), framebuffer_size)];
		let shader_specialization_map_entries =
		[
			VkSpecializationMapEntry(10, 0, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(11, (std::mem::size_of::<f32>() * 1) as u32, std::mem::size_of::<f32>()),
			VkSpecializationMapEntry(12, (std::mem::size_of::<f32>() * 2) as u32, std::mem::size_of::<f32>())
		];
		let shader_specialization_data = [1.0f32, 1.0f32, 1.0f32];
		let shader_const_specialization = VkSpecializationInfo
		{
			mapEntryCount: shader_specialization_map_entries.len() as u32, pMapEntries: shader_specialization_map_entries.as_ptr(),
			dataSize: std::mem::size_of::<[f32; 4]>(), pData: unsafe { std::mem::transmute(shader_specialization_data.as_ptr()) }
		};
		let shader_stages =
		[
			VkPipelineShaderStageCreateInfo
			{
				sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
				stage: VK_SHADER_STAGE_VERTEX_BIT, module: vshader.get(), pName: shader_entry.as_ptr(),
				pSpecializationInfo: &shader_const_specialization
			}, VkPipelineShaderStageCreateInfo
			{
				sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
				stage: VK_SHADER_STAGE_FRAGMENT_BIT, module: commons.alpha_applier_fs.get(), pName: shader_entry.as_ptr(),
				pSpecializationInfo: std::ptr::null()
			}
		];
		let vertex_input_state = VkPipelineVertexInputStateCreateInfo
		{
			sType: VkStructureType::Pipeline_VertexInputStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			vertexBindingDescriptionCount: vertex_bindings.len() as u32, pVertexBindingDescriptions: vertex_bindings.as_ptr(),
			vertexAttributeDescriptionCount: vertex_attributes.len() as u32, pVertexAttributeDescriptions: vertex_attributes.as_ptr()
		};
		let input_assembly_state = VkPipelineInputAssemblyStateCreateInfo
		{
			sType: VkStructureType::Pipeline_InputAssemblyStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			topology: VkPrimitiveTopology::TriangleList, primitiveRestartEnable: false as VkBool32
		};
		let viewport_state = VkPipelineViewportStateCreateInfo
		{
			sType: VkStructureType::Pipeline_ViewportStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			viewportCount: viewports.len() as u32, pViewports: viewports.as_ptr(),
			scissorCount: scissors.len() as u32, pScissors: scissors.as_ptr()
		};
		let rasterization_state = VkPipelineRasterizationStateCreateInfo
		{
			depthClampEnable: true as VkBool32, .. Default::default()
		};
		let multisample_state: VkPipelineMultisampleStateCreateInfo = Default::default();
		let attachment_blend_states =
		[
			VkPipelineColorBlendAttachmentState
			{
				blendEnable: true as VkBool32,
				colorWriteMask: VK_COLOR_COMPONENT_A_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_R_BIT,
				srcColorBlendFactor: VkBlendFactor::SrcAlpha, dstColorBlendFactor: VkBlendFactor::OneMinusSrcAlpha, colorBlendOp: VkBlendOp::Add,
				srcAlphaBlendFactor: VkBlendFactor::OneMinusDstAlpha, dstAlphaBlendFactor: VkBlendFactor::One, alphaBlendOp: VkBlendOp::Add
			}
		];
		let blend_state = VkPipelineColorBlendStateCreateInfo
		{
			sType: VkStructureType::Pipeline_ColorBlendStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			logicOpEnable: false as VkBool32, logicOp: VkLogicOp::NOP, blendConstants: [0.0f32; 4],
			attachmentCount: attachment_blend_states.len() as u32, pAttachments: attachment_blend_states.as_ptr()
		};
		let pipeline_info = VkGraphicsPipelineCreateInfo
		{
			sType: VkStructureType::GraphicsPipelineCreateInfo, pNext: std::ptr::null(), flags: 0,
			stageCount: shader_stages.len() as u32, pStages: shader_stages.as_ptr(),
			pVertexInputState: &vertex_input_state,
			pInputAssemblyState: &input_assembly_state,
			pTessellationState: std::ptr::null(),
			pDepthStencilState: std::ptr::null(),
			pViewportState: &viewport_state,
			pRasterizationState: &rasterization_state,
			pMultisampleState: &multisample_state,
			pColorBlendState: &blend_state,
			pDynamicState: std::ptr::null(),
			layout: commons.layout_ub1_s1.get(), renderPass: render_pass.get(), subpass: 0,
			basePipelineHandle: std::ptr::null_mut(), basePipelineIndex: 0
		};

		DebugRenderer
		{
			layout_ref: &commons.layout_ub1_s1,
			state: commons.device_ref.create_graphics_pipelines(&commons.cache, &[pipeline_info]).unwrap().into_iter().next().unwrap()
		}
	}
}
