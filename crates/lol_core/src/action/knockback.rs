//! 击退 / 击飞统一实现。
//!
//! 强制位移必须走直线（[`MovementWay::Path`]），不能走 [`MovementWay::Pathfind`]：
//! 被震退的目标不该绕障寻路——一旦导航网格把落点判为不可达，位移会在一帧内
//! 把目标直接搬到终点，这就是「瞬移式震退」。
//!
//! 当前引擎在每帧末尾把角色 `y` 贴回地形高度（`navigation::update_y`），
//! 因此击飞只做横向下压的前推 + 滞空控制（[`DebuffKnockup`]），不带垂直弹道。

use bevy::prelude::*;

use crate::base::buff::BuffOf;
use crate::buffs::cc_debuffs::DebuffKnockup;
use crate::movement::{CommandMovement, MovementAction, MovementSource, MovementWay};

/// 重力加速度（单位/秒²）。击飞的滞空时长由 [`Knockup::height`] 与重力决定。
pub const KNOCKUP_GRAVITY: f32 = 2500.0;

/// 相对 source 的位移方向：`Away` 为击退（背离 source），`Toward` 为拉回（朝向 source）。
/// `Toward` 会把距离钳制到不越过 source，故传一个大于间距的 distance 即可"拉到脚下"。
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaceDirection {
    #[default]
    Away,
    Toward,
}

/// 击飞：滞空 `air_time` 秒。
#[derive(Debug, Clone, Copy)]
pub struct Knockup {
    pub height: f32,
}

impl Knockup {
    pub fn new(height: f32) -> Self {
        Self { height }
    }

    /// 滞空时长（上升 + 下落）。
    pub fn air_time(&self) -> f32 {
        2.0 * (2.0 * self.height / KNOCKUP_GRAVITY).sqrt()
    }
}

#[derive(EntityEvent, Debug, Clone)]
pub struct CommandKnockback {
    pub entity: Entity,               // 被击退的目标
    pub source: Entity,               // 击退来源（用于计算方向）
    pub distance: f32,                // 击退距离
    pub speed: f32,                   // 击退速度
    pub knockup: Option<Knockup>,     // Some 时同时击飞（滞空期间无法施法）
    pub direction: DisplaceDirection, // 相对 source 的方向，默认 Away（击退）
}

pub fn on_command_knockback(
    trigger: On<CommandKnockback>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
) {
    let target = trigger.event_target();
    let source = trigger.source;

    let (Ok(target_transform), Ok(source_transform)) =
        (q_transform.get(target), q_transform.get(source))
    else {
        return;
    };

    let diff = match trigger.direction {
        DisplaceDirection::Away => {
            target_transform.translation.xz() - source_transform.translation.xz()
        }
        DisplaceDirection::Toward => {
            source_transform.translation.xz() - target_transform.translation.xz()
        }
    };

    // 位置重叠时沿 source 朝向击退
    let direction = diff
        .try_normalize()
        .unwrap_or_else(|| source_transform.forward().xz());

    // Toward 钳制距离：不越过 source，避免拉过头
    let distance = match trigger.direction {
        DisplaceDirection::Away => trigger.distance,
        DisplaceDirection::Toward => trigger.distance.min(diff.length()),
    };

    let destination = target_transform.translation + (direction * distance).extend(0.0);
    debug!(
        "击退 {:?} <- {:?}: 方向 {:?}, 距离 {}, 击飞 {:?}",
        target, source, trigger.direction, distance, trigger.knockup
    );

    // 击退位移本身需经过 update_path_movement，故不加 MovementBlock（会跳过该实体）。
    // CastBlock 由 PluginCc 的 On<Add, ControlTag> 观察者自动桥接到角色。
    commands.entity(target).trigger(|e| CommandMovement {
        entity: e,
        priority: 100,
        action: MovementAction::Start {
            way: MovementWay::Path(vec![destination]),
            speed: Some(trigger.speed),
            source: MovementSource::Knockback,
        },
    });

    if let Some(knockup) = trigger.knockup {
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffKnockup::new(knockup.air_time()));
    }
}

#[cfg(test)]
mod tests {
    use bevy::time::TimeUpdateStrategy;
    use lol_base::grid::ConfigNavigationGrid;
    use lol_base::spell::Spell;

    use super::*;
    use crate::action::PluginAction;
    use crate::movement::{Movement, PluginMovement};
    use crate::navigation::grid::ResourceGrid;
    use crate::navigation::navigation::PluginNavigaton;
    use crate::team::Team;

