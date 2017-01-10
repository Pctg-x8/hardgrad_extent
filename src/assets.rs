// Assets

use std;
use interlude::*;
use interlude::ffi::*;
use std::mem::size_of;
use structures::*;
use logical_resources::*;

pub struct ShaderStore
{
	// Vertex Shaders //
	pub geometry_preinstancing_vsh: ShaderProgram,
	pub erz_preinstancing_vsh: ShaderProgram,
	pub player_rotate_vsh: ShaderProgram,
	pub playerbullet_vsh: ShaderProgram,
	pub lineburst_particle_vsh: ShaderProgram,
	pub gridrender_vsh: ShaderProgram,
	pub bullet_vsh: ShaderProgram,
	// Geometry Shaders //
	pub enemy_duplication_gsh: ShaderProgram,
	pub background_duplication_gsh: ShaderProgram,
	pub enemy_rezonator_duplication_gsh: ShaderProgram,
	pub lineburst_particle_instantiate_gsh: ShaderProgram,
	// Fragment Shaders //
	pub solid_fsh: ShaderProgram,
	pub sprite_fsh: ShaderProgram,
	pub tonemap_fsh: ShaderProgram,
	pub colored_sprite_fsh: ShaderProgram
}
impl ShaderStore
{
	pub fn new<Engine: EngineCore>(engine: &Engine) -> Self
	{
		ShaderStore
		{
			geometry_preinstancing_vsh: Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.GeometryPreinstancing", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<u32>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32_UINT, 0)
			])),
			erz_preinstancing_vsh: Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.EnemyRezonatorV", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<CVector4>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
			])),
			player_rotate_vsh: Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.PlayerRotor", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<CVector4>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
			])),
			playerbullet_vsh: Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.PlayerBullet", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<CVector4>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
			])),
			lineburst_particle_vsh: Unrecoverable!(engine.create_vertex_shader_from_asset("shaders.LineBurstParticleVert", "main", &[
				VertexBinding::PerVertex(size_of::<LineBurstParticleGroup>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32_UINT, 0),
				VertexAttribute(0, VkFormat::R32G32_SFLOAT, size_of::<u32>() as u32)
			])),
			bullet_vsh: engine.create_vertex_shader_from_asset("shaders.BulletVert", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<CVector2>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32G32_SFLOAT, 0)
			]).or_crash(),
			gridrender_vsh: engine.create_vertex_shader_from_asset("shaders.GridRenderV", "main",
				&[VertexBinding::PerVertex(size_of::<Position>() as u32)], &[VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0)]).or_crash(),
			enemy_duplication_gsh: Unrecoverable!(engine.create_geometry_shader_from_asset("shaders.EnemyDuplicator", "main")),
			background_duplication_gsh: Unrecoverable!(engine.create_geometry_shader_from_asset("shaders.BackLineDuplicator", "main")),
			enemy_rezonator_duplication_gsh: Unrecoverable!(engine.create_geometry_shader_from_asset("shaders.EnemyRezonatorDup", "main")),
			lineburst_particle_instantiate_gsh: Unrecoverable!(engine.create_geometry_shader_from_asset("shaders.LineBurstParticleInstantiate", "main")),
			solid_fsh: Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.ThroughColor", "main")),
			sprite_fsh: Unrecoverable!(engine.create_fragment_shader_from_asset("shaders.SpriteFrag", "main")),
			tonemap_fsh: engine.create_fragment_shader_from_asset("shaders.SaturatingToneMap", "main").or_crash(),
			colored_sprite_fsh: engine.create_fragment_shader_from_asset("shaders.ColoredSpriteFrag", "main").or_crash()
		}
	}
}

