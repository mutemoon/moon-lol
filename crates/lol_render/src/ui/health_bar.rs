use bevy::prelude::*;
use lol_base::character::HealthBar;
use lol_base::hash::hash_bin;
use lol_base::hash_key::LoadHashKeyTrait;
use lol_base::ui::{
    LOLEnumHealthBarTickStyle, LOLEnumUiPosition, LOLFloatingInfoBarViewController,
    LOLHeroFloatingInfoBarData, LOLStructureFloatingInfoBarData, LOLUiElementEffectInstancedData,
    LOLUiElementIconData, LOLUiElementRegionData, LOLUnitFloatingInfoBarData,
};
use lol_base::ui_components::{HealthBind, UIBindData, UIBindOf, UIBindTarget, UIElement};
use lol_core::base::bounding::Bounding;
use lol_core::base::level::Level;
use lol_core::life::Health;

use crate::ui::element::UIState;
use crate::ui::text::UiTextState;

pub struct PluginUIHealthBar;

impl Plugin for PluginUIHealthBar {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_spawn_health_bar.run_if(in_state(UIState::Loaded)),
        );
        app.add_systems(
            Update,
            (update_health, update_health_ticks, update_level_text),
        );
    }
}

#[derive(Component, Debug, Clone)]
pub struct LevelTextBind(pub Entity);

#[derive(Component, Debug, Clone)]
pub struct HealthTickBind {
    pub target: Entity,
    pub style: LOLEnumHealthBarTickStyle,
    pub last_max_hp: f32,
    pub hp_width: f32,
}

#[derive(Component)]
pub struct HealthTick;

fn update_spawn_health_bar(
    mut commands: Commands,
    q_added_health_bar: Query<(Entity, &HealthBar, &Bounding, &Name), Without<UIBindTarget>>,
    res_floating_info_bar_view_controller: Option<Res<LOLFloatingInfoBarViewController>>,
    res_assets_unit_floating_info_bar_data: Res<Assets<LOLUnitFloatingInfoBarData>>,
    res_assets_hero_floating_info_bar_data: Res<Assets<LOLHeroFloatingInfoBarData>>,
    res_assets_structure_floating_info_bar_data: Res<Assets<LOLStructureFloatingInfoBarData>>,
    res_assets_ui_element_region_data: Res<Assets<LOLUiElementRegionData>>,
    res_assets_ui_element_icon_data: Res<Assets<LOLUiElementIconData>>,
) {
    let Some(controller) = res_floating_info_bar_view_controller.as_deref() else {
        return;
    };

    for (entity, health_bar, bounding, _name) in q_added_health_bar.iter() {
        let health_bar_entity = commands
            .spawn((Node {
                position_type: PositionType::Absolute,
                ..default()
            },))
            .id();

        let Some(&hash) = controller
            .info_bar_style_source_map
            .get(&health_bar.bar_type)
        else {
            warn!("未找到健康条样式 hash: {}", health_bar.bar_type);
            continue;
        };

        let bar_unit = res_assets_unit_floating_info_bar_data.load_hash(hash);
        let bar_hero = res_assets_hero_floating_info_bar_data.load_hash(hash);
        let bar_structure = res_assets_structure_floating_info_bar_data.load_hash(hash);

        let (hp_bar, border, anchor) = match (bar_unit, bar_hero, bar_structure) {
            (Some(bar), None, None) => (&bar.health_bar, bar.border, bar.anchor),
            (None, Some(bar), None) => {
                let level_bg = bar.borders.default_border.level_box_overlay_self.unwrap();
                commands
                    .entity(health_bar_entity)
                    .with_child(UIElement::Icon(level_bg));

                commands.entity(health_bar_entity).with_child((
                    UIElement::Text(bar.borders.level_text),
                    LevelTextBind(entity),
                    UiTextState {
                        text: "0".to_string(),
                    },
                ));

                (
                    &bar.health_bar,
                    bar.borders.default_border.border,
                    bar.anchor,
                )
            }
            (None, None, Some(bar)) => (&bar.health_bar, bar.border, bar.anchor),
            _ => {
                info!("未找到健康条样式 hash: {}", hash);
                return;
            }
        };

        let Some(anchor_region) = res_assets_ui_element_region_data.load_hash(&anchor) else {
            info!("未找到健康条锚点 hash: {:?}", anchor);
            return;
        };

        if let Some(LOLEnumUiPosition::UiPositionRect(anchor_position)) = &anchor_region.position {
            commands.entity(health_bar_entity).insert((
                UIBindOf(entity),
                UIBindData {
                    position: Vec3::ZERO.with_y(bounding.height),
                    anchor: anchor_position
                        .ui_rect
                        .as_ref()
                        .and_then(|v| v.position)
                        .unwrap()
                        + Vec2::new(0.0, 30.0),
                },
            ));
        }

        commands
            .entity(health_bar_entity)
            .with_child(UIElement::Icon(border));

        let hp_bar_icon = hp_bar
            .health_bar
            .additional_bar_types
            .as_ref()
            .and_then(|m| {
                m.get(&hash_bin("green"))
                    .or_else(|| m.get(&hash_bin("enemy")))
            })
            .copied()
            .unwrap_or(hp_bar.health_bar.default_bar);

        commands
            .entity(health_bar_entity)
            .with_child((UIElement::Icon(hp_bar_icon), HealthBind(entity)));

        let hp_bar_icon_data = res_assets_ui_element_icon_data
            .load_hash(hp_bar_icon)
            .unwrap();
        let LOLEnumUiPosition::UiPositionRect(rect) = &hp_bar_icon_data.position else {
            panic!("未找到健康条图标位置: {:?}", hp_bar_icon_data.position);
        };

        if let Some(tick_style) = &hp_bar.tick_style {
            commands.entity(health_bar_entity).insert(HealthTickBind {
                target: entity,
                style: tick_style.clone(),
                last_max_hp: 0.0,
                hp_width: rect.ui_rect.as_ref().unwrap().size.unwrap().x,
            });
        }
    }
}

