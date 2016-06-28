HardGrad -> Extent
---

-- Beyond the Galaxy of Wires --  
(Bullet-hell) Shooting game for Windows/Linux ported from Unity to DirectX12/Vulkan

For Android N(in progress, many features are not implemented) => [Pctg-x8/hardgrad_mobile](https://github.com/Pctg-x8/hardgrad_mobile)

## Properties

- Language: Rust 1.9.0
- Build System: cargo 0.10.0
- Supported Platform: Windows(future support with DirectX12)/Linux(Xorg/Vulkan)

## Build features
- Window System Dependencies
  - `use_x11`: Use X11 as Window System
  - `use_win32`: Use default window system of Windows
- Rendering API Dependencies
  - `use_d3d12`: Use Direct3D12 as Rendering API
  - `use_vk`: Use Vulkan as Rendering API
