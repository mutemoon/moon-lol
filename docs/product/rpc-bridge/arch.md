# 游戏控制 RPC — 架构设计

本文档描述把游戏进程的控制指令分发从「Bevy observer + cmd 字符串手搓 match」迁移到「`CommandWsRequest<T>` 泛型事件 + 类型化入参 + 单一 dispatch 注册表」的方案。目标是消除双重分发与类型擦除，handler 类型化、样板归零，同时留在 Bevy 内不引入额外传输框架。

## 一、现状与问题

### 1. 现状数据流

```
WS client (lol_client::WsSession / 浏览器)
        │  WsRequest { id, cmd: String, params: Value }
        ▼
lol_server::server::start  (tokio 后台线程，accept loop)
        │  async_channel (id, cmd, params)
        ▼
lol_server::server::poll_commands  (Bevy Update system)
        │  handlers::dispatch(world, id, cmd, params)
        │  world.trigger(CommandWsRequest { id, cmd, params, response })
        ▼
Bevy observer  (CommandWsRequest 是 Event)
  ├─ lol_agent::systems::on_command_ws_request   match cmd { ... _ => return }
  └─ lol_debug::on_command_ws_request            match cmd { ... _ => return }
        │  event.response.lock() = WsResponse::ok_with_data / err
        ▼
poll_commands 回写 out_tx → WS client
```

权威文件：
- [server.rs](/crates/lol_server/src/server.rs) — WS accept loop、channel、poll_commands
- [handlers.rs](/crates/lol_server/src/handlers.rs) — dispatch：trigger 事件 + 等 response
- [events.rs](/crates/lol_server/src/events.rs) — CommandWsRequest 事件
- [systems.rs](/crates/lol_agent/src/systems.rs) — agent 侧 match 分发
- [lib.rs](/crates/lol_debug/src/lib.rs) — debug 侧 match 分发
- [protocol.rs](/crates/lol_server/src/protocol.rs) — cmd 常量 + WsRequest/WsResponse
- [game_client.rs](/crates/lol_client/src/game_client.rs) — 类型化命令面

### 2. 问题清单

1. **双重分发**。`CommandWsRequest { cmd: String, params: Value }` 是单一事件类型，`world.trigger` 会**同时**触发 `lol_agent` 与 `lol_debug` 两个 observer，每个再各自 `match cmd` 做字符串比较、`_ => return`。Bevy 的 event 机制在这里只承担了「广播给所有插件」，并未按 cmd 路由——第二层 match 是手搓补上的。
2. **类型擦除**。`params: serde_json::Value` 一路裸传，handler 里手动 `from_value` / `params.get("entity_id").and_then(as_u64)`，无编译期保证，每个 arm 重复解析。
3. **样板重复**。每个 arm 重复 `event.response.lock()` + `WsResponse::ok_with_data / err`，并用 `(|| -> Result<_,_> {})()` IIFE 换 `?`，丑且不可单测。
4. **entity_id 解析重复**。`CMD_OBSERVE` / `CMD_ACTION` 内联了一遍，而 `resolve_target` 已存在却没用全。
5. **高频路径重复**。`CMD_OBSERVE_PACKED` / `CMD_RL_RESET` / `CMD_RL_STEP` 三处重复 observe→pack→b64。
6. **闭合耦合分散**。每条指令的 cmd 字符串常量、match arm、GameClient 方法、MCP tool 散落四处，加指令要改全。
7. **响应与帧耦合**。`response: Arc<Mutex<Option<WsResponse>>>` 由 observer 在 `trigger` 内同步填充——能用，但这是「用 Bevy event 跑请求/响应 RPC」的固有形态，本方案接受不改。

## 二、方案：CommandWsRequest<T>

### 1. 核心思路

