use bevy::prelude::*;

use crate::base::buff::{Buff, BuffOf};
use crate::life::Health;
use crate::movement::Movement;

/// 施法期间阻塞 buff（通用）
/// 阻止移动和技能施放。
///
/// 标记（`MovementBlock`/`CastBlock`）由 `PluginCc` 的 `On<Add/Remove, BuffCastBlock>`
/// 观察者桥接到角色，本组件只携带倒计时逻辑（Buff 自己管自己）。
/// 非 `ControlTag`：自施法锁不可被净化。
#[derive(Component, Debug)]
#[require(Buff = Buff { name: "CastBlock" })]
pub struct BuffCastBlock {
    pub timer: Timer,
}

impl BuffCastBlock {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 移动速度加成 buff（通用）
///
/// 首次 tick 把 `bonus_percent * 持有者当前移速` 加到 `Movement.speed`，到期精确回退并销毁。
/// `applied`/`applied_bonus` 保证幂等应用与精确回退，多 buff 叠加各自独立记账。
/// 被动击破要害、R 大招期间、Aatrox/Sett/Kayn/Hecarim/Volibear/MasterYi 的移速增益共用此 buff。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MoveSpeed" })]
pub struct BuffMoveSpeed {
    pub bonus_percent: f32,
    pub timer: Timer,
    pub applied: bool,
    pub applied_bonus: f32,
}

impl BuffMoveSpeed {
    pub fn new(bonus_percent: f32, duration: f32) -> Self {
        Self {
            bonus_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
            applied: false,
            applied_bonus: 0.0,
        }
    }
}

/// 双抗加成 buff（通用）
///
/// 注意：当前 `magic_resist` 字段尚无对应 `MagicResist` 原语与伤害结算通路，
/// 故本 buff 暂无生效系统（见体检报告 F3）。仅 `armor` 在未来接入 `Armor` 后可生效。
/// 保留字段以匹配调用方签名（Jax R），待魔抗原语就绪后补齐系统。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Resist" })]
pub struct BuffResist {
    pub armor: f32,
    pub magic_resist: f32,
    pub timer: Timer,
}

impl BuffResist {
    pub fn new(armor: f32, magic_resist: f32, duration: f32) -> Self {
        Self {
            armor,
            magic_resist,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 自我治疗 buff（通用）
///
/// 一次性：下次 tick 把 `amount` 加到持有者生命值（夹取到 `max`）后立即销毁。
/// 无 timer——治疗是瞬发的，buff 只是「待结算的治疗票据」。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SelfHeal" })]
pub struct BuffSelfHeal {
    pub amount: f32,
}

impl BuffSelfHeal {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}

/// 计时并回退 `BuffMoveSpeed`：首次 tick 应用加成，到期回退并销毁。
pub fn update_move_speed_buff(
    mut commands: Commands,
    mut q_buff: Query<(Entity, &BuffOf, &mut BuffMoveSpeed)>,
    mut q_movement: Query<&mut Movement>,
    time: Res<Time<Fixed>>,
) {
    for (buff_entity, buff_of, mut buff) in q_buff.iter_mut() {
        let holder = buff_of.0;
        if !buff.applied {
            if let Ok(mut mov) = q_movement.get_mut(holder) {
                buff.applied_bonus = mov.speed * buff.bonus_percent;
                mov.speed += buff.applied_bonus;
                buff.applied = true;
            }
        }
        buff.timer.tick(time.delta());
        if buff.timer.is_finished() && buff.applied {
            if let Ok(mut mov) = q_movement.get_mut(holder) {
                mov.speed -= buff.applied_bonus;
            }
            commands.entity(buff_entity).despawn();
        }
    }
}

/// 结算 `BuffSelfHeal`：一次性治疗持有者后销毁 buff。
pub fn update_self_heal_buff(
    mut commands: Commands,
    q_buff: Query<(Entity, &BuffOf, &BuffSelfHeal)>,
    mut q_health: Query<&mut Health>,
) {
    for (buff_entity, buff_of, buff) in q_buff.iter() {
        let holder = buff_of.0;
        if let Ok(mut health) = q_health.get_mut(holder) {
            if health.value > 0.0 {
                health.value = (health.value + buff.amount).min(health.max);
            }
        }
        commands.entity(buff_entity).despawn();
    }
}

#[derive(Default)]
pub struct PluginCommonBuffs;

impl Plugin for PluginCommonBuffs {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (update_move_speed_buff, update_self_heal_buff));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::team::Team;

    /// 构造仅含 `PluginCommonBuffs` 的最小 app：30fps 固定步进，每 `update()` 推进 1/30 秒。
    fn app_with_common_buffs() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginCommonBuffs);
        app.insert_resource(Time::<Fixed>::from_hz(30.0));
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
        app
    }

