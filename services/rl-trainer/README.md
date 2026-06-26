# MoonLoL RL 训练守护进程

用 Python 封装的 RL 训练控制中心，对应产品文档 `docs/product/agent/` 中
**Section 2 服务端 · 训练守护进程与 API**。

- **零三方依赖**：仅用 Python 标准库（`http.server` + 手写最小 WebSocket），可直接 `python3 -m moonlol_trainer` 运行。
- **REST 控制接口**：启动/暂停/恢复/停止训练、checkpoint 存取。
- **WebSocket 遥测**：向客户端 `/rl-training` 页面推送 `status` / `metrics` / `checkpoint` 帧，
  并接收控制帧（与 `apps/client/src/composables/useRlTelemetry.ts` 的协议一致）。
- **环境接入**：默认用内置 `SimulatedEnv` 自洽运行；可注入 `BevyEnv` 连接 Bevy 侧
  `MoonLoLEnv`（`rl_reset` / `rl_step`，见 `crates/lol_agent`），后者需额外安装 `websockets`。

## 运行

```sh
cd services/rl-trainer
python3 -m moonlol_trainer            # REST :8770, WS :8771
python3 -m moonlol_trainer --rest-port 9000 --ws-port 9001 --checkpoint-dir ./ckpts
```

客户端 `/rl-training` 在「训练守护进程 WS 地址」填 `ws://127.0.0.1:8771` 即可对接。

## REST 接口

| 方法 | 路径 | 说明 |
|---|---|---|
| GET  | `/api/status` | 当前训练状态与步数 |
| POST | `/api/train/start` | 启动训练（body 为训练配置 JSON） |
| POST | `/api/train/pause` | 暂停 |
| POST | `/api/train/resume` | 恢复 |
| POST | `/api/train/stop` | 终止 |
| GET  | `/api/checkpoints` | 列出 checkpoint |
| POST | `/api/checkpoints` | 保存当前 checkpoint |
| GET  | `/api/checkpoints/{id}` | 获取 checkpoint 元数据 |
| GET  | `/api/checkpoints/{id}/download` | 下载权重文件 |

## 测试

```sh
cd services/rl-trainer
python3 -m unittest discover -s tests -v
```
