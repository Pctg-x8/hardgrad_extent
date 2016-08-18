// Prelude: Error Enums and Crash Handling

use std;
use vkffi::*;

pub enum EngineError
{
	DeviceError(VkResult), IOError(std::io::Error), GenericError(&'static str)
}
impl std::convert::From<VkResult> for EngineError
{
	fn from(res: VkResult) -> EngineError { EngineError::DeviceError(res) }
}
impl std::convert::From<std::io::Error> for EngineError
{
	fn from(ie: std::io::Error) -> EngineError { EngineError::IOError(ie) }
}
impl std::fmt::Debug for EngineError
{
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
	{
		match self
		{
			&EngineError::DeviceError(ref r) => write!(formatter, "DeviceError: {:?}", r),
			&EngineError::IOError(ref e) => write!(formatter, "IOError: {:?}", e),
			&EngineError::GenericError(ref e) => write!(formatter, "GenericError: {}", e)
		}
	}
}
pub fn crash(err: EngineError) -> !
{
	error!(target: "Prelude", "{:?}", err);
	panic!("Application has exited due to {}", match err
	{
		EngineError::DeviceError(_) => "DeviceError",
		EngineError::IOError(_) => "Input/Output Error",
		EngineError::GenericError(_) => "GenericError"
	})
}
