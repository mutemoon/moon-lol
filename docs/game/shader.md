# 着色器架构与 SPIR-V 转译集成指南

## 着色器变体与结构体匹配规则

- 英雄联盟客户端中不同分支条件下的着色器包含不同的结构体定义与全局变量布局
- 对于变体结构体，统一采用包含字符最长与字段最全的结构体声明进行对接
- 对于 Uniform 与 Input Output 接口，按照物理内存槽位与全局映射规则统一布局

## Vulkan SPIR-V 资源绑定与槽位隔离

### 描述符集冲突背景

- 从英雄联盟客户端提取的 `DXBC` 字节码通过 `dxbc-spirv` 工具转译为 `SPIR-V` 时，触发 `Vulkan` 验证层拦截
- 验证层报错标识为 `VUID-VkGraphicsPipelineCreateInfo-layout-07988`
- 独立转译导致顶点着色器与像素着色器均将资源默认映射在 `DescriptorSet` 0 且从 `Binding` 0 自增分配
- 顶点着色器占用 `Binding` 0 与 1 存放常量缓冲区，像素着色器占用 `Binding` 0 与 1 存放纹理与采样器，合并管线时物理重叠
- `Bevy` 引擎规范将 `Set` 0 预留给通用 `View`，`Set` 1 预留给 `Mesh View`，`Set` 2 预留给多实例，自定义材质资产必须绑定在 `Set` 3

### 编译器源码改造与选项扩充

- 修改 `spirv_mapping.h` 与 `spirv_mapping.cpp` 中的 `BasicResourceMapping` 类
- 增加成员变量 `m_setIndex` 与 `m_bindingOffset`
- 改写 `mapDescriptor` 虚方法，采用指定的 `Set` 索引并将绑定槽位改为 `m_bindingOffset` 加上自增描述符索引
- 修改 `tools/dxbc_compiler.cpp` 中的 `Options` 结构，增加 `--set` 与 `--binding-shift` 命令行解析参数

### 构建与提取器逻辑改造

- 项目新增 `CMakeLists.txt` 替代 `Meson` 构建配置，生成 Release 可执行文件并覆盖至 `assets/tools/dxbc_compiler.exe`
- 修改 `examples/extract_shaders.rs` 中的 `process_toc` 函数
- 根据 `shader_toc.shader_type` 区分顶点着色器与像素着色器
- 顶点着色器转译参数附加 `--set 3`
- 像素着色器转译参数附加 `--set 3 --binding-shift 2`
- 使得顶点常量缓冲区分布在 `Set` 3 的 `Binding` 0 与 1，像素采样器与纹理平移映射在 `Set` 3 的 `Binding` 2 至 7

### Bevy 材质管线对齐

- 在 `ParticleMaterialQuad` 结构体中使用 `AsBindGroup` 宏声明槽位
- 顶点 Uniforms 绑定在 `Binding` 0
- 像素 Uniforms 绑定在 `Binding` 1
- 采样器分别绑定在 `Binding` 2 至 4
- 纹理分别绑定在 `Binding` 5 至 7

## DXBC RDEF 内存布局解析与符号导出

### RDEF 模块解析设计

- 新增 `dxbc_rdef.h` 与 `dxbc_rdef.cpp` 中的 `RdefParser` 模块
- 读取 `RDEF` 或 `RD11` Chunk，跳过 8 字节头部精确定位 Payload 指针
- 提取常量缓冲区的名称、字节大小、变量列表、起始字节偏移量 `startOffset` 与尺寸 `size`
- 提取资源绑定列表中的资源名称、绑定类型与绑定点 `bindPoint`

### 调试故障排查与修复阶段

#### 阶段 1 无名称
- 原版未解析 `RDEF` 块，默认导出匿名符号
- 修复方案为在 `dxbc_resources.cpp` 的 `emitDebugName` 中优先使用 `RDEF` 解析出的原始标识符替换默认编号

#### 阶段 2 顶级成员缺失
- 原版在 `emitDclCbv` 中为常量缓冲区外层套用了 `defStructWrapper` 包装结构体
- `spirv-reflect` 仅反射最外层结构体导致内部成员被隐藏
- 修复方案为剔除外层包装，直接暴露顶层 `Block` 结构体

#### 阶段 3 名字偏移走样
- 常量缓冲区包含 `float3` 与 `uint` 等非 16 字节成员
- 先前计算使用 `startOffset / 16` 作为成员索引导致累计走样
- 修复方案为在 `spirv_builder.cpp` 的 `emitDebugMemberNames` 中遍历 `structType->byteOffset(m)` 匹配真实物理字节偏移

#### 阶段 4 名称乱码前缀
- `Shader Model` 5 的 `RDEF` 变量记录结构体大小为 40 字节，先前按 `Shader Model` 4 的 24 字节步长遍历导致越界
- 先前误将操作数 1 的字节偏移量当作字符拼接入字符串
- 修复方案为在 `dxbc_rdef.cpp` 中检测 Shader Model 版本，版本高于 5.0 时按 40 字节步长解析变量记录

### 复合类型与未引用字段表现

- 矩阵类型如 `float4x4` 连续占用 4 个 16 字节槽位，`RDEF` 仅在起始点记录一次变量名，后续行由转译器自动推导
- 常量缓冲区中未被逻辑读写的字段在内存布局中仍保留物理槽位，并由验证工具标记为未引用

## 验证与调试结论

- 使用 `spirv-reflect.exe` 验证 `shader_0015.spv`，确认所有字段与绝对物理字节偏移精准对齐且无乱码
- 运行 `extract_shaders` 重新转换全量 256 个着色器
- 绑定红黄蓝绿测试图与纯白纯绿控制变量验证粒子平面渲染，材质调色盘逻辑与颜色混合在 `GPU` 上运行正常，验证层无报错
