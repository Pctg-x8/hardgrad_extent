// Assets

use std;
use interlude::*;
use interlude::ffi::*;
use std::mem::size_of;
use structures::*;
use constants::*;
use logical_resources::*;
use std::ops::Deref;
use std::rc::Rc;

pub struct ShaderStore
{
	// Vertex Shaders //
	pub geometry_preinstancing_vsh: Rc<VertexShader>,
	pub erz_preinstancing_vsh: Rc<VertexShader>,
	pub player_rotate_vsh: Rc<VertexShader>,
	pub playerbullet_vsh: Rc<VertexShader>,
	pub lineburst_particle_vsh: Rc<VertexShader>,
	pub gridrender_vsh: Rc<VertexShader>,
	pub bullet_vsh: Rc<VertexShader>,
	// Geometry Shaders //
	pub enemy_duplication_gsh: Rc<GeometryShader>,
	pub background_duplication_gsh: Rc<GeometryShader>,
	pub enemy_rezonator_duplication_gsh: Rc<GeometryShader>,
	pub lineburst_particle_instantiate_gsh: Rc<GeometryShader>,
	// Fragment Shaders //
	pub solid_fsh: Rc<FragmentShader>,
	pub sprite_fsh: Rc<FragmentShader>,
	pub tonemap_fsh: Rc<FragmentShader>,
	pub colored_sprite_fsh: Rc<FragmentShader>
}
impl ShaderStore
{
	pub fn new<Engine: AssetProvider + Deref<Target = GraphicsInterface>>(engine: &Engine) -> Self
	{
		ShaderStore
		{
			geometry_preinstancing_vsh: VertexShader::from_asset(engine, "shaders.GeometryPreinstancing", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<u32>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32_UINT, 0)
			]).or_crash(),
			erz_preinstancing_vsh: VertexShader::from_asset(engine, "shaders.EnemyRezonatorV", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<CVector4>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
			]).or_crash(),
			player_rotate_vsh: VertexShader::from_asset(engine, "shaders.PlayerRotor", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<CVector4>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
			]).or_crash(),
			playerbullet_vsh: VertexShader::from_asset(engine, "shaders.PlayerBullet", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<CVector4>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)
			]).or_crash(),
			lineburst_particle_vsh: VertexShader::from_asset(engine, "shaders.LineBurstParticleVert", "main", &[
				VertexBinding::PerVertex(size_of::<LineBurstParticleGroup>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32_UINT, 0),
				VertexAttribute(0, VkFormat::R32G32_SFLOAT, size_of::<u32>() as u32)
			]).or_crash(),
			bullet_vsh: VertexShader::from_asset(engine, "shaders.BulletVert", "main", &[
				VertexBinding::PerVertex(size_of::<CVector4>() as u32),
				VertexBinding::PerInstance(size_of::<CVector2>() as u32)
			], &[
				VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0),
				VertexAttribute(1, VkFormat::R32G32_SFLOAT, 0)
			]).or_crash(),
			gridrender_vsh: VertexShader::from_asset(engine, "shaders.GridRenderV", "main",
				&[VertexBinding::PerVertex(size_of::<Position>() as u32)], &[VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0)]).or_crash(),
			enemy_duplication_gsh: GeometryShader::from_asset(engine, "shaders.EnemyDuplicator", "main").or_crash(),
			background_duplication_gsh: GeometryShader::from_asset(engine, "shaders.BackLineDuplicator", "main").or_crash(),
			enemy_rezonator_duplication_gsh: GeometryShader::from_asset(engine, "shaders.EnemyRezonatorDup", "main").or_crash(),
			lineburst_particle_instantiate_gsh: GeometryShader::from_asset(engine, "shaders.LineBurstParticleInstantiate", "main").or_crash(),
			solid_fsh: FragmentShader::from_asset(engine, "shaders.ThroughColor", "main").or_crash(),
			sprite_fsh: FragmentShader::from_asset(engine, "shaders.SpriteFrag", "main").or_crash(),
			tonemap_fsh: FragmentShader::from_asset(engine, "shaders.SaturatingToneMap", "main").or_crash(),
			colored_sprite_fsh: FragmentShader::from_asset(engine, "shaders.ColoredSpriteFrag", "main").or_crash()
		}
	}
}

// Application specified buffer data
pub struct ApplicationBufferData
{
	alloc_bp: BufferOffsets,
	pub dev: DeviceBuffer, pub stg: StagingBuffer
}
impl ApplicationBufferData
{
	pub fn new(engine: &GraphicsInterface, target_extent: &Size2) -> Self
	{
		let application_buffer_prealloc = BufferPreallocator::new(engine, &[
			BufferContent::Storage(size_of::<BulletTranslations>()),
			BufferContent::Uniform(size_of::<UniformMemory>()),
			BufferContent::Vertex(size_of::<InstanceMemory>()),
			BufferContent::Vertex(size_of::<[PosUV; 4]>()),
			BufferContent::Vertex(size_of::<VertexMemoryForWireRender>()),
			BufferContent::Index(size_of::<IndexMemory>()),
			BufferContent::Custom(VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT, size_of::<[PosUV; 4]>() * MAX_BPARTICLES)
		]);
		let (application_data, appdata_stage) = application_buffer_prealloc.instantiate().or_crash();
		let this = ApplicationBufferData { alloc_bp: application_buffer_prealloc.independence(), dev: application_data, stg: appdata_stage };
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
