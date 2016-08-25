// Prelude: Debug Printing

#![allow(mutable_transmutes)]

use prelude;
use prelude::internals::*;
use vkffi::*;
use std::collections::LinkedList;
use std;
use std::cell::RefCell;

const TEXTURE_SIZE: u32 = 512;

// Shelf method
#[derive(Debug)]
pub struct TextureRegion
{
	u: f32, v: f32, uw: f32, vh: f32
}
pub struct Horizon
{
	base_height: u32, maximum_height: u32, placement_left: u32
}
impl Horizon
{
	fn new(base_height: u32, init_height: u32, init_left: u32) -> Self
	{
		Horizon { base_height: base_height, maximum_height: init_height, placement_left: init_left }
	}
}

pub struct DebugInfo
{
	dres: DeviceResource, sres: StagingResource, horizons: RefCell<LinkedList<Horizon>>
}
impl DebugInfo
{
	pub fn new(engine: &Engine) -> Result<Self, EngineError>
	{
		let texture_atlas_desc = prelude::ImageDescriptor2::new(VkFormat::R8_UNORM, VkExtent2D(TEXTURE_SIZE, TEXTURE_SIZE),
			prelude::ImageUsagePresets::AsColorTexture);
		let resource_prealloc = prelude::ResourcePreallocator::new()
			.image_2d(vec![&texture_atlas_desc]);
		let (dev, stage) = try!(engine.create_double_buffer(&resource_prealloc));
		let (dev, stage) = (dev, stage.unwrap());

		let this = DebugInfo
		{
			dres: dev, sres: stage, horizons: RefCell::new(LinkedList::new())
		};

		Ok(this)
	}
	pub fn allocate_rect(&self, rect: VkExtent2D) -> Option<TextureRegion>
	{
		let VkExtent2D(tw, th) = rect;

		fn recursive_find<'a, IterMut: 'a + std::iter::Iterator<Item=&'a mut Horizon>>(mut iter: IterMut, target: VkExtent2D) -> Option<TextureRegion>
		{
			let VkExtent2D(tw, th) = target;
			match iter.next()
			{
				Some(h) => if h.maximum_height >= th && h.placement_left + th <= TEXTURE_SIZE
				{
					// use this
					let new_left = h.placement_left;
					h.placement_left += tw;
					Some(TextureRegion
					{
						u: new_left as f32 / TEXTURE_SIZE as f32, v: h.base_height as f32 / TEXTURE_SIZE as f32,
						uw: tw as f32 / TEXTURE_SIZE as f32, vh: th as f32 / TEXTURE_SIZE as f32
					})
				}
				else { recursive_find(iter, target) },
				_ => None
			}
		}

		let mut horizons_mut = self.horizons.borrow_mut();
		recursive_find(horizons_mut.iter_mut(), rect).or_else(||
			// cannot find free space
			match horizons_mut.back_mut()
			{
				Some(ref mut lh) if lh.placement_left + tw <= TEXTURE_SIZE =>
				{
					// use this with height expansion
					let new_left = lh.placement_left;
					lh.maximum_height = std::cmp::max(th, lh.maximum_height);
					lh.placement_left += tw;
					Some(TextureRegion
					{
						u: new_left as f32 / TEXTURE_SIZE as f32, v: lh.base_height as f32 / TEXTURE_SIZE as f32,
						uw: tw as f32 / TEXTURE_SIZE as f32, vh: th as f32 / TEXTURE_SIZE as f32
					})
				},
				_ => None
			}.or_else(||
			{
				// no available horizons found
				let new_base_height = if let Some(lh) = horizons_mut.back() { lh.base_height + lh.maximum_height } else { 0 };
				if new_base_height + th < TEXTURE_SIZE
				{
					horizons_mut.push_back(Horizon::new(new_base_height, th, tw));
					Some(TextureRegion
					{
						u: 0.0f32, v: new_base_height as f32 / TEXTURE_SIZE as f32,
						uw: tw as f32 / TEXTURE_SIZE as f32, vh: th as f32 / TEXTURE_SIZE as f32
					})
				}
				else { None }
			})
		)
	}

	pub fn test(&self)
	{
		let alloc = self.allocate_rect(VkExtent2D(8, 16)).unwrap();
		info!(target: "Prelude::Test", "Allocate 8x16 at {:?}", alloc);
		let alloc = self.allocate_rect(VkExtent2D(9, 16)).unwrap();
		info!(target: "Prelude::Test", "Allocate 9x16 at {:?}", alloc);
		let alloc = self.allocate_rect(VkExtent2D(20, 23)).unwrap();
		info!(target: "Prelude::Test", "Allocate 20x23 at {:?}", alloc);
		let alloc = self.allocate_rect(VkExtent2D(128, 8)).unwrap();
		info!(target: "Prelude::Test", "Allocate 128x8 at {:?}", alloc);
		let alloc = self.allocate_rect(VkExtent2D(512, 8)).unwrap();
		info!(target: "Prelude::Test", "Allocate 512x8 at {:?}", alloc);
		let alloc = self.allocate_rect(VkExtent2D(256, 16)).unwrap();
		info!(target: "Prelude::Test", "Allocate 256x16 at {:?}", alloc);
		let alloc = self.allocate_rect(VkExtent2D(256, 16)).unwrap();
		info!(target: "Prelude::Test", "Allocate 256x16 at {:?}", alloc);
	}
}
