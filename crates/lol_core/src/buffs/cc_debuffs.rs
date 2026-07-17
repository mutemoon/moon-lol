use bevy::prelude::*;

use crate::base::buff::{Buff, BuffOf};
use crate::buffs::common_buffs::BuffCastBlock;
use crate::movement::{CastBlock, MovementBlock, MovementSlow};

/// 控制标签：所有可被净化的 CC debuff 都 `#[require(ControlTag)]`。
/// 净化（`CommandCleanse`）= 批量销毁角色身上带此标签的 buff 实体，
/// 标记会随 buff 死亡（`On<Remove, ControlTag>`）自动清除。
#[derive(Component, Default)]
pub struct ControlTag;

/// 免疫控制标记：挂在角色上，供 CC 施加观察者极速查询。
/// 由免控 buff（如 Olaf R）的 `On<Add/Remove>` 驱动加/删。
#[derive(Component, Default)]
pub struct ImmuneToCC;

/// 眩晕
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Stun" }, ControlTag)]
pub struct DebuffStun {
    pub timer: Timer,
}

impl DebuffStun {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Slow" }, ControlTag)]
pub struct DebuffSlow {
    pub percent: f32, // 0.0-1.0
    pub timer: Timer,
}

impl DebuffSlow {
    pub fn new(percent: f32, duration: f32) -> Self {
        Self {
            percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 沉默
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Silence" }, ControlTag)]
pub struct DebuffSilence {
    pub timer: Timer,
}

impl DebuffSilence {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 恐惧
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Fear" }, ControlTag)]
pub struct DebuffFear {
    pub timer: Timer,
}

impl DebuffFear {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 击飞（不受韧性减免；不加 MovementBlock，保留击退位移，见 action/knockback.rs）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Knockup" }, ControlTag)]
pub struct DebuffKnockup {
    pub timer: Timer,
}

impl DebuffKnockup {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 净化命令：销毁目标身上所有带 `ControlTag` 的 buff 实体。
/// 不认识任何具体控制，只按标签批量杀人——角色标记随 buff 死亡自动清除。
#[derive(EntityEvent, Debug, Clone)]
pub struct CommandCleanse {
    pub entity: Entity,
}

/// 重算角色身上的 CC 标记（MovementBlock / CastBlock / MovementSlow）。
///
/// 遍历所有 `BuffOf` 指向该角色的 buff 实体（排除 `excluding`），按类别取并集。
/// 幂等：每次 CC buff 增删都调用一次，标记始终反映当前活跃 CC 的并集。
/// 这是"Buff 自己管自己"的落点——buff 的增删事件驱动标记重算。
fn sync_cc_markers(
    commands: &mut Commands,
    char: Entity,
    excluding: Option<Entity>,
    q_buffof: &Query<(Entity, &BuffOf)>,
    q_stun: &Query<&DebuffStun>,
    q_silence: &Query<&DebuffSilence>,
    q_fear: &Query<&DebuffFear>,
    q_slow: &Query<&DebuffSlow>,
    q_knockup: &Query<&DebuffKnockup>,
    q_cast_block: &Query<&BuffCastBlock>,
) {
    let mut move_block = false;
    let mut cast_block = false;
    let mut slow_percent = 0.0f32;

    for (buff_entity, buffof) in q_buffof.iter() {
        if buffof.0 != char {
            continue;
        }
        if Some(buff_entity) == excluding {
            continue;
        }
        if q_stun.get(buff_entity).is_ok() {
            move_block = true;
            cast_block = true;
        }
        if q_silence.get(buff_entity).is_ok() {
            cast_block = true;
        }
        if q_fear.get(buff_entity).is_ok() {
            move_block = true;
            cast_block = true;
        }
        // 击飞不加 MovementBlock：保留击退位移通路（见 action/knockback.rs 注释）
        if q_knockup.get(buff_entity).is_ok() {
            cast_block = true;
        }
        if q_cast_block.get(buff_entity).is_ok() {
            move_block = true;
            cast_block = true;
        }
        if let Ok(slow) = q_slow.get(buff_entity) {
            slow_percent = slow_percent.max(slow.percent);
        }
    }

    let mut entity = commands.entity(char);
    if move_block {
        entity.insert(MovementBlock);
    } else {
        entity.remove::<MovementBlock>();
    }
    if cast_block {
        entity.insert(CastBlock);
    } else {
        entity.remove::<CastBlock>();
    }
    if slow_percent > 0.0 {
        entity.insert(MovementSlow {
            percent: slow_percent,
        });
    } else {
        entity.remove::<MovementSlow>();
    }
}

/// `On<Add, ControlTag>`：任一可净化 CC buff 生成时触发。
/// 若角色免控，立即销毁该 buff（CC 不沾身）；否则重算标记。
fn on_add_control(
    trigger: On<Add, ControlTag>,
    mut commands: Commands,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_immune: Query<(), With<ImmuneToCC>>,
    q_stun: Query<&DebuffStun>,
    q_silence: Query<&DebuffSilence>,
    q_fear: Query<&DebuffFear>,
    q_slow: Query<&DebuffSlow>,
    q_knockup: Query<&DebuffKnockup>,
    q_cast_block: Query<&BuffCastBlock>,
) {
    let buff_entity = trigger.entity;
    let Ok((_, buffof)) = q_buffof.get(buff_entity) else {
        return;
    };
    let char = buffof.0;

    // 免控：CC 不沾身，立即销毁（On<Remove> 会重算，不会留下标记）
    if q_immune.get(char).is_ok() {
        commands.entity(buff_entity).despawn();
        return;
    }

    sync_cc_markers(
        &mut commands,
        char,
        None,
        &q_buffof,
        &q_stun,
        &q_silence,
        &q_fear,
        &q_slow,
        &q_knockup,
        &q_cast_block,
    );
}

/// `On<Remove, ControlTag>`：任一可净化 CC buff 消亡（过期/净化/免疫销毁）时触发。
/// 排除自身后重算标记——多 buff 叠加时移除其一不会误清标记。
fn on_remove_control(
    trigger: On<Remove, ControlTag>,
    mut commands: Commands,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_stun: Query<&DebuffStun>,
    q_silence: Query<&DebuffSilence>,
    q_fear: Query<&DebuffFear>,
    q_slow: Query<&DebuffSlow>,
    q_knockup: Query<&DebuffKnockup>,
    q_cast_block: Query<&BuffCastBlock>,
) {
    let buff_entity = trigger.entity;
    let Ok((_, buffof)) = q_buffof.get(buff_entity) else {
        // buff 实体可能在 despawn 过程中已不可查；标记由其它存活 buff 的增删或计时收敛
        return;
    };
    let char = buffof.0;
    sync_cc_markers(
        &mut commands,
        char,
        Some(buff_entity),
        &q_buffof,
        &q_stun,
        &q_silence,
        &q_fear,
        &q_slow,
        &q_knockup,
        &q_cast_block,
    );
}

/// `BuffCastBlock`（自施法锁，非 ControlTag、不可净化）的标记桥接。
/// 自施法锁不被免控拦截（自身技能 windup 不受 Olaf R 影响）。
fn on_add_cast_block(
    trigger: On<Add, BuffCastBlock>,
    mut commands: Commands,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_stun: Query<&DebuffStun>,
    q_silence: Query<&DebuffSilence>,
    q_fear: Query<&DebuffFear>,
    q_slow: Query<&DebuffSlow>,
    q_knockup: Query<&DebuffKnockup>,
    q_cast_block: Query<&BuffCastBlock>,
) {
    let buff_entity = trigger.entity;
    let Ok((_, buffof)) = q_buffof.get(buff_entity) else {
        return;
    };
    let char = buffof.0;
    sync_cc_markers(
        &mut commands,
        char,
        None,
        &q_buffof,
        &q_stun,
        &q_silence,
        &q_fear,
        &q_slow,
        &q_knockup,
        &q_cast_block,
    );
}

fn on_remove_cast_block(
    trigger: On<Remove, BuffCastBlock>,
    mut commands: Commands,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_stun: Query<&DebuffStun>,
    q_silence: Query<&DebuffSilence>,
    q_fear: Query<&DebuffFear>,
    q_slow: Query<&DebuffSlow>,
    q_knockup: Query<&DebuffKnockup>,
    q_cast_block: Query<&BuffCastBlock>,
) {
    let buff_entity = trigger.entity;
    let Ok((_, buffof)) = q_buffof.get(buff_entity) else {
        return;
    };
    let char = buffof.0;
    sync_cc_markers(
        &mut commands,
        char,
        Some(buff_entity),
        &q_buffof,
        &q_stun,
        &q_silence,
        &q_fear,
        &q_slow,
        &q_knockup,
        &q_cast_block,
    );
}

/// 净化即杀人：销毁目标身上所有带 `ControlTag` 的 buff 实体。
fn on_command_cleanse(
    trigger: On<CommandCleanse>,
    mut commands: Commands,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_control: Query<(), With<ControlTag>>,
) {
    let char = trigger.event_target();
    for (buff_entity, buffof) in q_buffof.iter() {
        if buffof.0 != char {
            continue;
        }
        if q_control.get(buff_entity).is_ok() {
            commands.entity(buff_entity).despawn();
        }
    }
    debug!("净化: 销毁 {:?} 身上所有控制 buff", char);
}

/// 全局 CC 计时：tick 所有 CC debuff 与自施法锁的 timer，过期则销毁实体
/// （销毁触发 `On<Remove, ControlTag>` -> 标记自动重算）。
fn update_cc_buff_timers(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_stun: Query<(Entity, &mut DebuffStun)>,
    mut q_slow: Query<(Entity, &mut DebuffSlow)>,
    mut q_silence: Query<(Entity, &mut DebuffSilence)>,
    mut q_fear: Query<(Entity, &mut DebuffFear)>,
    mut q_knockup: Query<(Entity, &mut DebuffKnockup)>,
    mut q_cast_block: Query<(Entity, &mut BuffCastBlock)>,
) {
    let delta = time.delta();

    for (entity, mut buff) in q_stun.iter_mut() {
        buff.timer.tick(delta);
        if buff.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut buff) in q_slow.iter_mut() {
        buff.timer.tick(delta);
        if buff.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut buff) in q_silence.iter_mut() {
        buff.timer.tick(delta);
        if buff.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut buff) in q_fear.iter_mut() {
        buff.timer.tick(delta);
        if buff.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut buff) in q_knockup.iter_mut() {
        buff.timer.tick(delta);
        if buff.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut buff) in q_cast_block.iter_mut() {
        buff.timer.tick(delta);
        if buff.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Default)]
pub struct PluginCc;

impl Plugin for PluginCc {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_control);
        app.add_observer(on_remove_control);
        app.add_observer(on_add_cast_block);
        app.add_observer(on_remove_cast_block);
        app.add_observer(on_command_cleanse);
        app.add_systems(FixedUpdate, update_cc_buff_timers);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::base::buff::{BuffOf, Buffs};
    use crate::movement::{CastBlock, MovementBlock, MovementSlow};
    use crate::team::Team;

    /// 构造仅含 PluginCc 的最小 app：30fps 固定步进，每 `update()` 推进 1/30 秒。
    fn app_with_cc() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginCc);
        app.insert_resource(Time::<Fixed>::from_hz(30.0));
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
        app
    }

    /// 生成一个角色实体（带 Team/Transform），返回其 Entity。
    fn spawn_char(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((Team::Order, Transform::from_xyz(0.0, 0.0, 0.0)))
            .id()
    }

    /// 对角色施加一个 CC buff（独立实体），并推进一帧让观察者桥接标记。
    fn apply_cc<T: Component>(app: &mut App, char: Entity, buff: T) {
        app.world_mut()
            .entity_mut(char)
            .with_related::<BuffOf>(buff);
        app.update();
    }

    #[test]
    fn stun_adds_movement_and_cast_block() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        apply_cc(&mut app, char, DebuffStun::new(1.0));
        assert!(app.world().get::<MovementBlock>(char).is_some());
        assert!(app.world().get::<CastBlock>(char).is_some());
        assert!(app.world().get::<MovementSlow>(char).is_none());
    }

    #[test]
    fn silence_adds_only_cast_block() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        apply_cc(&mut app, char, DebuffSilence::new(1.0));
        assert!(app.world().get::<CastBlock>(char).is_some());
        assert!(app.world().get::<MovementBlock>(char).is_none());
        assert!(app.world().get::<MovementSlow>(char).is_none());
    }

    #[test]
    fn slow_adds_movement_slow_marker() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        apply_cc(&mut app, char, DebuffSlow::new(0.5, 1.0));
        let slow = app
            .world()
            .get::<MovementSlow>(char)
            .expect("应有 MovementSlow");
        assert!((slow.percent - 0.5).abs() < 1e-4);
        assert!(app.world().get::<MovementBlock>(char).is_none());
        assert!(app.world().get::<CastBlock>(char).is_none());
    }

