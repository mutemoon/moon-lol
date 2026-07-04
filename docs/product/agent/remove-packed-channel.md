# 删除 packed 二进制传输通道

## 一、背景与动机

RL 训练通道现有一套 msgpack + base64 的二进制编码，经 `observe_packed` / `action_packed` 两个 WS 命令收发，`rl_reset` / `rl_step` 的观测返回也走同一套编码。

这套编码是过早优化，理由：

- 传输瓶颈不在序列化格式。RL 单步吞吐受限于 Bevy `FixedUpdate` 环境 step、策略推理、以及每步一次的 WS 往返延迟，序列化格式排在这些之后很远，可忽略。
- 编码没真正换成二进制传输。外层 `WsRequest` / `WsResponse` 的 `params` / `data` 仍是 `serde_json::Value`，msgpack 编码后又 base64 包成字符串塞进 JSON。base64 膨胀 33%，把 msgpack 的体积优势吃掉大半，对 `Observe` 这种小字段密集结构体，`base64(msgpack)` 经常比纯 JSON 还大。
- 无损数值的理由也不成立。`observe` 传 `json: true` 返回的 `serde_json` 序列化 f32 同样无损，packed 用的还是同一个 `Observe` 结构体，只是换了序列化器。
- `observe_packed` / `action_packed` 两个 client 方法无任何调用方，是死代码。

结论：整条 msgpack 传输编码连同两个 packed 命令一并移除，`rl_reset` / `rl_step` 的观测返回改走普通 JSON。RL 业务逻辑（`MoonLoLEnv` / `RewardShaper` / `StepResult` / `RlEnvs`）全部保留。

## 二、删除边界

| 保留 | 删除 |
|---|---|
| `rl_reset` / `rl_step` 命令 | `observe_packed` / `action_packed` 命令 |
| `MoonLoLEnv` / `RewardShaper` / `StepResult` / `RlEnvs` | msgpack + base64 编解码函数 |
| `Observe` / `Action` 的 JSON 序列化路径 | `rmp-serde` / `base64` 依赖 |

`rl_reset` / `rl_step` 的返回从 `observation_b64`（msgpack + base64 字符串）改为 `observation`（JSON 对象），由 `serde_json::to_value(&obs)` 直接序列化 `Observe`。

## 三、逐文件改动清单

### 1. lol_agent

**删除整个文件**

- `crates/lol_agent/src/systems/rpc/observe_packed.rs`
- `crates/lol_agent/src/systems/rpc/action_packed.rs`

**`crates/lol_agent/src/systems/rpc/mod.rs`**

- 删 `pub mod action_packed;` 与 `pub mod observe_packed;`
- 删 `pub use action_packed::on_action_packed;` 与 `pub use observe_packed::on_observe_packed;`

**`crates/lol_agent/src/params.rs`**

- 删 `ObservePackedParams` 与 `ActionPackedParams` 两个结构体

**`crates/lol_agent/src/lib.rs`**

- 删 `app.register_rpc::<ObservePackedParams>("observe_packed");`
- 删 `app.register_rpc::<ActionPackedParams>("action_packed");`
- 删 `.add_observer(on_observe_packed)` 与 `.add_observer(on_action_packed)`
- 清理因此不再使用的 import

**`crates/lol_agent/src/systems/rpc/rl_reset.rs`**

- 删 `let bytes = lol_rpc::observe_packed_bytes(&obs)?;`
- 返回值由 `json!({ "observation_b64": lol_rpc::b64_encode(&bytes) })` 改为 `json!({ "observation": serde_json::to_value(&obs).map_err(|e| e.to_string())? })`

**`crates/lol_agent/src/systems/rpc/rl_step.rs`**

- 删 `let bytes = lol_rpc::observe_packed_bytes(&obs)?;`
- 删 `map.insert("observation_b64".into(), json!(lol_rpc::b64_encode(&bytes)));`
- 改为 `map.insert("observation".into(), serde_json::to_value(&obs).map_err(|e| e.to_string())?);`
- 清理不再使用的 `lol_rpc` import

**`crates/lol_agent/src/rl.rs`**

- 删第 232 至 264 行整段 msgpack / base64 编解码：`pack_observe` / `unpack_observe` / `pack_action` / `unpack_action` / `b64_encode` / `b64_decode`
- 删 `use base64::Engine;`，删 `use lol_core::action::Action;`（仅 `pack_action` / `unpack_action` 使用）
- 删测试 `observe_msgpack_round_trip` 与 `action_msgpack_round_trip`
- 改模块文档第 6 行：去掉「msgpack 编解码 + base64 包装」那条能力描述
- 改模块文档第 8 至 9 行：环境交互指令清单去掉 `observe_packed` / `action_packed`
- 改第 182 行 `MoonLoLEnv` 文档：`action_packed` 引用改为 `action`

**`crates/lol_agent/Cargo.toml`**

- 删 `rmp-serde = "1.3.1"` 与 `base64 = "0.22"`（删完 `rl.rs` 编解码后仅剩 `rl.rs` 曾使用，确认无其他引用后移除）

### 2. lol_client

**`crates/lol_client/src/protocol.rs`**

- 删 `pub const CMD_OBSERVE_PACKED: &str = "observe_packed";`
- 删 `pub const CMD_ACTION_PACKED: &str = "action_packed";`

**`crates/lol_client/src/game_client.rs`**

- 删 `observe_packed` 方法
- 删 `action_packed` 方法

### 3. lol_rpc

**`crates/lol_rpc/src/lib.rs`**

- 删 `observe_packed_bytes` / `b64_encode` / `b64_decode` 三个函数
- 删对应 `test_resolve_target` 之外涉及这三个函数的测试（当前无，确认即可）
- 清理因此不再使用的 import（`base64` / `rmp_serde` / `serde::Serialize`）

**`crates/lol_rpc/Cargo.toml`**

- 删 `rmp-serde = "1.3.1"` 与 `base64 = "0.22"`

### 4. 文档同步

- `docs/product/CLAUDE.md` 第五节 RL Agent 描述、第六节若提及 packed，核对无引用即可
- `docs/product/agent/` 下若有 packed 传输描述一并清理

## 四、契约变更与风险

**返回字段重命名**：`rl_reset` / `rl_step` 的 `observation_b64`（字符串）改为 `observation`（对象）。Rust 侧无消费方；若已有 Python 训练守护进程按 `observation_b64` 解码 msgpack，需同步改为按 `observation` 读 JSON。落地前需确认 Python 侧消费状态。

**依赖收窄**：`lol_rpc` 与 `lol_agent` 各减 `rmp-serde` + `base64` 两个依赖，编译产物体积下降。

## 五、验证

1. `cargo check --all-targets` 全绿。
2. `cargo test -p lol_agent -p lol_rpc` 通过，重点关注 `rl.rs` 剩余 reward / env 测试不受影响。
3. `cargo check` 确认无残留对已删符号的引用。
4. 若 Python 训练端在用，手动跑一次 `rl_reset` / `rl_step` 往返，确认 `observation` JSON 可被正常解析。