    /// 构造一个 App：导航网格刻意留空（`ConfigNavigationGrid::default()` 全不可走），
    /// 强制位移必须无视寻路——只要它还在走 Pathfind，就会原地不动或瞬移。
    fn app_without_grid_cells() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginNavigaton);
        app.add_plugins(crate::buffs::cc_debuffs::PluginCc);
        app.init_asset::<Spell>();
        app.insert_resource(Time::<Fixed>::from_hz(30.0));
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ConfigNavigationGrid>>()
            .add(ConfigNavigationGrid::default());
        app.insert_resource(ResourceGrid(handle));
        app
    }

    fn spawn_source_and_target(app: &mut App) -> (Entity, Entity) {
        let source = app
            .world_mut()
            .spawn((Team::Order, Transform::from_xyz(0.0, 0.0, 0.0)))
            .id();
        let target = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(200.0, 0.0, 0.0),
                Movement { speed: 340.0 },
            ))
            .id();
        (source, target)
    }

    fn target_x(app: &App, target: Entity) -> f32 {
        app.world()
            .get::<Transform>(target)
            .expect("target should have Transform")
            .translation
            .x
    }

    #[test]
    fn knockback_toward_pulls_target_to_source() {
        let mut app = app_without_grid_cells();
        let (source, target) = spawn_source_and_target(&mut app);

        app.world_mut()
            .entity_mut(target)
            .trigger(|e| CommandKnockback {
                entity: e,
                source,
                distance: 200.0,
                speed: 1000.0,
                knockup: None,
                direction: DisplaceDirection::Toward,
            });
        for _ in 0..15 {
            app.update();
        }

        let x = target_x(&app, target);
        assert!(
            x.abs() < 5.0,
            "Toward 应把目标拉向 source（终点≈0），实际 x = {x}"
        );
    }

    #[test]
    fn knockback_away_pushes_target_from_source() {
        let mut app = app_without_grid_cells();
        let (source, target) = spawn_source_and_target(&mut app);

        app.world_mut()
            .entity_mut(target)
            .trigger(|e| CommandKnockback {
                entity: e,
                source,
                distance: 200.0,
                speed: 1000.0,
                knockup: None,
                direction: DisplaceDirection::Away,
            });
        for _ in 0..15 {
            app.update();
        }

        let x = target_x(&app, target);
        assert!(
            x > 395.0,
            "Away 应把目标推离 source（终点≈400），实际 x = {x}"
        );
    }

    /// 击退必须是平滑推进，不能在一帧内把目标搬到终点。
    #[test]
    fn knockback_moves_smoothly_not_teleport() {
        let mut app = app_without_grid_cells();
        let (source, target) = spawn_source_and_target(&mut app);

        app.world_mut()
            .entity_mut(target)
            .trigger(|e| CommandKnockback {
                entity: e,
                source,
                distance: 200.0,
                speed: 1000.0,
                knockup: None,
                direction: DisplaceDirection::Away,
            });

        let mut xs = vec![target_x(&app, target)];
        for _ in 0..15 {
            app.update();
            xs.push(target_x(&app, target));
        }

        assert!(
            xs.windows(2).all(|w| w[1] - w[0] <= 40.0),
            "击退不应瞬移，逐帧位移应平滑，实际轨迹 = {xs:?}"
        );
        assert!(
            xs.iter().any(|x| (210.0..390.0).contains(x)),
            "击退应在中间帧处于途中位置，实际轨迹 = {xs:?}"
        );
    }

    /// 击飞附带的滞空控制：目标获得 Knockup buff（期间不可施法）。
    #[test]
    fn knockup_applies_airborne_control() {
        let mut app = app_without_grid_cells();
        let (source, target) = spawn_source_and_target(&mut app);

        app.world_mut()
            .entity_mut(target)
            .trigger(|e| CommandKnockback {
                entity: e,
                source,
                distance: 100.0,
                speed: 1000.0,
                knockup: Some(Knockup::new(150.0)),
                direction: DisplaceDirection::Away,
            });
        // 观察者桥接 CastBlock 需要额外一帧
        app.update();
        app.update();

        let buffs = app
            .world()
            .get::<crate::base::buff::Buffs>(target)
            .expect("target 应有 Buffs");
        assert!(
            buffs
                .iter()
                .any(|b| app.world().get::<DebuffKnockup>(b).is_some()),
            "击飞应挂上 DebuffKnockup"
        );
        assert!(
            app.world()
                .get::<crate::movement::CastBlock>(target)
                .is_some(),
            "滞空期间应无法施法"
        );
    }
}
