// Render Backend Common Traits

#[cfg(feature = "use_win32")]
use winapi::*;
#[cfg(feature = "use_x11")]
use x11::*;

pub trait SwapchainFactory<TargetType, ObjectiveType>
{
	fn create_swapchain(&self, target: &TargetType) -> Result<ObjectiveType, String>;
}