把单一字符串事件换成**泛型事件** `CommandWsRequest<T>`，`T` 是每条指令的类型化入参。`world.trigger(CommandWsRequest::<ObserveParams>{...})` 只命中 `On<CommandWsRequest<ObserveParams>>` observer——Bevy 的 typed event 天然按类型路由，双重分发塌缩成一层。字符串→类型的映射集中到 `dispatch` 一处，做成声明式注册表。

### 2. 核心类型

```rust
// crates/lol_rpc/src/lib.rs  —— 契约单一事实源
use std::sync::{Arc, Mutex};
use bevy::prelude::*;

#[derive(Event)]
pub struct CommandWsRequest<T> {
    pub id: u64,
    pub params: T,
    pub response: Arc<Mutex<Option<WsResponse>>>,
}

// ── 每条指令一个入参类型（Deserialize 自动解析）──
#[derive(Deserialize)]
pub struct ObserveParams   { pub entity_id: Option<u64>, pub json: bool }
#[derive(Deserialize)]
pub struct ActionParams    { pub entity_id: Option<u64>, pub action: Action }
#[derive(Deserialize)]
pub struct SetScriptParams { pub entity_id: u64, pub source: String }
#[derive(Deserialize)]
pub struct ObservePackedParams { pub entity_id: Option<u64> }
#[derive(Deserialize)]
pub struct ActionPackedParams  { pub entity_id: Option<u64>, pub msgpack_b64: String }
#[derive(Deserialize)]
pub struct RlResetParams { pub entity_id: Option<u64>, pub config_json: Option<Value> }
#[derive(Deserialize)]
pub struct RlStepParams  { pub entity_id: Option<u64> }
#[derive(Deserialize)]
pub struct GetAgentsParams; // 无参，unit struct
// ── debug 面（debug 构建才注册到 dispatch）──
#[derive(Deserialize)]
pub struct SwitchChampionParams { pub name: String }
#[derive(Deserialize)]
pub struct GodModeParams        { pub enabled: bool }
// ... toggle_cooldown / reset_position / toggle_pause / set_speed / get_state
```

### 3. 声明式 dispatch 注册表

把字符串→类型映射压成一张扁平表，宏展开成 match，加指令 = 加一行：

```rust
// crates/lol_rpc/src/dispatch.rs
pub fn dispatch(world: &mut World, id: u64, cmd: &str, params: Value) -> WsResponse {
    let response: Arc<Mutex<Option<WsResponse>>> = Arc::new(Mutex::new(None));
    rpc_dispatch!(world, id, cmd, params, &response {
        "observe"         => ObserveParams,
        "action"          => ActionParams,
        "set_script"      => SetScriptParams,
        "observe_packed"  => ObservePackedParams,
        "action_packed"   => ActionPackedParams,
        "rl_reset"        => RlResetParams,
        "rl_step"         => RlStepParams,
        "get_agents"      => GetAgentsParams,
        // ── debug 面（cfg(debug_assertions) 或 feature）──
        "switch_champion"  => SwitchChampionParams,
        "god_mode"         => GodModeParams,
        "toggle_cooldown"  => ToggleCooldownParams,
        "reset_position"   => ResetPositionParams,
        "toggle_pause"     => TogglePauseParams,
        "set_speed"        => SetSpeedParams,
        "get_state"        => GetStateParams,
    });
    response.lock().unwrap().clone()
        .unwrap_or_else(|| WsResponse::err(id, format!("未知指令: {cmd}")))
}

// 宏展开为：
//   "observe" => { trigger::<ObserveParams>(world, id, params, &response) }
// 其中 trigger 反序列化 params → 失败回填 err → 成功 world.trigger(CommandWsRequest{...})
fn trigger<T: DeserializeOwned + Send + Sync + 'static>(
    world: &mut World, id: u64, params: Value, response: &Arc<Mutex<Option<WsResponse>>>,
) {
    let params: T = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => { *response.lock().unwrap() = Some(WsResponse::err(id, format!("无效参数: {e}"))); return; }
    };
    world.trigger(CommandWsRequest { id, params, response: response.clone() });
}
```

