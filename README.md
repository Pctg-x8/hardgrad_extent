HardGrad -> Extent
---

**This is development branch for Linux(XCB) and Vulkan**  

-- Beyond the Galaxy of Wires --  
(Bullet-hell) Shooting game for Linux with Vulkan

For Android N(in progress, many features are not implemented) => [Pctg-x8/hardgrad_mobile](https://github.com/Pctg-x8/hardgrad_mobile)

## Properties

- Language: Rust 1.9.0
- Build System: cargo 0.10.0
- Supported Platform: ~~Windows(future support with DirectX12)/~~Linux(X11+Vulkan)

## Compiling Shaders

HardGrad -> Extend uses glslc in google's shaderc([google/shaderc](https://github.com/google/shaderc)) to compile shaders.

1. Install or build shaderc(glslc) into your system.
2. set $SHADERC_BUILD_DIR to your shaderc build directory
3. `make -C shaders` to get SPIR-V binaries
