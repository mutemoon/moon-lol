# 测试思考逻辑

## 核心规则

- 优先写系统行为测试，不要优先写常量抄写测试。
- 运行时如果依赖 Bevy `App`、插件、时间推进、事件、资源或 ECS 查询，测试也必须尽量走同一条真实路径。
- 对 `skill`、`action`、`controller`、`movement` 这类系统，测试目标是验证"玩家操作后世界状态如何变化"，不是验证"代码里写了什么"。
- 纯静态断言无法发现冷却、蓝耗、目标筛选、事件链、插件接线等真实回归。

## 技能测试规则

- 通用技能系统测试位于 `lol_core::skill::tests` / `integration_tests`。
- 英雄技能测试位于 `lol_champions::<hero>::tests`（`#[cfg(test)]` 条件编译）。
- 英雄测试不再拆分 `tests.rs` / `render_tests.rs`；headless 与 render 测试统一在 `tests.rs`，通过 harness 的 `.render()` 方法切换。
- 渲染录像测试产物位于 `assets/test_videos/<champion>/<test_name>.mp4`。
- `skill` 测试默认运行在无头 Bevy 实例里。
- 必须显式构造最小完整场景：施法者、敌方目标、友方目标，以及必要的 `Transform`、`Team`、`Health`、`Skills`、`SkillPoints`、`AbilityResource`。
- 必须显式注册技能依赖资产，如 `SpellObject`，不要依赖隐式全局状态。
- 输入必须走真实语义，优先使用 `CommandAction -> Action::Skill`、`Action::SkillLevelUp`，不要直接改内部组件假装测试通过。
- 对依赖 `FixedUpdate` 或 `Time` 的逻辑，必须使用固定帧率和手动时间推进，例如 `TimeUpdateStrategy::ManualDuration`。
- 一个合格的技能测试至少覆盖：初始场景、玩家输入、帧推进、最终世界状态。
- 断言优先看蓝量、冷却、位置、血量、Buff、目标筛选和副作用，不要优先看 enum 或硬编码常量。
- 纯数学或纯几何逻辑可以保留快单测，但只能作为补充，不能代替系统集成测试。

## Harness 规则

- 英雄测试统一使用 `crate::test_utils::ChampionTestHarness`，避免每个英雄模块重复实现 harness。
- 每个英雄模块只提供 `ChampionHarnessConfig`（champion_dir、config_path、skin_path、add_champion_plugin、mock spell 工厂、cooldown 模式、spell keys）。
- `ChampionTestHarness::build::<Champion>(name, mode, &config)` 负责：
  - 创建无头/渲染 `App`、安装必要插件、注册资产。
  - **Headless 模式**：使用 mock spell（快速、同步、无需 GPU/IO），通过 Startup 系统生成实体。
  - **Render 模式**：使用真实 `ConfigCharacterRecord` + `ConfigSkin`，通过 `AssetServer` 加载角色配置和皮肤。
  - 生成初始实体、提供"模拟输入""推进时间""查询世界状态"辅助方法。
- 这样每个测试只描述：给定什么场景、玩家做了什么、最终应发生什么。
- `build_render(name)`（内部传 `HarnessMode::Render`）创建渲染模式 harness，测试结束后自动调用 `finish()` 生成视频。

## 已落地经验

- `lol_core::skill::integration_tests` 是当前推荐方向：无头 Bevy `App` + `CommandAction` 输入 + 手动时间推进 + 场景目标断言。
- `lol_champions::test_utils::ChampionTestHarness` 是英雄测试的统一入口，消除了 Riven/Fiora 之间 ~80% 的 harness 重复代码。
- 不要猜 `assets/data/*.lol` 路径；更稳的做法是从英雄主 bin 读取，例如 `DATA/Characters/Riven/Riven.bin`，遍历 `SpellObject` 后按 `m_script_name` / `object_name` 筛选目标技能。
- 技能录像测试也必须走 `cargo test`，不要再依赖 `examples/*` 作为主入口。
- 每个 `#[test]` 都应独立产出自己的视频文件，不要把多个技能流程混成一个"英雄展示视频"。
- 录像插件应同时负责离屏渲染、抓帧、脚本驱动、视频后处理和最小导航网格注入，否则位移类技能只会播 `idle`。
- 录像目录应尽量只剩最终 `mp4`：帧图放 `frames/` 子目录，`ffmpeg` 成功后默认删除。
- 不要把一个测试文件里的多个 Bevy 录像测试放进同一个测试进程里一起跑后处理；统一入口应用逐测试用例单独执行。
- 根目录脚本 `npm run test:render` 现在就是统一入口：`MOON_LOL_RUN_RENDER_TESTS=1 cargo test -p lol_champions`。

## 真实资源测试策略

- 如果目标是验证"英雄技能是否正确"，优先使用本地已导出的真实资源。
- 如果目标是验证"系统行为是否正确"，可以继续使用最小手工资源。
- 两者不要混为一类测试。

### 优先级

- 英雄技能行为正确性测试：
  - 优先使用真实导出资源
  - 例如从 `DATA/Characters/Riven/Riven.bin`、`DATA/Characters/Fiora/Fiora.bin` 里提取真实 `SpellObject`
  - 或直接挂 `ConfigCharacterRecord` 组件，系统自动加载角色配置和技能

- 通用系统行为测试：
  - 使用最小手工资源
  - 例如蓝耗门禁、冷却门禁、范围筛选、Buff 链路