注册表就是 RPC 契约的单一事实源——cmd 名、入参类型、debug 面裁剪全在这张表里。debug 面 `cfg` 门控：release 构建注册表不含作弊指令，从 dispatch 层杜绝 agent 越权（取代现在仅在 MCP 层收窄）。

### 4. 类型化 observer（无 guard、无 Value 解析、可单测）

```rust
// crates/lol_agent/src/systems.rs
fn on_observe(
    event: On<CommandWsRequest<ObserveParams>>,
    q: Queries,
    time: Res<Time>,
) {
    respond(&event, handle_observe(&event.params, &q, time.elapsed_secs()));
}

// 纯函数，可单测，告别 IIFE
fn handle_observe(p: &ObserveParams, q: &Queries, time: f32) -> Result<Value, String> {
    let target = resolve_target(&q.player, p.entity_id)?;
    let obs = observe(target, q, time).ok_or("无法获取当前游戏局势观测数据")?;
    if p.json { to_value(obs).map_err(|e| e.to_string()) }
    else { Ok(Value::String(format_observation(&obs))) }
}

// 统一响应回填，泛型 over T
fn respond<T>(_event: &On<CommandWsRequest<T>>, result: Result<Value, String>) {
    // event.response 通过 On 取引用，回填 WsResponse::ok_with_data / err
}
```

observer 不再做字符串比较、不再 `from_value`、不再手写 response 样板。`resolve_target` 收敛一处，observe/action/packed/rl 全用。`observe_packed_bytes` 抽出，packed/rl_reset/rl_step 三处复用。

### 5. 注册

每个插件只挂自己负责的 observer，互不感知：

```rust
// lol_agent
app.add_observer(on_observe)
   .add_observer(on_action)
   .add_observer(on_set_script)
   .add_observer(on_observe_packed)
   .add_observer(on_action_packed)
   .add_observer(on_rl_reset)
   .add_observer(on_rl_step)
   .add_observer(on_get_agents);

// lol_debug（debug 构建）
app.add_observer(on_switch_champion)
   .add_observer(on_god_mode)
   ...;
```

加一条指令 = 写入参 struct + 注册表加一行 + 写 observer/handler。不碰任何中心化 match 手写代码。

## 三、问题解决对照

| 问题 | 解决? | 说明 |
|---|---|---|
| 1 双重分发 | ✅ | typed event 按 T 路由，只命中匹配 observer |
| 2 闭合耦合 | ✅ | 收敛为 dispatch 注册表单一事实源 |
| 3 response 样板/IIFE | ✅ | `respond()` helper + 拆纯函数 |
| 4 entity_id 重复 | ✅ | `resolve_target` 收敛 |
| 5 packed 重复 | ✅ | `observe_packed_bytes` 抽出 |
| 6 类型擦除 | ✅ | params 在 T 里类型化 |
| 7 响应与帧耦合 | ⚠️ 保留 | observer 在 trigger 内同步回填，能跑；不引入 channel 故不改 |

## 四、选型对比

- **CommandWsRequest<T>（本方案）**：留在 Bevy，typed event 路由，dispatch 注册表。解决 1/2/3/4/5/6，接受 7。改动集中在 `lol_server`/`lol_agent`/`lol_debug`，不碰传输层。
- **jsonrpsee WS RPC + channel 桥**：RPC 边界与游戏世界结构性分离，额外解决 7，handler 在 tokio 侧类型化。但需重写传输层（jsonrpsee 接管 WS）、引入 mpsc/oneshot 配对、ECS 侧 `&mut World` 手动 query 丢 SystemParam 注入糖。代价远大于本方案，仅当认同「WS 命令直插 Bevy world 这一耦合本身要切开」才值得——本方案不采纳，留作未来若多局化/进程托管需要时的演进方向。
- **axum**：协议是请求/响应 + 事件推送非 REST，axum 的 WS 不帮做 method 路由，得自己 match，回到痛点。不选。
- **gRPC/tonic**：传输是 WS+JSON，形状不匹配。不选。
- **inventory/linkme 自动注册**：~15 条指令规模，自动注册的魔改感与调试难抵不过省下的一行注册表。不选。

