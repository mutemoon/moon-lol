# DXBC 加载（预编译，非源码编译）

> 上级：[shader.md](../shader.md)

游戏使用**预编译 DXBC 字节码**，存储在 `ShaderCache.dx11.wad.client` 中；所谓「编译」实际是从 WAD 缓存查找/加载对应 define 组合的 DXBC 变体。详见 [analysis_shader_compile.md](file:///d:/Users/admin/workspace/lol-reverse/analysis_shader_compile.md)。

```
代码路径:  ASSETS/Shaders/HLSL/Environment/UNLIT_DECAL_PS.ps
  ↓ ShaderWad_LoadDx11File 拼接 .dx11
实际路径:  assets/shaders/hlsl/environment/unlit_decal_ps.ps.dx11
  ↓ 存储于 ShaderCache.dx11.wad.client（DXBC 字节码）
defines 列表 → 哈希 → 在 WAD 中定位对应变体
```

相关函数：`ShaderWad_LoadDx11File`(sub_1413CA860)、`ShaderWad_LoadByHash`(sub_1413C8030)、`ShaderWad_LoadAndCreateD3DShader`(sub_1413C2D50)、`ShaderVS_LoadFromWad`(sub_14130ED40)、`ShaderPS_LoadFromWad`(sub_14130DAC0)、`Shader_MergeDefinesAndCompile`(sub_1413018C0)、`ShaderCompileCache_Lookup`(sub_141312FE0)、`WadArchive_Init`(sub_1405E3D10)。完整清单见 [functions.md](../functions.md)。
