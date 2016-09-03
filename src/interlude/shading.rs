// Prelude: Primitive Shading(Shaders and Pipelines)

use super::internals::*;
use std;
use vkffi::*;
use render_vk::wrap as vk;
use traits::*;
use std::ffi::CString;

pub struct VertexInputState(Vec<VertexBinding>, Vec<VertexAttribute>);
#[derive(Clone)]
pub enum VertexBinding
{
	PerVertex(u32), PerInstance(u32)
}
#[derive(Clone)]
pub struct VertexAttribute(pub u32, pub VkFormat, pub u32);
pub struct IntoNativeVertexInputState
{
	bindings: Vec<VkVertexInputBindingDescription>,
	attributes: Vec<VkVertexInputAttributeDescription>
}
impl <'a> std::convert::Into<VkPipelineVertexInputStateCreateInfo> for &'a IntoNativeVertexInputState
{
	fn into(self) -> VkPipelineVertexInputStateCreateInfo
	{
		VkPipelineVertexInputStateCreateInfo
		{
			sType: VkStructureType::Pipeline_VertexInputStateCreateInfo, pNext: std::ptr::null(), flags: 0,
			vertexBindingDescriptionCount: self.bindings.len() as u32, pVertexBindingDescriptions: self.bindings.as_ptr(),
			vertexAttributeDescriptionCount: self.attributes.len() as u32, pVertexAttributeDescriptions: self.attributes.as_ptr()
		}
	}
}

pub enum ShaderProgram
{
	Vertex { internal: vk::ShaderModule, entry_point: CString, vertex_input: VertexInputState },
	#[allow(dead_code)] TessControl { internal: vk::ShaderModule, entry_point: CString },
	#[allow(dead_code)] TessEvaluate { internal: vk::ShaderModule, entry_point: CString },
	Geometry { internal: vk::ShaderModule, entry_point: CString },
	Fragment { internal: vk::ShaderModule, entry_point: CString }
}
pub trait ShaderProgramInternals
{
	fn new_vertex(module: vk::ShaderModule, entry_point: &str, vbindings: &[VertexBinding], vattributes: &[VertexAttribute]) -> Self;
	fn new_geometry(module: vk::ShaderModule, entry_point: &str) -> Self;
	fn new_fragment(module: vk::ShaderModule, entry_point: &str) -> Self;
	fn get_entry_point(&self) -> &CString;
	fn shader_stage_create_info(&self) -> VkPipelineShaderStageCreateInfo;
	fn into_native_vertex_input_state(&self) -> IntoNativeVertexInputState;
}
impl InternalExports<vk::ShaderModule> for ShaderProgram
{
	fn get_internal(&self) -> &vk::ShaderModule
	{
		match self
		{
			&ShaderProgram::Vertex { internal: ref e, entry_point: _, vertex_input: _ } => e,
			&ShaderProgram::Geometry { internal: ref e, entry_point: _ } => e,
			&ShaderProgram::Fragment { internal: ref e, entry_point: _ } => e,
			&ShaderProgram::TessControl { internal: ref e, entry_point: _ } => e,
			&ShaderProgram::TessEvaluate { internal: ref e, entry_point: _ } => e
		}
	}
}
impl ShaderProgramInternals for ShaderProgram
{
	fn new_vertex(module: vk::ShaderModule, entry_point: &str, vbindings: &[VertexBinding], vattributes: &[VertexAttribute]) -> Self
	{
		ShaderProgram::Vertex
		{
			internal: module, entry_point: CString::new(entry_point).unwrap(),
			vertex_input: VertexInputState(Vec::from(vbindings), Vec::from(vattributes))
		}
	}
	fn new_geometry(module: vk::ShaderModule, entry_point: &str) -> Self
	{
		ShaderProgram::Geometry { internal: module, entry_point: CString::new(entry_point).unwrap() }
	}
	fn new_fragment(module: vk::ShaderModule, entry_point: &str) -> Self
	{
		ShaderProgram::Fragment { internal: module, entry_point: CString::new(entry_point).unwrap() }
	}
	fn get_entry_point(&self) -> &CString
	{
		match self
		{
			&ShaderProgram::Vertex { internal: _, entry_point: ref e, vertex_input: _ } => e,
			&ShaderProgram::Geometry { internal: _, entry_point: ref e } => e,
			&ShaderProgram::Fragment { internal: _, entry_point: ref e } => e,
			&ShaderProgram::TessControl { internal: _, entry_point: ref e } => e,
			&ShaderProgram::TessEvaluate { internal: _, entry_point: ref e } => e
		}
	}
	fn shader_stage_create_info(&self) -> VkPipelineShaderStageCreateInfo
	{
		VkPipelineShaderStageCreateInfo
		{
			sType: VkStructureType::Pipeline_ShaderStageCreateInfo, pNext: std::ptr::null(), flags: 0,
			stage: match self
			{
				&ShaderProgram::Vertex { internal: _, entry_point: _, vertex_input: _ } => VK_SHADER_STAGE_VERTEX_BIT,
				&ShaderProgram::Geometry { internal: _, entry_point: _ } => VK_SHADER_STAGE_GEOMETRY_BIT,
				&ShaderProgram::Fragment { internal: _, entry_point: _ } => VK_SHADER_STAGE_FRAGMENT_BIT,
				&ShaderProgram::TessControl { internal: _, entry_point: _ } => VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT,
				&ShaderProgram::TessEvaluate { internal: _, entry_point: _ } => VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT
			},
			module: self.get_internal().get(), pName: self.get_entry_point().as_ptr(), pSpecializationInfo: std::ptr::null()
		}
	}
	fn into_native_vertex_input_state(&self) -> IntoNativeVertexInputState
	{
		if let &ShaderProgram::Vertex { internal: _, entry_point: _, vertex_input: VertexInputState(ref vb, ref va) } = self
		{
			IntoNativeVertexInputState
			{
				bindings: vb.iter().enumerate().map(|(i, x)| match x
				{
					&VertexBinding::PerVertex(stride) => VkVertexInputBindingDescription(i as u32, stride, VkVertexInputRate::Vertex),
					&VertexBinding::PerInstance(stride) => VkVertexInputBindingDescription(i as u32, stride, VkVertexInputRate::Instance)
				}).collect(),
				attributes: va.iter().enumerate()
					.map(|(i, &VertexAttribute(binding, format, offset))| VkVertexInputAttributeDescription(i as u32, binding, format, offset))
					.collect()
			}
		}
		else { panic!("Unable to create vertex input state from the exception of vertex shader") }
	}
}

