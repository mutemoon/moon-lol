---
trigger: always_on
---

# bevy 0.17

1. 不要使用 system 中的 EventReader<CustomEvent>，而是使用 observer 中的 event: On<CustomEntityEvent>
2. 不要使用 commands.trigger_targets，而是使用 commands.trigger(CustomEntityEvent { entity, ... })
3. 不要使用 Camera3dBundle MeshBundle，XxxBundle 类型的 API 已经被废弃了，bevy 已采用 require 宏隐式创建必须的组件
4. system 命名规范为 {调度时机}\_{功能}，例如：update_movement，fixed_update_attack_enemy
5. observer 命名规范为 on\_{事件名}，例如：on_command_attack_start，on_event_died
6. event 命名规范为 CommandXXX 或者 EventXXX，Command 为供外部用于调用的命令，可以降低耦合，Event 为供外部监听的事件