## 五、crate 与文件映射

新增 `crates/lol_rpc`：RPC 契约单一事实源（泛型事件、入参类型、dispatch 注册表、`respond`/`resolve_target`/`observe_packed_bytes` 共享 helper）。

| 层 | 现状 | 去向 |
|---|---|---|
| `CommandWsRequest { cmd, params }` | `events.rs` | `CommandWsRequest<T>`，params 类型化 |
| `dispatch` | `handlers.rs` trigger 字符串事件 | `lol_rpc::dispatch` 声明式注册表 |
| `CMD_*` 常量 | `protocol.rs` | 删除（cmd 名在注册表里） |
| agent match 分发 | `lol_agent/src/systems.rs::on_command_ws_request` | 8 个 typed observer + 纯函数 handler |
| debug match 分发 | `lol_debug/src/lib.rs::on_command_ws_request` | 7 个 typed observer + 纯函数 handler |
| `WsSession` / `GameClient` | `lol_client` | 不变（仍是 cmd 字符串客户端，契约不变） |
| `WsEvent` / `WsResponse` | `protocol.rs` | 保留 |

客户端 `lol_client` 完全不动——它发的仍是 `{ id, cmd: String, params }`，dispatch 在服务端入口做字符串→类型映射。调用方零感知。

## 六、迁移策略

改动比 jsonrpsee 桥小一个量级，但仍分阶段、可回退。

1. **建 `lol_rpc` 骨架**。定义 `CommandWsRequest<T>`、入参类型、`trigger` helper、`respond`/`resolve_target`/`observe_packed_bytes`。`cargo check` 通过。
2. **建 dispatch 注册表**。`rpc_dispatch!` 宏 + `dispatch()`，在 `poll_commands` 替换旧 `handlers::dispatch`。此阶段 observer 还没拆，新事件类型尚无 observer——先让 dispatch 能 trigger，observer 侧并行迁移。
3. **迁 agent 面**。8 条指令拆 typed observer + 纯函数 handler，接 `On<CommandWsRequest<XxxParams>>`。为纯函数写单测。
4. **迁 debug 面**。7 条指令同理，debug `cfg` 门控。
5. **删旧路径**。移除旧 `CommandWsRequest` 字符串事件、`CMD_*` 常量、旧 observer、IIFE。更新 [game-tools/arch.md](/docs/product/game-tools/arch.md)「cmd 字符串分发」描述与 [产品架构总览](/docs/product/CLAUDE.md) crate 分层。

每阶段独立 `cargo check --all-targets`。旧路径在 step 5 前可用作回退。

## 七、风险与对策

- **泛型 Event 注册**。`CommandWsRequest<T>` 每个具体 T 是独立事件类型；observer 经 `add_observer` + `world.trigger` 工作，无需 `add_event`（buffered queue 才需要）。确认 Bevy 版本支持泛型 observer event。
- **debug 面越权**。dispatch 注册表 `cfg(debug_assertions)` 门控 debug 行，release 不注册对应 cmd，从协议层杜绝 agent 越权。
- **deferred 操作**。`commands.trigger(CommandAction{...})` 需 `flush` 当帧生效，每分支末尾固定 `world.flush`（或用 `Commands` system param 自动 flush）。
- **多局化**。dispatch 在 `poll_commands` 内、per-world 运行，天然多实例。
- **客户端契约**。`lol_client` 发的 cmd 字符串与注册表 key 必须一致——注册表即契约，改动需同步 GameClient。可在 `lol_rpc` 暴露 cmd 名常量供 GameClient 引用，避免漂移。
