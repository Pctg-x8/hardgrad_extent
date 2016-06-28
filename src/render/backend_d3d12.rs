// Render Backend(DirectX12)

use d3d12_sw::*;
use winapi::*;

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
		// let swapchain = factory.create_swapchain(&queue, target).unwrap();

		RenderBackend
		{
			instance: factory, physical_device: adapter, device: device, queue: queue
		}
	}
}
