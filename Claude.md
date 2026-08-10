# 目录

## 文档

- [游戏内容文档](docs/game/) - 包含 Bevy 游戏相关的设计与实现（如资产、动画、着色器、粒子、技能系统等）以及 156 个英雄的技能文档 (champions)。
- [产品业务文档](docs/product/) - 包含产品功能、计费、对局、平台及智能体 (agent) 等相关设计和任务规划。

# 术语

## 上单五虎

- aatrox（暗裔魔剑）
- darius（诺手）
- mordekaiser（莫德凯撒）
- sett（瑟提）
- volibear（沃里克）

## 上单四姐妹

- camille（青钢影）
- fiora（无双剑姬）
- irelia（刀锋舞者）
- riven（放逐之刃）

# 高频操作

## cargo check 检查

检查必须包括所有目标

```sh
cargo check --workspace --all-targets
```

# 项目结构

- `assets/props`: 从 WAD 中解包出的所有 PROP 文件，大约几万个

# Client 架构

client（`apps/client`，GPUI）自身零 bevy 依赖，bevy 功能全在子进程：

```text
client (gpui)
 ├─ WS ─────→ 游戏进程 moon_lol
 ├─ WS ─────→ 粒子进程 lol_particle
 └─ stdio ───→ 提取进程 lol_extractor
```

- 游戏：`ProcessService.start` → `GameProcessManager` spawn `moon_lol`，`--ws-port` 回连
- 粒子：纯 WS，发 RON 定义给 `lol_particle`
- 提取：`extractor_service.rs` spawn `lol_extractor`，stdout 每行 JSON 进度 `{"step","kind","msg"}`
- WAD 浏览器：进程内用 `league_loader`/`league_property`（bevy-free）

bevy 分布：`league_utils`/`league_property`/`league_loader` bevy-free；`league_to_lol` 及 `Data` trait（`league_to_lol::data`）保持 bevy-bound，仅提取/游戏栈用。

子进程二进制解析统一走 `lol_client::launch::resolve_executable(pkg, bin)`：dev `cargo run -p <pkg> --bin <bin>`，release 从 client 可执行文件同级目录解析。release 打包布局：`client.exe` 与 `moon_lol.exe`/`lol_extractor.exe`/`lol_rl_visual.exe` 及 `assets/` 放同一目录，cwd 用 `install_root()`。
