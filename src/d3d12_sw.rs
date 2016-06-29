// Direct3D12 Safety Wrappers

use winapi::*;
use d3d12::*;
use dxgi::*;
use dxguid::*;
use user32::*;
use ::std;

macro_rules! DefSafetyObject
{
	($name: ident for $ift: ty) =>
	{
		pub struct $name { ptr: *mut $ift }
		impl std::default::Default for $name { fn default() -> Self { $name { ptr: std::ptr::null_mut() } } }
		impl std::ops::Drop for $name
		{
			fn drop(&mut self) { if !self.ptr.is_null() { unsafe { (*self.ptr).Release() }; } }
		}
	}
}

pub struct ComResult
{
	result: HRESULT
}
impl ComResult
{
	fn from(res: HRESULT) -> Self { ComResult { result: res } }
	fn failure_or<T>(&self, obj: T) -> Result<T, HRESULT> where T: std::marker::Sized { if SUCCEEDED(self.result) { Ok(obj) } else { Err(self.result) } }
}

DefSafetyObject!(DXGIFactory for IDXGIFactory2);
DefSafetyObject!(DXGIAdapter for IDXGIAdapter1);
DefSafetyObject!(DXGISwapchain for IDXGISwapChain3);
impl DXGIFactory
{
	pub fn create(debug: bool) -> Result<Self, HRESULT>
	{
		let mut obj: DXGIFactory = Default::default();
		ComResult::from(unsafe { CreateDXGIFactory2(if debug { DXGI_CREATE_FACTORY_DEBUG } else { 0 }, &IID_IDXGIFactory2, std::mem::transmute(&mut obj.ptr)) })
			.failure_or(obj)
	}
	pub fn enum_adapters(&self, index: u32) -> Result<DXGIAdapter, HRESULT>
	{
		let mut obj: DXGIAdapter = Default::default();
		ComResult::from(unsafe { (*self.ptr).EnumAdapters1(index, std::mem::transmute(&mut obj.ptr)) }).failure_or(obj)
	}

	pub fn create_swapchain(&self, queue: &D3D12CommandQueue, target: HWND) -> Result<DXGISwapchain, HRESULT>
	{
		let mut obj1: *mut IDXGISwapChain = std::ptr::null_mut();
		let mut obj: DXGISwapchain = Default::default();

		let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
		unsafe { GetClientRect(target, &mut rect) };

		let mut desc = DXGI_SWAP_CHAIN_DESC
		{
			BufferCount: 2, BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT, Windowed: TRUE,
			SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, OutputWindow: target, SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
			BufferDesc: DXGI_MODE_DESC
			{
				Width: (rect.right - rect.left) as u32, Height: (rect.bottom - rect.top) as u32, Format: DXGI_FORMAT_R8G8B8A8_UNORM,
				Scaling: DXGI_MODE_SCALING_UNSPECIFIED, ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
				RefreshRate: DXGI_RATIONAL { Numerator: 0, Denominator: 0 }
			},
			Flags: 0
		};
		let hr = unsafe { (*self.ptr).CreateSwapChain(std::mem::transmute(queue.ptr), &mut desc, std::mem::transmute(&mut obj1)) };
		if SUCCEEDED(hr)
		{
			ComResult::from(unsafe { (*obj1).QueryInterface(&IID_IDXGISwapChain3, std::mem::transmute(&mut obj.ptr)) }).failure_or(obj)
		}
		else { Err(hr) }
	}
}
DefSafetyObject!(D3D12Debug for ID3D12Debug);
impl D3D12Debug
{
	pub fn get() -> Result<Self, HRESULT>
	{
		let mut obj: Self = Default::default();
		ComResult::from(unsafe { D3D12GetDebugInterface(&IID_ID3D12Debug, std::mem::transmute(&mut obj.ptr)) }).failure_or(obj)
	}
	pub fn enable_debug_layer(&self) { unsafe { (*self.ptr).EnableDebugLayer() }; }
}
DefSafetyObject!(D3D12Device for ID3D12Device);
DefSafetyObject!(D3D12CommandQueue for ID3D12CommandQueue);
impl D3D12Device
{
	pub fn create(adapter: &DXGIAdapter) -> Result<Self, HRESULT>
	{
		let mut obj: Self = Default::default();
		ComResult::from(unsafe { D3D12CreateDevice(std::mem::transmute(adapter.ptr), D3D_FEATURE_LEVEL_11_0, &IID_ID3D12Device, std::mem::transmute(&mut obj.ptr)) })
			.failure_or(obj)
	}

	// Create Device Objects //
	pub fn create_command_queue(&self, queue_type: D3D12_COMMAND_LIST_TYPE) -> Result<D3D12CommandQueue, HRESULT>
	{
		let mut obj: D3D12CommandQueue = Default::default();
		let desc = D3D12_COMMAND_QUEUE_DESC { Flags: D3D12_COMMAND_QUEUE_FLAG_NONE, Type: queue_type, NodeMask: 0, Priority: 0 };
		ComResult::from(unsafe { (*self.ptr).CreateCommandQueue(&desc, &IID_ID3D12CommandQueue, std::mem::transmute(&mut obj.ptr)) })
			.failure_or(obj)
	}
}