// Application specified buffer data
pub struct ApplicationBufferData
{
	alloc_bp: BufferPreallocator,
	pub dev: DeviceBuffer, pub stg: StagingBuffer
}
impl ApplicationBufferData
{
	pub fn new<Engine: EngineCore>(engine: &Engine, target_extent: &Size2) -> Self
	{
		let application_buffer_prealloc = engine.buffer_preallocate(&[
			(size_of::<BulletTranslations>(), BufferDataType::Storage),
			(size_of::<UniformMemory>(), BufferDataType::Uniform),
			(size_of::<InstanceMemory>(), BufferDataType::Vertex),
			(size_of::<[PosUV; 4]>(), BufferDataType::Vertex),
			(size_of::<VertexMemoryForWireRender>(), BufferDataType::Vertex),
			(size_of::<IndexMemory>(), BufferDataType::Index)
		]);
		let (application_data, appdata_stage) = engine.create_double_buffer(&application_buffer_prealloc).or_crash();
		let this = ApplicationBufferData { alloc_bp: application_buffer_prealloc, dev: application_data, stg: appdata_stage };
		this.initialize(target_extent);
		this
	}
	fn initialize(&self, target_extent: &Size2)
	{
		let mapped = self.stg.map().or_crash();

		*mapped.map_mut::<[PosUV; 4]>(self.offset_ppvbuf()) = [
			PosUV(-1.0f32, -1.0f32, 0.0f32, 0.0f32), PosUV(1.0f32, -1.0f32, 1.0f32, 0.0f32),
			PosUV(-1.0f32, 1.0f32, 0.0f32, 1.0f32), PosUV(1.0f32, 1.0f32, 1.0f32, 1.0f32)
		];
		let vertices = mapped.map_mut::<VertexMemoryForWireRender>(self.offset_vbuf());
		let indices = mapped.map_mut::<IndexMemory>(self.offset_ibuf());
		vertices.unit_plane_source_vts = [
			Position(-1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, 0.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, 0.0f32, 1.0f32)
		];
		vertices.player_cube_vts = [
			Position(-1.0f32, -1.0f32, -1.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, -1.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, -1.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, -1.0f32, 1.0f32),
			Position(-1.0f32, -1.0f32,  1.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32,  1.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32,  1.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32,  1.0f32, 1.0f32)
		];
		vertices.enemy_rezonator_vts = [
			Position(0.0f32, 1.0f32, 0.0f32, 1.0f32),
			Position(-1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position(1.0f32, -1.0f32, 0.0f32, 1.0f32)
		];
		vertices.sprite_plane_vts = [
			Position(-1.0, -1.0, 0.0, 1.0),
			Position( 1.0, -1.0, 0.0, 1.0),
			Position(-1.0,  1.0, 0.0, 1.0),
			Position( 1.0,  1.0, 0.0, 1.0)
		];
		indices.player_cube_ids = [
			0, 1, 1, 2, 2, 3, 3, 0,
			4, 5, 5, 6, 6, 7, 7, 4,
			0, 4, 1, 5, 2, 6, 3, 7
		];
		let uniforms = mapped.map_mut::<UniformMemory>(self.offset_uniform());
		projection_matrixes::setup_parameters(uniforms, target_extent);
	}

	pub fn offset_bullet_translations(&self) -> usize { self.alloc_bp.offset(0) }
	pub fn size_bullet_translations(&self) -> usize { size_of::<BulletTranslations>() }
	pub fn offset_uniform(&self) -> usize { self.alloc_bp.offset(1) }
	pub fn size_uniform(&self) -> usize { size_of::<UniformMemory>() }
	pub fn offset_instance(&self) -> usize { self.alloc_bp.offset(2) }
	pub fn range_need_to_update(&self) -> std::ops::Range<usize> { self.offset_bullet_translations() .. self.offset_instance() + size_of::<InstanceMemory>() }
	pub fn offset_ppvbuf(&self) -> usize { self.alloc_bp.offset(3) }
	pub fn offset_vbuf(&self) -> usize { self.alloc_bp.offset(4) }
	pub fn offset_ibuf(&self) -> usize { self.alloc_bp.offset(5) }
	pub fn size(&self) -> usize { self.alloc_bp.total_size() }
}
