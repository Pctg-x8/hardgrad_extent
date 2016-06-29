// Render Modules

macro_rules! ModuleRenamed
{
	($name: ident from $pkg: ident) =>
	{
		pub mod $pkg;
		pub use self::$pkg as $name;
	}
}

#[cfg(feature = "use_vk")] ModuleRenamed!(backend from vk);
#[cfg(feature = "use_d3d12")] ModuleRenamed!(backend from d3d12);

pub mod backend_common;
pub use self::backend_common::*;
