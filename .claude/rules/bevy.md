---
paths:
  - "crates/lol_core/src/**/*.rs"
  - "crates/lol_render/src/**/*.rs"
---

# bevy 0.19

- 不要使用 system 中的 EventReader<CustomEvent>，而是使用 observer 中的 event: On<CustomEntityEvent>
- 不要使用 commands.trigger_targets，而是使用 commands.trigger(CustomEntityEvent { entity, ... })
- 不要使用 Camera3dBundle MeshBundle，XxxBundle 类型的 API 已经被废弃了，bevy 已采用 require 宏隐式创建必须的组件
- system 命名规范为 {调度时机}\_{功能}，例如：update_movement，fixed_update_attack_enemy
- observer 命名规范为 on\_{事件名}，例如：on_command_attack_start，on_event_died，第一个参数名规范为 event，例如 (event: On<CustomEvent>, ... )
- event 命名规范为 CommandXXX 或者 EventXXX，Command 为供外部用于调用的命令，可以降低耦合，Event 为供外部监听的事件
- despawn_recursive 已经被废弃了，改用 despawn，despawn 已经是递归销毁了
- get_single 已经废弃了，改为 single() 了
- 不需要 register_type ，derive Reflect 时已经注册好了
