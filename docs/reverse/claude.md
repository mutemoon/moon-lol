- 使用 ida mcp 工具逆向分析
- 永远只能静态分析，禁止动态调试
- 在分析结束后尽量用 ida 工具函数命名、注释、定义结构体、定义变量签名等标记工作
- 有新的发现要记录到文档中
- 文档太大时需要拆分文档出一个文件夹，里面放子文档，外层文档内容只做简单的介绍和详细的目录

## 文档目录

| 文档 | 内容 |
|------|------|
| [functions.md](functions.md) | 函数命名速查表：只记录碰到并命名的函数 / 数据地址 / IDA 结构体清单 |
| [shader.md](shader.md) | Shader 系统分析入口：总体数据流简介 + 子文档目录 |
| [shader/structs.md](shader/structs.md) | 运行时结构体定义（ShaderEffect / ParticleEmitterDescriptor 等）与 IDA 复现脚本 |
| [shader/deserialization.md](shader/deserialization.md) | 配置 → 描述符反序列化 & 运行时默认值表 |
| [shader/assembly.md](shader/assembly.md) | Shader 输入装配核心概览（公共流程/两版差异/虚表分发/Type 总开关） |
| [shader/assembly-full.md](shader/assembly-full.md) | Full 版完整装配逻辑（Rust 伪代码：主链/宏开关/BindTextureSlot/逐-pass 渲染态） |
| [shader/constants.md](shader/constants.md) | 深度转换 & 软粒子 & 切片 & PIXEL_COLOR_REMAP_RAMP 默认纹理/采样器常量 |
| [shader/dxbc.md](shader/dxbc.md) | DXBC 预编译字节码加载流程 |

## 资源

已经从游戏解包出的所有配置文件 assets/props
