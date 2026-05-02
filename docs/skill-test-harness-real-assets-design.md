# 测试 Harness 真实资源设计

## 状态

- 日期：2026-05-02
- 状态：已批准，待实现

## 目标

将 `ChampionTestHarness` 从 mock 技能数据统一为使用真实导出的 champion 配置和技能资源，保留 Headless / Render 两种运行模式。

## 背景

当前 `build_with_mocks` 使用手工构造的 `Spell` 结构体，所有技能计算数据（`calculations`、`dataValues`、`mana` 等）为空。Headless 测试无法验证真实技能数据。

`build_with_real_assets` 通过 `ConfigCharacterRecord` 加载真实 `config.ron`，但存在两个问题：
1. Headless 和 Render 模式代码路径完全不共享
2. Render 模式下，技能资源（`Spell` asset）异步加载未等待完成，导致 `h.spell(0)` 返回 `None`

## 设计

### ChampionHarnessConfig 简化

```rust
pub struct ChampionHarnessConfig {
    pub champion_dir: &'static str,
    pub config_path: &'static str,   // e.g. "characters/riven/config.ron"
    pub skin_path: &'static str,     // e.g. "characters/riven/skins/skin0.ron"
    pub add_champion_plugin: fn(&mut App),
}
```

删除：
- `make_mock_spell: fn() -> Spell`
- `cooldown_mode_for: fn(SkillSlot) -> SkillCooldownMode`
- `spell_keys: SpellKeySet`

### build 函数统一

将 `build_with_mocks` 和 `build_with_real_assets` 合并为统一的 `build` 函数：

```rust
pub fn build<C: Component + Default + Send + Sync + 'static>(
    test_name: &str,
    mode: HarnessMode,
    config: &ChampionHarnessConfig,
) -> Self
```

根据 `mode` 决定插件集，资源加载路径统一。

### 模式差异

| | Headless | Render |
|---|---|---|
| 插件 | `MinimalPlugins` + `AssetPlugin` + `InputPlugin` + `ScenePlugin` + `PickingPlugin` | `DefaultPlugins.build().disable::<WinitPlugin>()` + `PluginSkillTestRender` |
| 敌人/友军 | 无渲染组件（纯逻辑） | `Mesh3d` + `MeshMaterial3d` + 光照 + 平台 |
| 视频输出 | 无 | `SkillTestVideoOutput` |
| Settle frames | 15 帧 | 15 帧 |

### 资源加载流程（统一）

1. 调用 `AssetServer::load(config_path)` 加载 `DynamicWorld`
2. Spawn champion entity with `ConfigCharacterRecord { character_record: config_handle }` + `ConfigSkin { skin: skin_handle }`
3. 轮询直到 `ConfigCharacterRecord` 和 `ConfigSkin` 被移除（表示反序列化完成）
4. 收集所有技能实体的 `spell: Handle<Spell>` 的 `AssetId`
5. 轮询直到所有技能资源的 `AssetId` 出现在 `Assets<Spell>` 中（超时 1000 帧）
6. 覆盖属性（`Health`、`AbilityResource`、`Level`、`Damage`、`Armor`、`SkillPoints`）
7. 设置初始冷却时间
8. Spawn 敌人/友军实体
9. 面向 X+ 方向，settle 15 帧

### 技能资源轮询

```rust
// 收集所有技能资源的 AssetId
let spell_ids: Vec<AssetId> = skill_entities
    .iter()
    .filter_map(|&se| {
        app.world()
            .get::<Skill>(se)
            .map(|s| s.spell.id())
    })
    .collect();

// 轮询直到所有技能资源加载完成
for _ in 0..1000 {
    app.update();
    if spell_ids.iter().all(|id| app.world().resource::<Assets<Spell>>().contains_id(*id)) {
        break;
    }
}
```

超时（1000 帧 ≈ 16 秒）时 panic，提示资源文件缺失。

### test_utils.rs 变更摘要

| 变更 | 说明 |
|---|---|
| 删除 `SpellKeySet` 结构体 | 不再需要 |
| 删除 `ChampionHarnessConfig.make_mock_spell` | 不再需要 |
| 删除 `ChampionHarnessConfig.cooldown_mode_for` | 不再需要 |
| 删除 `ChampionHarnessConfig.spell_keys` | 不再需要 |
| 删除 `build_with_mocks` 函数 | 合并到 `build` |
| 重命名 `build_with_real_assets` 为 `build` | 统一入口 |
| `build` 接收 `mode: HarnessMode` 参数 | 决定插件集 |
| Headless 模式也使用 `ConfigCharacterRecord` 加载 | 真实资源 |
| 添加技能资源轮询 | 修复 `spell()` 返回 `None` |
| Headless 敌人/友军无渲染组件 | 无变更 |

## 影响

- 使用 harness 的现有测试（如 `riven/tests.rs`）需要更新 `ChampionHarnessConfig` — 删除 `make_mock_spell`、`cooldown_mode_for`、`spell_keys`
- `cooldown_mode_for` 逻辑丢失 — 如果特定 champion 的技能有特殊冷却模式（如 Riven Q 的 `Manual`），需要从 `config.ron` 的 `Skill.cooldown_mode` 字段读取，或在 champion plugin 中设置
- Headless 测试运行速度可能略微下降（真实资源加载有开销），但仍远快于 Render 模式
