# UI 系统架构指南

本文档总结了 UI 系统的导出、加载及运行时逻辑。

## 1. 导出逻辑 (`league_to_lol`)

UI 导出负责将英雄联盟原始的 BIN 数据转换为 `lol_base` 定义的稳定数据结构。

### 核心流程：
- **精确提取**：使用 `PropFile::iter_entry_by_class` 方法，根据类名哈希值（Class Hash）精确匹配条目，避免了不同 UI 类型（如 Icon 和 Anim）因字段结构相似而导致的误判。
- **支持类别**：
    - `UiSceneData`：定义 UI 场景及层级关系。
    - `UiElementIconData`：基础图标，包含纹理和位置。
    - `UiElementGroupButtonData`：复合按钮，包含多个状态元素（默认、悬停、点击）。
    - `UiElementEffectAnimationData`：序列帧动画。
    - `UiElementRegionData`：逻辑区域，常用于定义交互热区。
    - `LOLUiElementTextData`：文本元素，包含字体、颜色和文本内容。
- **纹理处理**：收集所有元素引用的纹理路径，并将其导出为项目可识别的 `.png` 资源。

---

## 2. 加载与初始化 (`lol_render`)

在渲染层启动时，UI 会经历多阶段的初始化过程，以建立完整的运行时环境。

### 初始化阶段：
1. **Asset 注册**：从 `.ron` 文件中反序列化数据，并将所有 UI 元素注册为 Bevy Assets（`Assets<T>`）。
2. **Handle 填充**：由于导出数据只包含哈希 ID，系统在此时会通过哈希查找，为按钮元素手动填充 `hit_region_handle` 和 `element_handles`。
3. **实体映射**：
    - 为每个 UI 元素创建一个对应的 Bevy 实体。
    - 建立哈希到实体的映射表 (`LOLUiElementEntity`)，便于后续逻辑查找。
4. **层级树重建**：
    - 根据 `scene` 和 `parent_scene` 属性，使用 Bevy 的 `add_child` 将元素挂载到场景实体下，将场景挂载到父场景下。
    - 按钮内部包含的图标也会被嵌套在按钮实体之下。
5. **树形导出**：初始化完成后，会自动生成 `ui_tree.json` 文件，用于直观地查看当前加载的 UI 逻辑层级。

---

## 3. 运行时逻辑

运行时系统通过组件和系统驱动 UI 的交互与展示。

### 关键特性：
- **可见性继承**：UI 元素默认使用 `Visibility::Inherited`。通过切换父级场景实体的可见性，可以一次性控制整个场景及其子元素的显示与隐藏。
- **交互逻辑**：
    - `UIButton`：由专门的按钮系统监听 `Interaction` 事件，根据 `Pressed`、`Hovered`、`None` 状态切换对应实体的可见性。
- **动画驱动**：
    - `UiAnimationState`：存储动画播放进度。
    - 动画系统会根据 `Timer` 定期更新 `UIElement` 的 UV 坐标或纹理，实现平滑的序列帧播放。
- **Z 轴排序**：利用 `ZIndex` 组件处理元素间的遮挡关系。
