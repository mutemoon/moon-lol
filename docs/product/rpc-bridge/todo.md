# 游戏控制 RPC — 待办

分阶段、可回退。每阶段独立 `cargo check --all-targets` 通过，旧字符串事件路径在阶段五前一直可用作回退。详见 [架构设计](arch.md) 第六节。

## 阶段一：建 lol_rpc 骨架

- [ ] 新增 `crates/lol_rpc`，Cargo.toml 声明依赖
- [ ] `lib.rs`：定义 `CommandWsRequest<T>` 泛型事件（`#[derive(Event)]`，含 `id` / `params: T` / `response: Arc<Mutex<Option<WsResponse>>>`）
- [ ] 定义入参类型：`ObserveParams` / `ActionParams` / `SetScriptParams` / `ObservePackedParams` / `ActionPackedParams` / `RlResetParams` / `RlStepParams` / `GetAgentsParams`，以及 debug 面 `SwitchChampionParams` / `GodModeParams` / `ToggleCooldownParams` / `ResetPositionParams` / `TogglePauseParams` / `SetSpeedParams` / `GetStateParams`
- [ ] 共享 helper：`trigger<T>()`、`respond<T>()`、`resolve_target()`、`observe_packed_bytes()`
- [ ] 确认 Bevy 版本支持泛型 `#[derive(Event)]` + `On<CommandWsRequest<T>>` observer
- [ ] `cargo check --all-targets` 通过（无业务接入）

## 阶段二：dispatch 注册表

- [ ] `dispatch.rs`：声明式 `rpc_dispatch!` 宏 + `dispatch()`，展开为字符串→`trigger::<T>` 的 match
- [ ] debug 面 `cfg(debug_assertions)` 门控（release 注册表不含作弊指令）
- [ ] `poll_commands` 改调 `lol_rpc::dispatch`，替换旧 `handlers::dispatch`
- [ ] 暴露 cmd 名常量供 `GameClient` 引用，防漂移
- [ ] 此阶段 observer 尚未拆，新事件类型暂无 observer——确认 dispatch 能 trigger 即可
- [ ] `cargo check --all-targets` 通过；旧 observer 仍工作（旧字符串事件保留）

## 阶段三：迁 agent 面

- [ ] `lol_agent/src/systems.rs`：8 条指令拆 typed observer + 纯函数 handler
  - on_observe / on_action / on_set_script / on_observe_packed / on_action_packed / on_rl_reset / on_rl_step / on_get_agents
- [ ] handler 拆纯函数：`handle_observe` 等，入参 `&XxxParams`，可单测
- [ ] `resolve_target` 收敛一处，observe/action/packed/rl 全用
- [ ] `observe_packed_bytes` 抽出，packed/rl_reset/rl_step 三处复用
- [ ] deferred 操作每分支末尾 `world.flush`
- [ ] `app.add_observer(...)` 注册 8 个 observer
- [ ] 旧 `on_command_ws_request` observer 保留，标记 deprecated
- [ ] 为纯函数 handler 写单测
- [ ] `cargo check --all-targets` + `cargo test` 通过

## 阶段四：迁 debug 面

- [ ] `lol_debug/src/lib.rs`：7 条指令拆 typed observer + 纯函数 handler
- [ ] `GlobalDebugState` 资源保留，handler 读写之
- [ ] debug observer 仅 debug 构建注册
- [ ] 旧 debug observer 标记 deprecated
- [ ] `cargo check --all-targets` 通过

## 阶段五：删旧路径

- [ ] 删旧 `CommandWsRequest { cmd, params }`（events.rs）
- [ ] 删 `CMD_*` 常量（protocol.rs，保留 `WsEvent` / `WsResponse`）
- [ ] 删 `handlers::dispatch`（handlers.rs）
- [ ] 删 `lol_agent` / `lol_debug` 的 deprecated observer
- [ ] 更新 [game-tools/arch.md](/docs/product/game-tools/arch.md)「cmd 字符串分发」描述与数据流图
- [ ] 更新 [产品架构总览](/docs/product/CLAUDE.md) crate 分层（补 `lol_rpc`）
- [ ] 全量 `cargo check --all-targets` + `cargo test` 通过

## 验证清单

- [ ] agent 面 8 条指令经新 dispatch 往返正确
- [ ] debug 面 7 条指令经新 dispatch 往返正确（仅 debug 构建）
- [ ] release 构建注册表无 debug 指令（agent 越权防护）
- [ ] 泛型 typed event 只命中匹配 observer（无双分发）
- [ ] packed 高频路径延迟不退化
- [ ] 多 Bevy 实例并存时 dispatch 互不串扰
- [ ] CLI / MCP / web server 三处调用方契约不变
- [ ] 纯函数 handler 单测通过