    /// 对角色施加一个 buff（独立实体，不推进时间）。
    fn apply_buff<T: Component>(app: &mut App, char: Entity, buff: T) {
        app.world_mut()
            .entity_mut(char)
            .with_related::<BuffOf>(buff);
    }

    /// 推进若干帧。`FixedTimesteps(1)` 首帧仅初始化时钟（0 个固定步），
    /// 故需 ≥2 帧才能保证至少一个固定步执行。
    fn step(app: &mut App, frames: usize) {
        for _ in 0..frames {
            app.update();
        }
    }

    #[test]
    fn move_speed_buff_increases_speed_on_apply() {
        let mut app = app_with_common_buffs();
        let char = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Movement { speed: 1000.0 },
            ))
            .id();
        apply_buff(&mut app, char, BuffMoveSpeed::new(0.3, 1.0));
        step(&mut app, 3);
        let speed = app.world().get::<Movement>(char).unwrap().speed;
        assert!(
            (speed - 1300.0).abs() < 1e-2,
            "30% 加成应使移速 1000 -> 1300，实际 {speed}"
        );
    }

    #[test]
    fn move_speed_buff_reverts_on_expire() {
        let mut app = app_with_common_buffs();
        let char = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Movement { speed: 1000.0 },
            ))
            .id();
        apply_buff(&mut app, char, BuffMoveSpeed::new(0.3, 0.1));
        // 0.1s 加成：3+ 固定步后过期
        step(&mut app, 10);
        let speed = app.world().get::<Movement>(char).unwrap().speed;
        assert!(
            (speed - 1000.0).abs() < 1e-2,
            "过期后移速应回退到 1000，实际 {speed}"
        );
        let buffs = app.world().get::<crate::base::buff::Buffs>(char);
        let still_has = buffs
            .map(|b| {
                b.iter()
                    .any(|e| app.world().get::<BuffMoveSpeed>(e).is_some())
            })
            .unwrap_or(false);
        assert!(!still_has, "过期后 buff 实体应已销毁");
    }

    #[test]
    fn move_speed_buffs_stack_independently() {
        let mut app = app_with_common_buffs();
        let char = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Movement { speed: 1000.0 },
            ))
            .id();
        apply_buff(&mut app, char, BuffMoveSpeed::new(0.3, 1.0));
        apply_buff(&mut app, char, BuffMoveSpeed::new(0.2, 1.0));
        step(&mut app, 3);
        // 1000 -> 1300 (+30%) -> 1560 (+20% of 1300)
        let speed = app.world().get::<Movement>(char).unwrap().speed;
        assert!(
            (speed - 1560.0).abs() < 1e-1,
            "两 buff 独立叠加应得 1560，实际 {speed}"
        );
    }

    #[test]
    fn self_heal_buff_heals_and_despawns() {
        let mut app = app_with_common_buffs();
        let char = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Health {
                    value: 50.0,
                    max: 100.0,
                    ..default()
                },
            ))
            .id();
        apply_buff(&mut app, char, BuffSelfHeal::new(30.0));
        step(&mut app, 3);
        let health = app.world().get::<Health>(char).unwrap();
        assert!(
            (health.value - 80.0).abs() < 1e-2,
            "应治疗 50 -> 80，实际 {}",
            health.value
        );
        let buffs = app.world().get::<crate::base::buff::Buffs>(char);
        let still_has = buffs
            .map(|b| {
                b.iter()
                    .any(|e| app.world().get::<BuffSelfHeal>(e).is_some())
            })
            .unwrap_or(false);
        assert!(!still_has, "一次性治疗后 buff 应已销毁");
    }

    #[test]
    fn self_heal_buff_clamps_to_max() {
        let mut app = app_with_common_buffs();
        let char = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Health {
                    value: 90.0,
                    max: 100.0,
                    ..default()
                },
            ))
            .id();
        apply_buff(&mut app, char, BuffSelfHeal::new(30.0));
        step(&mut app, 3);
        let health = app.world().get::<Health>(char).unwrap();
        assert!(
            (health.value - 100.0).abs() < 1e-2,
            "治疗不应超过上限，实际 {}",
            health.value
        );
    }

    #[test]
    fn self_heal_buff_skips_dead() {
        let mut app = app_with_common_buffs();
        let char = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Health {
                    value: 0.0,
                    max: 100.0,
                    ..default()
                },
            ))
            .id();
        apply_buff(&mut app, char, BuffSelfHeal::new(30.0));
        step(&mut app, 3);
        let health = app.world().get::<Health>(char).unwrap();
        assert!(
            (health.value - 0.0).abs() < 1e-2,
            "死亡实体不应被治疗，实际 {}",
            health.value
        );
    }
}
