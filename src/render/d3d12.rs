// Render Backend(DirectX12)

use d3d12_sw::*;
use winapi::*;
use render::SwapchainFactory;

pub struct RenderBackend
{
	instance: DXGIFactory, physical_device: DXGIAdapter, device: D3D12Device, queue: D3D12CommandQueue
}
impl RenderBackend
{
	pub fn init() -> Self
	{
		println!("-- Initializing RenderBackend with Direct3D12");

		D3D12Debug::get().unwrap().enable_debug_layer();
		let factory = DXGIFactory::create(true).unwrap();
		let adapter = factory.enum_adapters(0).unwrap();
		let device = D3D12Device::create(&adapter).unwrap();
		let queue = device.create_command_queue(D3D12_COMMAND_LIST_TYPE_DIRECT).unwrap();

		RenderBackend
		{
			instance: factory, physical_device: adapter, device: device, queue: queue
		}
	}
}
impl SwapchainFactory<HWND, DXGISwapchain> for RenderBackend
{
	fn create_swapchain(&self, target: &HWND) -> Result<DXGISwapchain, String>
	{
		self.instance.create_swapchain(&self.queue, *target).map_err(|hr| format!("{:?}", hr))
	}
}
