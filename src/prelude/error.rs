// Prelude: Error Enums and Crash Handling

use std;
use vkffi::*;

pub enum EngineError
{
	DeviceError(VkResult), GenericError(&'static str)
}
impl std::convert::From<VkResult> for EngineError
{
	fn from(res: VkResult) -> EngineError { EngineError::DeviceError(res) }
}
impl std::fmt::Debug for EngineError
{
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
	{
		match self
		{
			&EngineError::DeviceError(ref r) => write!(formatter, "DeviceError: {:?}", r),
			&EngineError::GenericError(ref e) => write!(formatter, "GenericError: {}", e),
		}
	}
}
pub fn crash(err: EngineError) -> !
{
	error!(target: "Prelude", "{:?}", err);
	panic!("Application has exited due to {}", match err
	{
		EngineError::DeviceError(_) => "DeviceError",
		EngineError::GenericError(_) => "GenericError"
	})
}