    #[test]
    fn knockup_adds_cast_block_not_movement() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        apply_cc(&mut app, char, DebuffKnockup::new(1.0));
        assert!(app.world().get::<CastBlock>(char).is_some());
        assert!(
            app.world().get::<MovementBlock>(char).is_none(),
            "击飞不加 MovementBlock，保留击退位移通路"
        );
    }

    #[test]
    fn stun_expires_clears_markers() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        apply_cc(&mut app, char, DebuffStun::new(0.1));
        assert!(app.world().get::<MovementBlock>(char).is_some());
        // 0.1s 眩晕：~4 帧（0.13s）后过期
        for _ in 0..10 {
            app.update();
        }
        assert!(
            app.world().get::<MovementBlock>(char).is_none(),
            "过期后 MovementBlock 应清"
        );
        assert!(
            app.world().get::<CastBlock>(char).is_none(),
            "过期后 CastBlock 应清"
        );
    }

    #[test]
    fn two_stuns_remove_one_keeps_markers() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        apply_cc(&mut app, char, DebuffStun::new(1.0));
        apply_cc(&mut app, char, DebuffStun::new(1.0));
        // 收集角色身上的眩晕 buff 实体
        let buffs = app.world().get::<Buffs>(char).expect("应有 Buffs");
        let stun_buffs: Vec<Entity> = buffs
            .iter()
            .filter(|b| app.world().get::<DebuffStun>(*b).is_some())
            .collect();
        assert_eq!(stun_buffs.len(), 2, "应有两个眩晕 buff");
        // 销毁其中一个
        app.world_mut().entity_mut(stun_buffs[0]).despawn();
        app.update();
        // 仍有一个眩晕 -> 标记保留
        assert!(
            app.world().get::<MovementBlock>(char).is_some(),
            "余下一个眩晕应保留 MovementBlock"
        );
        assert!(
            app.world().get::<CastBlock>(char).is_some(),
            "余下一个眩晕应保留 CastBlock"
        );
    }

    #[test]
    fn cleanse_removes_all_control() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        apply_cc(&mut app, char, DebuffStun::new(1.0));
        apply_cc(&mut app, char, DebuffSlow::new(0.5, 1.0));
        assert!(app.world().get::<MovementBlock>(char).is_some());
        assert!(app.world().get::<MovementSlow>(char).is_some());
        // 净化即杀人：销毁所有 ControlTag buff
        app.world_mut()
            .entity_mut(char)
            .trigger(|e| CommandCleanse { entity: e });
        app.update();
        for _ in 0..2 {
            app.update();
        }
        assert!(
            app.world().get::<MovementBlock>(char).is_none(),
            "净化后 MovementBlock 应清"
        );
        assert!(
            app.world().get::<CastBlock>(char).is_none(),
            "净化后 CastBlock 应清"
        );
        assert!(
            app.world().get::<MovementSlow>(char).is_none(),
            "净化后 MovementSlow 应清"
        );
    }

    #[test]
    fn immune_to_cc_blocks_new_cc() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        // 先免疫
        app.world_mut().entity_mut(char).insert(ImmuneToCC);
        apply_cc(&mut app, char, DebuffStun::new(1.0));
        for _ in 0..2 {
            app.update();
        }
        assert!(
            app.world().get::<MovementBlock>(char).is_none(),
            "免控时 CC 不应沾身"
        );
        assert!(app.world().get::<CastBlock>(char).is_none());
    }

    #[test]
    fn cast_block_buff_adds_markers_and_not_cleansed() {
        let mut app = app_with_cc();
        let char = spawn_char(&mut app);
        apply_cc(&mut app, char, BuffCastBlock::new(1.0));
        assert!(app.world().get::<MovementBlock>(char).is_some());
        assert!(app.world().get::<CastBlock>(char).is_some());
        // 净化不应移除自施法锁（非 ControlTag）
        app.world_mut()
            .entity_mut(char)
            .trigger(|e| CommandCleanse { entity: e });
        for _ in 0..2 {
            app.update();
        }
        assert!(
            app.world().get::<CastBlock>(char).is_some(),
            "自施法锁不可被净化"
        );
    }

    /// 减速应真正降低移动速度：同样路径下，被 DebuffSlow 影响的实体移动距离更短。
    /// 这覆盖所有用 `with_related(DebuffSlow)` 的技能（Fiora E / Camille / Darius E / on-hit）。
    #[test]
    fn slow_buff_reduces_movement_distance() {
        use lol_base::grid::ConfigNavigationGrid;
        use lol_base::spell::Spell;

        use crate::action::PluginAction;
        use crate::movement::{
            CommandMovement, Movement, MovementAction, MovementSource, MovementWay, PluginMovement,
        };
        use crate::navigation::grid::ResourceGrid;
        use crate::navigation::navigation::PluginNavigaton;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginNavigaton);
        app.add_plugins(PluginCc);
        app.init_asset::<Spell>();
        app.insert_resource(Time::<Fixed>::from_hz(30.0));
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ConfigNavigationGrid>>()
            .add(ConfigNavigationGrid::default());
        app.insert_resource(ResourceGrid(handle));

        let dest = Vec3::new(500.0, 0.0, 0.0);
        let normal = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Movement { speed: 1000.0 },
            ))
            .id();
        let slowed = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(0.0, 0.0, 0.0),
                Movement { speed: 1000.0 },
            ))
            .id();
        // 施加 50% 减速（独立 buff 实体，观察者写 MovementSlow 到角色）
        app.world_mut()
            .entity_mut(slowed)
            .with_related::<BuffOf>(DebuffSlow::new(0.5, 5.0));
        app.update(); // 让观察者桥接 MovementSlow

        for e in [normal, slowed] {
            app.world_mut()
                .entity_mut(e)
                .trigger(|ent| CommandMovement {
                    entity: ent,
                    priority: 0,
                    action: MovementAction::Start {
                        way: MovementWay::Path(vec![dest]),
                        speed: None,
                        source: MovementSource::Run,
                    },
                });
        }
        for _ in 0..5 {
            app.update();
        }

        let xn = app.world().get::<Transform>(normal).unwrap().translation.x;
        let xs = app.world().get::<Transform>(slowed).unwrap().translation.x;
        assert!(xn > 0.0, "正常实体应已移动，x={xn}");
        assert!(xs < xn, "减速实体应移动更少：normal x={xn}, slowed x={xs}");
    }
}