fn update_health_ticks(
    mut commands: Commands,
    mut q_tick_bind: Query<(Entity, &mut HealthTickBind, Option<&Children>)>,
    q_health: Query<&Health>,
    q_ticks: Query<Entity, With<HealthTick>>,
    res_assets_ui_element_effect_instanced_data: Res<Assets<LOLUiElementEffectInstancedData>>,
) {
    for (container, mut bind, children) in q_tick_bind.iter_mut() {
        let Ok(health) = q_health.get(bind.target) else {
            continue;
        };

        if (health.max - bind.last_max_hp).abs() < 0.1 {
            continue;
        }
        bind.last_max_hp = health.max;

        // 清理旧刻度
        if let Some(children) = children {
            for &child in children {
                if q_ticks.get(child).is_ok() {
                    commands.entity(child).despawn();
                }
            }
        }

        let (micro_tick, standard_tick, micro_per_standard) = match &bind.style {
            LOLEnumHealthBarTickStyle::HealthBarTickStyleHero(hero) => (
                Some(hero.micro_tick),
                hero.standard_tick,
                hero.micro_tick_per_standard_tick_data
                    .iter()
                    .rev()
                    .find(|v| health.max > v.min_health)
                    .map(|d| d.micro_ticks_between)
                    .unwrap_or(10),
            ),
            LOLEnumHealthBarTickStyle::HealthBarTickStyleTftCompanion(tft) => {
                (None, tft.standard_tick, 10)
            }
            LOLEnumHealthBarTickStyle::HealthBarTickStyleUnit(unit) => {
                (None, unit.standard_tick, 10)
            }
        };

        let standard_interval = 1000.0;

        let standard_icon = res_assets_ui_element_effect_instanced_data
            .load_hash(standard_tick)
            .unwrap();
        let micro_icon =
            micro_tick.and_then(|v| res_assets_ui_element_effect_instanced_data.load_hash(v));

        let max_ticks = (health.max * micro_per_standard as f32 / standard_interval) as i32 + 1;

        for i in 1..max_ticks {
            let hp_value = (i as f32) * standard_interval / (micro_per_standard as f32);

            let is_standard = i % micro_per_standard as i32 == 0;

            let icon = if is_standard {
                standard_icon
            } else if let Some(micro) = micro_icon {
                micro
            } else {
                continue;
            };

            let ui_rect = match &icon.position {
                LOLEnumUiPosition::UiPositionRect(rect) => rect.ui_rect.as_ref().unwrap(),
                _ => continue,
            };

            commands.entity(container).with_child((
                HealthTick,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(
                        ui_rect.position.unwrap().x + (hp_value / health.max * bind.hp_width),
                    ),
                    top: Val::Px(ui_rect.position.unwrap().y),
                    width: Val::Px(ui_rect.size.unwrap().x),
                    height: Val::Px(ui_rect.size.unwrap().y),
                    ..default()
                },
                ZIndex(icon.layer as i32),
                BackgroundColor(Color::BLACK),
            ));
        }
    }
}

fn update_health(
    mut q_health_bind: Query<(&HealthBind, &Children)>,
    mut q_health_bar: Query<&mut Node>,
    q_health: Query<&Health>,
) {
    for (health_bind, children) in q_health_bind.iter_mut() {
        let Ok(health) = q_health.get(health_bind.0) else {
            continue;
        };

        let Ok(mut node) = q_health_bar.get_mut(children[0]) else {
            continue;
        };

        node.width = Val::Percent(health.value / health.max * 100.0);
    }
}

fn update_level_text(
    mut q_level_bind: Query<(&LevelTextBind, &mut UiTextState)>,
    q_level: Query<&Level>,
) {
    for (bind, mut text_state) in q_level_bind.iter_mut() {
        let Ok(level) = q_level.get(bind.0) else {
            continue;
        };

        let new_text = level.value.to_string();
        if text_state.text != new_text {
            info!("更新等级文本: {} -> {}", text_state.text, new_text);
            text_state.text = new_text;
        }
    }
}
