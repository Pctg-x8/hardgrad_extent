// Render Backend Common Traits

// Traits of device that create a Swapchain(typed TargetType) from Window/Surface(ObjectiveType)
pub trait SwapchainFactory<TargetType, ObjectiveType>
{
	fn create_swapchain(&self, target: &TargetType) -> Result<ObjectiveType, String>;
}
