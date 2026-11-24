# apps/particle

粒子浏览器：Bevy 渲染（WASM）+ React 检查器（`web/`）两层。

```text
apps/particle
├─ src/main.rs          Bevy WASM 渲染端，接收 RON 定义并播放
├─ src/bin/export_vfx.rs 从 assets/characters 导出 web 用的数据
└─ web/                 React + Vite 检查器（UI、RON 弹窗、发射器属性面板）
```

# web/public/data 是生成产物，不提交

`web/public/data/`（约 200 个英雄 / 200+ MB）不是手写数据，全部由 `export_vfx` 从
`assets/characters/*/skins/skin0_vfx.ron` 解析生成，因此已被 `.gitignore` 忽略。
拉取仓库后必须自己生成一次，否则 `pnpm dev` 首屏只会报 `加载 heroes_catalog.json 失败`。

## 生成

```sh
cargo run -p particle --bin export_vfx
```

- 输入：`assets/characters/<Champion>/skins/skin0_vfx.ron`（缺失的英雄会被跳过）
- 输出：
  - `web/public/data/heroes_catalog.json`：英雄目录（id、name、category、count、系统摘要列表）
  - `web/public/data/heroes/<champion>.json`：每个英雄的全部粒子系统（`ron_str` + 解析后的 `def`）
- 英雄分类（`five_tigers` / `four_sisters` / `other`）写死在 `export_vfx.rs::categorize_hero`，
  要改分组就改代码再重跑
- 全量重建，不增量：资产更新后直接重跑覆盖

## 注意

- **不要读取或编辑 `web/public/data` 里的 JSON**：单个英雄文件约 1 MB、纯单行 JSON，读进上下文会瞬间爆掉。
  需要看结构时只看 `src/bin/export_vfx.rs` 里的 `WebHeroSummary` / `WebHeroDetail` / `WebParticleSystem` 定义。
- `ron_str` 与 `def` 是同一份数据的两种表示（前者喂给 WASM，后者给 React 面板），体积因此翻倍。
- 排查「数据缺失 / 英雄不出现」先重跑生成命令，再看 `assets/characters` 下是否存在该英雄的 `skin0_vfx.ron`。
