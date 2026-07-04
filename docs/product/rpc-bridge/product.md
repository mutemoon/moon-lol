# 游戏控制 RPC — 产品说明

## 一、定位

游戏进程对外暴露的控制能力（观测、下发动作、调试、RL 环境、脚本热重载）需要一个统一的 RPC 入口。历史实现用 Bevy observer + cmd 字符串手搓 match 分发，本子系统将其重构为 `CommandWsRequest<T>` 泛型事件 + 类型化入参 + 单一 dispatch 注册表。详见 [架构设计](arch.md)。

## 二、服务对象

| 调用方 | 形态 | 命令面 |
|---|---|---|
| CLI（开发者/运维） | `lol_cli` 薄前端 | 完整控制面 |
| rig Agent（LLM） | 进程内 MCP 工具层 | 仅 observe + action（防越权） |
| RL Agent | Gymnasium 环境 `MoonLoLEnv` | packed 高频微操 + rl_reset/rl_step |
| 浏览器/前端 | 经 web server 代理 | 观测 + 观战事件流 |
| Web server | 云端 match_supervisor | 完整控制面（托管对局） |

重构对调用方**完全透明**——`GameClient` 方法签名不变、MCP tool 不变、CLI 子命令不变、WS 协议帧不变。变的是服务端入口之后的分发机制。

## 三、能力分级

控制指令按越权风险分两级，由 dispatch 注册表的构建期裁剪保证：

- **Agent 面**（observe / action / set_script / observe_packed / action_packed / rl_reset / rl_step / get_agents）：所有调用方可用。
- **Debug 面**（switch_champion / god_mode / toggle_cooldown / reset_position / toggle_pause / set_speed / get_state）：仅 CLI 与 web server 可用。release 构建中 dispatch 注册表不含这些行，从协议层杜绝 agent 越权，取代现在仅在 MCP 层收窄的做法。
- **事件流**（game_loaded / champion_changed / entity_selected / match_event / game_close）：所有连接方可收，仅推送。

## 四、关键产品约束

1. **契约稳定**。`GameClient`、MCP tool、CLI 子命令、WS 帧的签名与语义不变，重构对调用方透明。
2. **高频路径不退化**。packed（msgpack+base64）观测/动作、rl_reset/rl_step 延迟不得高于现状。
3. **多局化就绪**。dispatch 在 per-world `poll_commands` 内运行，配合 [游戏进程托管](/docs/product/game-host/) 端口池，天然多实例。
4. **可回退**。分阶段迁移，旧字符串事件路径在最终删除前一直可用。

## 五、非目标

- 不改变对局三形态（本地/房间/Rank）的业务逻辑。
- 不改变 Agent 选手体系与 ELO 计算。
- 不引入新的传输框架（不切 jsonrpsee/axum），留在 Bevy event 机制内。
- 不重写 RL 环境与脚本驱动，仅改其与外界的指令分发通道。

## 六、关联文档

- [架构设计](arch.md) — 数据流、核心类型、dispatch 注册表、选型对比、迁移策略。
- [待办](todo.md) — 分阶段任务清单。
- [游戏工具](/docs/product/game-tools/) — CLI + MCP 共享 `lol_client`，本重构后其 arch.md「cmd 字符串分发」描述需同步更新。
- [游戏进程托管](/docs/product/game-host/) — 端口池与多实例，dispatch 的 per-world 形态与之对齐。
- [Agent 选手与决策体系](/docs/product/agent/) — 三种 Agent 类型如何消费 observe/action。