#[derive(Clone)]
pub struct PushConstantDesc(pub VkShaderStageFlags, pub std::ops::Range<u32>);
impl <'a> std::convert::Into<VkPushConstantRange> for &'a PushConstantDesc
{
	fn into(self) -> VkPushConstantRange
	{
		let PushConstantDesc(stage, ref range) = *self;
		VkPushConstantRange(stage, range.start, range.len() as u32)
	}
}

pub struct PipelineLayout { internal: vk::PipelineLayout }
pub trait PipelineLayoutInternals { fn new(pl: vk::PipelineLayout) -> Self; }
impl PipelineLayoutInternals for PipelineLayout
{
	fn new(pl: vk::PipelineLayout) -> Self { PipelineLayout { internal: pl } }
}
impl InternalExports<vk::PipelineLayout> for PipelineLayout
{
	fn get_internal(&self) -> &vk::PipelineLayout { &self.internal }
}

// Primitive Topology + With-Adjacency flag
#[derive(Clone, Copy)]
pub enum PrimitiveTopology
{
	LineList(bool), LineStrip(bool), TriangleList(bool), TriangleStrip(bool)
}
impl std::convert::Into<VkPrimitiveTopology> for PrimitiveTopology
{
	fn into(self) -> VkPrimitiveTopology
	{
		match self
		{
			PrimitiveTopology::LineList(false)		=> VkPrimitiveTopology::LineList,
			PrimitiveTopology::LineList(true)		=> VkPrimitiveTopology::LineListWithAdjacency,
			PrimitiveTopology::LineStrip(false)		=> VkPrimitiveTopology::LineStrip,
			PrimitiveTopology::LineStrip(true)		=> VkPrimitiveTopology::LineStripWithAdjacency,
			PrimitiveTopology::TriangleList(false)	=> VkPrimitiveTopology::TriangleList,
			PrimitiveTopology::TriangleList(true)	=> VkPrimitiveTopology::TriangleListWithAdjacency,
			PrimitiveTopology::TriangleStrip(false)	=> VkPrimitiveTopology::TriangleStrip,
			PrimitiveTopology::TriangleStrip(true)	=> VkPrimitiveTopology::TriangleStripWithAdjacency
		}
	}
}
#[derive(Clone, Copy)]
pub struct ViewportWithScissorRect(VkViewport, VkRect2D);
impl ViewportWithScissorRect
{
	pub fn default_scissor(vp: VkViewport) -> Self
	{
		let VkViewport(vx, vy, vw, vh, _, _) = vp;
		ViewportWithScissorRect(vp, VkRect2D(VkOffset2D(vx as i32, vy as i32), VkExtent2D(vw as u32, vh as u32)))
	}
}
#[derive(Clone, Copy)]
pub enum CullingSide { Front, Back }
impl std::convert::Into<VkCullModeFlags> for CullingSide
{
	fn into(self) -> VkCullModeFlags
	{
		match self
		{
			CullingSide::Front => VK_CULL_MODE_FRONT_BIT,
			CullingSide::Back => VK_CULL_MODE_BACK_BIT
		}
	}
}
#[derive(Clone)]
pub struct RasterizerState
{
	pub wired_render: bool, pub cull_side: Option<CullingSide>
}
#[derive(Clone, Copy)]
pub enum AttachmentBlendState
{
	Disabled, AlphaBlend, PremultipliedAlphaBlend
}
impl std::convert::Into<VkPipelineColorBlendAttachmentState> for AttachmentBlendState
{
	fn into(self) -> VkPipelineColorBlendAttachmentState
	{
		match self
		{
			AttachmentBlendState::Disabled => VkPipelineColorBlendAttachmentState
			{
				blendEnable: false as VkBool32,
				srcColorBlendFactor: VkBlendFactor::One, dstColorBlendFactor: VkBlendFactor::One,
				srcAlphaBlendFactor: VkBlendFactor::One, dstAlphaBlendFactor: VkBlendFactor::One,
				colorBlendOp: VkBlendOp::Add, alphaBlendOp: VkBlendOp::Add,
				colorWriteMask: VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT
			},
			AttachmentBlendState::AlphaBlend => VkPipelineColorBlendAttachmentState
			{
				blendEnable: true as VkBool32,
				srcColorBlendFactor: VkBlendFactor::SrcAlpha, dstColorBlendFactor: VkBlendFactor::OneMinusSrcAlpha,
				srcAlphaBlendFactor: VkBlendFactor::One, dstAlphaBlendFactor: VkBlendFactor::OneMinusSrcAlpha,
				colorBlendOp: VkBlendOp::Add, alphaBlendOp: VkBlendOp::Add,
				colorWriteMask: VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT
			},
			AttachmentBlendState::PremultipliedAlphaBlend => VkPipelineColorBlendAttachmentState
			{
				blendEnable: true as VkBool32,
				srcColorBlendFactor: VkBlendFactor::One, dstColorBlendFactor: VkBlendFactor::OneMinusSrcAlpha,
				srcAlphaBlendFactor: VkBlendFactor::One, dstAlphaBlendFactor: VkBlendFactor::OneMinusSrcAlpha,
				colorBlendOp: VkBlendOp::Add, alphaBlendOp: VkBlendOp::Add,
				colorWriteMask: VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT
			}
		}
	}
}

