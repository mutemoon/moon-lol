use bevy::prelude::*;

use crate::buffs::cc_debuffs::DebuffKnockup;
use crate::movement::{CastBlock, CommandMovement, MovementAction, MovementSource, MovementWay};

/// 相对 source 的位移方向：`Away` 为击退（背离 source），`Toward` 为拉回（朝向 source）。
/// `Toward` 会把距离钳制到不越过 source，故传一个大于间距的 distance 即可"拉到脚下"。
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaceDirection {
    #[default]
    Away,
    Toward,
}

#[derive(EntityEvent, Debug, Clone)]
pub struct CommandKnockback {
    pub entity: Entity,               // 被击退的目标
    pub source: Entity,               // 击退来源（用于计算方向）
    pub distance: f32,                // 击退距离
    pub speed: f32,                   // 击退速度
    pub duration: Option<f32>,        // 击飞持续时间（None 则根据 distance/speed 计算）
    pub direction: DisplaceDirection, // 相对 source 的方向，默认 Away（击退）
}

pub fn on_command_knockback(
    trigger: On<CommandKnockback>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
) {
    let target = trigger.event_target();
    let source = trigger.source;

    let Ok(target_transform) = q_transform.get(target) else {
        return;
    };
    let Ok(source_transform) = q_transform.get(source) else {
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
    let direction = diff.normalize_or_zero();

    // 如果位置重叠，默认向后方击退（这里简单处理，实际可能需要 source 的 forward）
    let direction = if direction == Vec2::ZERO {
        Vec2::new(0.0, 1.0)
    } else {
        direction
    };

    // Toward 钳制距离：不越过 source，避免拉过头
    let distance = match trigger.direction {
        DisplaceDirection::Away => trigger.distance,
        DisplaceDirection::Toward => trigger.distance.min(diff.length()),
    };

    let dest_xz = target_transform.translation.xz() + direction * distance;
    let destination = Vec3::new(dest_xz.x, target_transform.translation.y, dest_xz.y);

    let duration = trigger.duration.unwrap_or(distance / trigger.speed);

    info!(
        "触发击退: {:?} -> {:?}, 方向: {:?}, 距离: {}, 速度: {}, 持续时间: {}",
        source, target, trigger.direction, distance, trigger.speed, duration
    );

    // 触发位移
    commands.entity(target).trigger(|e| CommandMovement {
        entity: e,
        priority: 100, // 高优先级
        action: MovementAction::Start {
            way: MovementWay::Path(vec![destination]),
            speed: Some(trigger.speed),
            source: MovementSource::Knockback,
        },
    });

    // 注入击飞控制（不添加 MovementBlock，否则位移系统会跳过该实体）
    commands
        .entity(target)
        .insert((DebuffKnockup::new(duration), CastBlock));
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

    /// 构造一个带空导航网格的 App：Path 类位移不需要网格单元格，
    /// 但 `apply_final_movement_decision` 受 `ResourceGrid` 存在性门控，故提供一个空资产。
    fn app_with_grid() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginNavigaton);
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
        let mut app = app_with_grid();
        let (source, target) = spawn_source_and_target(&mut app);

        app.world_mut()
            .entity_mut(target)
            .trigger(|e| CommandKnockback {
                entity: e,
                source,
                distance: 200.0,
                speed: 1000.0,
                duration: None,
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
        let mut app = app_with_grid();
        let (source, target) = spawn_source_and_target(&mut app);

        app.world_mut()
            .entity_mut(target)
            .trigger(|e| CommandKnockback {
                entity: e,
                source,
                distance: 200.0,
                speed: 1000.0,
                duration: None,
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
}