- 最新版本兼容性检查：
  - 允许依赖当前本地导出的全量资源
  - 但这类测试要接受上游资源变动带来的不稳定性

## 从真实资源取技能的规则

- 不要假设 `Characters/X/Spells/...` 一定能直接映射到某个 `assets/data/{hash}.lol` 文件。
- 优先从英雄主 bin 入手，例如 `DATA/Characters/Riven/Riven.bin`、`DATA/Characters/Fiora/Fiora.bin`、`DATA/Characters/Hwei/Hwei.bin`。
- 从主 bin 中解析 `PropFile`，过滤 `SpellObject`，再按 `m_script_name`、`object_name` 精确筛选技能。
- 这样更稳定，也更贴近真实运行时配置。

### Render 模式：ConfigCharacterRecord + ConfigSkin

- 渲染测试不再使用 mock spell + 彩色胶囊替代英雄模型。
- 通过 `ConfigCharacterRecord { character_record: asset_server.load("characters/riven/config.ron") }` 自动加载英雄所有技能组件。
- 通过 `ConfigSkin { skin: asset_server.load("characters/riven/skins/skin0.ron") }` 自动加载皮肤模型和动画。
- `PluginCharacter`（`lol_core::character`）在 `FixedUpdate` 中处理 `ConfigCharacterRecord`，将场景实体写入世界。
- 皮肤系统（`lol_render::skin`）在 `Update` 中处理 `ConfigSkin`。

## 资源路径和 hash 约束

- 运行时对 prop 路径做 hash 时，默认使用小写路径。
- 如果测试里需要按路径去定位导出文件，必须和运行时保持同样的小写规则。
- 但即使如此，也不要优先依赖"路径直接命中文件"这件事；优先走主 bin 反查。

## 断言真实资源时的注意事项

- 不要硬编码"某技能一定耗蓝""某技能一定有某个字段"。
- 真实资源里字段可能为空，例如 `mana: None`。
- 测试应优先读取真实资源，再根据真实值断言行为。
- 可以断言"释放后 mana 按资源中的 cost 扣除"，不要断言"释放后一定扣 20 蓝"。

## 对技能测试的推荐分层

- `lol_core::skill::tests` / `integration_tests`
  - 无头 Bevy + 最小手工资源，验证技能系统公共行为

- `lol_champions::<hero>::tests`（`#[cfg(test)]`）
  - 无头 Bevy + mock spell（headless）或 `ConfigCharacterRecord`（render）
  - 验证该英雄技能的核心逻辑
  - 统一使用 `ChampionTestHarness`，通过 `HarnessMode` 切换模式
  - 使用 `build_render(name)`（`HarnessMode::Render`）录制视频到 `assets/test_videos/<hero>/<test_name>.mp4`

- `lol_champions::test_utils`（`#[cfg(test)]`）
  - 共享 harness：`ChampionTestHarness`、`ChampionHarnessConfig`、`SpellKeySet`、`HarnessMode`
  - 共享工具函数：`make_test_grid`、`render_output_dir`、`skip_due_to_missing_gpu`、`run_render_test`

- 最新版本兼容性检查
  - 可以额外增加一层面向最新导出资源的检查测试
  - 但不要让所有基础开发都绑定在这层之上

## 写测试时的自检问题

- 这个测试是在验证真实行为，还是在重复抄实现细节？
- 如果技能插件失效，这个测试会失败吗？
- 如果事件链断了，这个测试会失败吗？
- 如果目标筛选错了，这个测试会失败吗？
- 如果冷却和蓝耗没生效，这个测试会失败吗？
- 如果这些问题的答案大多是否，那么测试还不够好。

---

## 技能系统测试策略

技能系统必须优先保证在测试中可验证。

当前测试关注三层：

### 1. 核心技能单测 + 管线集成测试

位于 `crates/lol_core/src/skill/`：

- `tests.rs` — spell 数值计算单测 + 管线集成测试
- `integration_tests.rs` — 技能系统公共行为集成测试（冷却、蓝耗、重施窗口等）
- `test_utils.rs` — 共享 headless 测试 harness（`test_app()`、`spawn_caster()` 等）

覆盖内容：

- 升级规则
- 资源门槛
- 施法日志
- observer 技能执行
- `SkillRecastWindow` 生命周期
- 手动冷却逻辑
- 通过 `Action` 输入走完整施法管线

### 2. 英雄逻辑测试

位于 `crates/lol_champions/src/<hero>/tests.rs`（条件编译 `#[cfg(test)]`）：

- headless Bevy + mock spell 或真实 `ConfigCharacterRecord`
- 统一使用 `ChampionTestHarness`（定义在 `crates/lol_champions/src/test_utils.rs`）
- 验证该英雄技能的真实行为

当前实现参考 `crates/lol_champions/src/riven/tests.rs`：

### 3. 英雄渲染测试

使用 `build_render()` 创建（参考 `riven_q_cycles_through_three_real_stages`），`HarnessMode::Render` 同时验证逻辑行为并录制视频。

覆盖内容：

- 离屏渲染 + ffmpeg 视频后处理
- 每个 `#[test]` 只录一个技能行为
- 产物输出到 `assets/test_videos/<champion>/<test_name>.mp4`

### 可测试性标准

一个技能设计如果做不到下面几点，通常说明实现方式不够好：

- 能在 headless `App` 中直接施放
- 能直接断言技能实体状态
- 能直接断言施法失败原因
- 不依赖渲染即可验证技能结果
- 测试应放在对应英雄模块的 `#[cfg(test)] mod tests` 中