pub struct GraphicsPipelineBuilder<'a>
{
	layout: &'a PipelineLayout, render_pass: &'a RenderPass, subpass_index: u32,
	vertex_shader: Option<&'a ShaderProgram>, geometry_shader: Option<&'a ShaderProgram>,
	fragment_shader: Option<&'a ShaderProgram>,
	primitive_topology: PrimitiveTopology, vp_sc: Vec<ViewportWithScissorRect>,
	rasterizer_state: RasterizerState, use_alpha_to_coverage: bool, attachment_blend_states: Vec<AttachmentBlendState>
}
impl <'a> GraphicsPipelineBuilder<'a>
{
	pub fn new(layout: &'a PipelineLayout, render_pass: &'a RenderPass, subpass_index: u32) -> Self
	{
		GraphicsPipelineBuilder
		{
			layout: layout, render_pass: render_pass, subpass_index: subpass_index,
			vertex_shader: None, geometry_shader: None, fragment_shader: None,
			primitive_topology: PrimitiveTopology::TriangleList(false),
			vp_sc: Vec::new(), rasterizer_state: RasterizerState { wired_render: false, cull_side: None },
			use_alpha_to_coverage: false, attachment_blend_states: Vec::new()
		}
	}
	pub fn inherit(base: &GraphicsPipelineBuilder<'a>) -> Self
	{
		GraphicsPipelineBuilder
		{
			layout: base.layout, render_pass: base.render_pass, subpass_index: base.subpass_index,
			vertex_shader: base.vertex_shader, geometry_shader: base.geometry_shader, fragment_shader: base.fragment_shader,
			primitive_topology: base.primitive_topology, vp_sc: base.vp_sc.clone(), rasterizer_state: base.rasterizer_state.clone(),
			use_alpha_to_coverage: base.use_alpha_to_coverage, attachment_blend_states: base.attachment_blend_states.clone()
		}
	}
	pub fn vertex_shader(mut self, vshader: &'a ShaderProgram) -> Self
	{
		match vshader
		{
			&ShaderProgram::Vertex { internal: _, entry_point: _, vertex_input: _ } => { self.vertex_shader = Some(vshader); self },
			_ => panic!("Prelude Assertion: GraphicsPIpelineBuilder::geometry_shader is called with not a geometry shader")
		}
	}
	pub fn geometry_shader(mut self, gshader: &'a ShaderProgram) -> Self
	{
		match gshader
		{
			&ShaderProgram::Geometry { internal: _, entry_point: _ } => { self.geometry_shader = Some(gshader); self },
			_ => panic!("Prelude Assertion: GraphicsPIpelineBuilder::geometry_shader is called with not a geometry shader")
		}
	}
	pub fn fragment_shader(mut self, fshader: &'a ShaderProgram) -> Self
	{
		match fshader
		{
			&ShaderProgram::Fragment { internal: _, entry_point: _ } => { self.fragment_shader = Some(fshader); self },
			_ => panic!("Prelude Assertion: GraphicsPIpelineBuilder::fragment_shader is called with not a fragment shader")
		}
	}
	pub fn primitive_topology(mut self, pt: PrimitiveTopology) -> Self
	{
		self.primitive_topology = pt;
		self
	}
	pub fn viewport_scissors(mut self, vpsc: &[ViewportWithScissorRect]) -> Self
	{
		self.vp_sc = Vec::from(vpsc);
		self
	}
	pub fn rasterizer_enable_wired_mode(mut self) -> Self
	{
		self.rasterizer_state.wired_render = true;
		self
	}
	pub fn rasterizer_enable_culling(mut self, side: CullingSide) -> Self
	{
		self.rasterizer_state.cull_side = Some(side);
		self
	}
	pub fn enable_alpha_to_coverage(mut self) -> Self
	{
		self.use_alpha_to_coverage = true;
		self
	}
	pub fn blend_state(mut self, state: &[AttachmentBlendState]) -> Self
	{
		self.attachment_blend_states = Vec::from(state);
		self
	}
}
pub struct IntoNativeGraphicsPipelineCreateInfoStruct<'a>
{
	base: &'a GraphicsPipelineBuilder<'a>,
	#[allow(dead_code)] viewports: Vec<VkViewport>, #[allow(dead_code)] scissors: Vec<VkRect2D>,
	#[allow(dead_code)] attachment_blend_states: Vec<VkPipelineColorBlendAttachmentState>,
	#[allow(dead_code)] into_vertex_input_state: IntoNativeVertexInputState,
	shader_stage: Vec<VkPipelineShaderStageCreateInfo>,
	vertex_input_state: VkPipelineVertexInputStateCreateInfo,
	input_assembly_state: VkPipelineInputAssemblyStateCreateInfo,
	viewport_state: VkPipelineViewportStateCreateInfo,
	rasterization_state: VkPipelineRasterizationStateCreateInfo,
	multisample_state: VkPipelineMultisampleStateCreateInfo,
	color_blend_state: VkPipelineColorBlendStateCreateInfo
}
impl <'a> std::convert::Into<IntoNativeGraphicsPipelineCreateInfoStruct<'a>> for &'a GraphicsPipelineBuilder<'a>
{
	fn into(self) -> IntoNativeGraphicsPipelineCreateInfoStruct<'a>
	{
		let vshader = self.vertex_shader.expect("VertexShader is required");
		let mut shader_stage_vec = vec![vshader.shader_stage_create_info()];
		if let Some(gs) = self.geometry_shader { shader_stage_vec.push(gs.shader_stage_create_info()); }
		if let Some(fs) = self.fragment_shader { shader_stage_vec.push(fs.shader_stage_create_info()); }
		let into_input_state = vshader.into_native_vertex_input_state();
		let vports = self.vp_sc.iter().map(|&ViewportWithScissorRect(vp, _)| vp).collect::<Vec<_>>();
		let scissors = self.vp_sc.iter().map(|&ViewportWithScissorRect(_, sc)| sc).collect::<Vec<_>>();
		let attachment_blend_states = self.attachment_blend_states.iter().map(|&x| x.into()).collect::<Vec<_>>();
		IntoNativeGraphicsPipelineCreateInfoStruct
		{
			shader_stage: shader_stage_vec,
			vertex_input_state: (&into_input_state).into(),
			input_assembly_state: VkPipelineInputAssemblyStateCreateInfo
			{
				sType: VkStructureType::Pipeline_InputAssemblyStateCreateInfo, pNext: std::ptr::null(), flags: 0,
				topology: self.primitive_topology.into(), primitiveRestartEnable: false as VkBool32
			},
			viewport_state: VkPipelineViewportStateCreateInfo
			{
				sType: VkStructureType::Pipeline_ViewportStateCreateInfo, pNext: std::ptr::null(), flags: 0,
				viewportCount: vports.len() as u32, pViewports: vports.as_ptr(),
				scissorCount: scissors.len() as u32, pScissors: scissors.as_ptr()
			},
			rasterization_state: VkPipelineRasterizationStateCreateInfo
			{
				sType: VkStructureType::Pipeline_RasterizationStateCreateInfo, pNext: std::ptr::null(), flags: 0,
				depthClampEnable: false as VkBool32, depthBiasEnable: false as VkBool32, rasterizerDiscardEnable: self.fragment_shader.is_none() as VkBool32,
				polygonMode: if self.rasterizer_state.wired_render { VkPolygonMode::Line } else { VkPolygonMode::Fill },
				cullMode: if let Some(side) = self.rasterizer_state.cull_side { side.into() } else { VK_CULL_MODE_NONE },
				frontFace: VkFrontFace::CounterClockwise,
				depthBiasConstantFactor: 0.0f32, depthBiasClamp: 0.0f32, depthBiasSlopeFactor: 0.0f32,
				lineWidth: 1.0f32
			},
			multisample_state: VkPipelineMultisampleStateCreateInfo
			{
				sType: VkStructureType::Pipeline_MultisampleStateCreateInfo, pNext: std::ptr::null(), flags: 0,
				rasterizationSamples: VK_SAMPLE_COUNT_1_BIT, sampleShadingEnable: false as VkBool32,
				minSampleShading: 0.0f32, pSampleMask: std::ptr::null(),
				alphaToCoverageEnable: self.use_alpha_to_coverage as VkBool32, alphaToOneEnable: false as VkBool32
			},
			color_blend_state: VkPipelineColorBlendStateCreateInfo
			{
				sType: VkStructureType::Pipeline_ColorBlendStateCreateInfo, pNext: std::ptr::null(), flags: 0,
				logicOpEnable: false as VkBool32, logicOp: VkLogicOp::NOP,
				attachmentCount: attachment_blend_states.len() as u32, pAttachments: attachment_blend_states.as_ptr(),
				blendConstants: [0.0f32; 4]
			},
			into_vertex_input_state: into_input_state, attachment_blend_states: attachment_blend_states,
			viewports: vports, scissors: scissors, base: self
		}
	}
}
impl <'a> std::convert::Into<VkGraphicsPipelineCreateInfo> for &'a IntoNativeGraphicsPipelineCreateInfoStruct<'a>
{
	fn into(self) -> VkGraphicsPipelineCreateInfo
	{
		VkGraphicsPipelineCreateInfo
		{
			sType: VkStructureType::GraphicsPipelineCreateInfo, pNext: std::ptr::null(), flags: 0,
			stageCount: self.shader_stage.len() as u32, pStages: self.shader_stage.as_ptr(),
			pVertexInputState: &self.vertex_input_state, pInputAssemblyState: &self.input_assembly_state,
			pTessellationState: std::ptr::null(), pViewportState: &self.viewport_state,
			pRasterizationState: &self.rasterization_state, pMultisampleState: &self.multisample_state,
			pDepthStencilState: std::ptr::null(), pColorBlendState: &self.color_blend_state,
			pDynamicState: std::ptr::null(),
			layout: self.base.layout.internal.get(), renderPass: self.base.render_pass.get_internal().get(), subpass: self.base.subpass_index,
			basePipelineHandle: std::ptr::null_mut(), basePipelineIndex: 0
		}
	}
}

pub struct GraphicsPipeline { internal: vk::Pipeline }
pub trait GraphicsPipelineInternals { fn new(p: vk::Pipeline) -> Self; }
impl GraphicsPipelineInternals for GraphicsPipeline
{
	fn new(p: vk::Pipeline) -> Self { GraphicsPipeline { internal: p } }
}
impl InternalExports<vk::Pipeline> for GraphicsPipeline
{
	fn get_internal(&self) -> &vk::Pipeline { &self.internal }
}
