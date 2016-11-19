HardGrad -> Extend
---

**This is development branch for Windows and Vulkan**  

-- Beyond the Galaxy of Wires --  
(Bullet-hell) Shooting game for Windows with Vulkan

For Android N(in progress, many features are not implemented) => [Pctg-x8/hardgrad_mobile](https://github.com/Pctg-x8/hardgrad_mobile)

## Properties

- Language: Rust 1.13.0
- Build System: cargo 0.13.0
- Supported Platform: Windows(Win32+Vulkan)~~/Linux(X11+Vulkan)~~**Another branch(xcb_vk)**

## Compiling Shaders

HardGrad -> Extend uses glslc in google's shaderc([google/shaderc](https://github.com/google/shaderc)) to compile shaders.

1. Install or build shaderc(glslc) into your system.
2. set $Env:SHADERC_BUILD_DIR to your shaderc build directory
3. Execute following commands to get SPIR-V binaries
 - On Linux: `make -C shaders`
 - On Windows: `assets/build_shaders.ps1`
