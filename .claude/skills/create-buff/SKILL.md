---
name: create-buff
description: 创建一个新的 Buff 并实现功能
---

# 创建 Buff

创建一个 Buff 组件来实现游戏的功能

## 例子

以创建 Fiora 的 E 技能为例

### 1. 创建 Buff 文件

在 `src/buffs` 目录下创建一个新的 .rs 文件，文件名以英雄名加技能名命名，此例中文件名为 `fiora_e.rs`。
如果是通用的 buff 则以功能名命名，名词在前，动词在后，例如 `attack_speed_up.rs`。

### 2. 编写 Buff 文件

在 `src/buffs/fiora_e.rs` 文件中添加插件和 Buff 组件的定义：

```rs
use bevy::prelude::*;

use crate::{Buff, Buffs, Lifetime};

#[derive(Default)]
pub struct PluginFioraE;

impl Plugin for PluginFioraE {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Clone, Debug)]
#[require(Buff = Buff { name: "FioraE" }, Lifetime = Lifetime::new_timer(3.0))]
pub struct BuffFioraE {
    pub left: i32,
}

impl Default for BuffFioraE {
    fn default() -> Self {
        Self { left: 2 }
    }
}
```

如果 Buff 永远存在，则不需要 require Lifetime 组件

### 3. 注册 Buff 模块和 Buff 插件

在 `src/buffs.rs` 文件中添加 Buff 模块

```rs
// ...其它 mod
mod fiora_e;

// ...其它 use
pub use fiora_e::*;
```

在 `src/lib.rs` 文件中注册 Buff 插件

```rs
plugin_group! {
    pub struct PluginCore {
        // ...其它 Buff 插件
        :PluginFioraE,
    }
}
```

### 4. 实现 Buff 的逻辑

实现 Buff 的逻辑，例如 Fiora 的 E 技能会在每次攻击结束时减少一次，当次数为 0 时移除 Buff。

```rs
fn on_event_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_buffs: Query<&Buffs>,
    mut q_buff_fiora_e: Query<&mut BuffFioraE>,
) {
    let entity = trigger.event_target();
    let Ok(buffs) = q_buffs.get(entity) else {
        return;
    };

    for buff in buffs.iter() {
        let Ok(mut buff_fiora_e) = q_buff_fiora_e.get_mut(buff) else {
            continue;
        };

        buff_fiora_e.left -= 1;

        if buff_fiora_e.left <= 0 {
            commands.entity(buff).despawn();
        }
    }
}
```

在插件添加 observer

```rs
impl Plugin for PluginFioraE {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_attack_end);
    }
}
```

### 5. 添加 Buff

在技能效果中添加 Buff

```rs
                Behave::trigger(ActionBuffSpawn::new((
                    BuffAttack {
                        bonus_attack_speed: 0.5,
                    },
                    BuffFioraE::default()
                ))),
```

由于 Fiora 的 E 技能是伴随着攻速增加的，因此将两个 Buff 组合在一起挂载到同一个 Buff 实体上，BuffFioraE 负责管理次数，BuffAttack 负责管理攻速，从而实现了 Buff 效果的组合。
