use bevy::prelude::*;
use lol_base::spell::Spell;
use lol_base::ui::LOLEnumData::AtlasData;
use lol_base::ui::{
    LOLAtlasData, LOLPlayerFrameViewController, LOLSpellSlotDetailedUiDefinition,
    LOLUiElementEffectDesaturateData, LOLUiElementIconData,
};
use lol_core::base::level::Level;
use lol_core::character::CharacterReady;
use lol_core::skill::{CommandSkillLevelUp, CoolDown, PassiveSkill, Skill, SkillPoints, Skills};

use crate::controller::Controller;
use crate::ui::button::update_button;
use crate::ui::element::{UIElement, UIElementEntity, UIState};
use crate::ui::text::UiTextState;

#[derive(Default)]
pub struct PluginUISkill;

impl Plugin for PluginUISkill {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillLevelUpButton>();
        app.add_systems(
            Update,
            (
                update_skill_level_up_buttons.after(update_button).run_if(
                    in_state(UIState::Loaded)
                        .and_then(any_match_filter::<(With<Controller>, With<CharacterReady>)>),
                ),
                (update_passive_skill_icon, update_active_skill_icons).run_if(
                    in_state(UIState::Loaded)
                        .and_then(any_match_filter::<(With<Controller>, With<CharacterReady>)>)
                        .and_then(run_once),
                ),
                (update_skill_cooldown, update_skill_rank_pips).run_if(
                    in_state(UIState::Loaded)
                        .and_then(any_match_filter::<(With<Controller>, With<CharacterReady>)>),
                ),
            ),
        );
    }
}

#[derive(Resource, Default)]
struct SkillLevelUpButton {
    pub entities: [Option<Entity>; 4],
}

fn update_passive_skill_icon(
    mut commands: Commands,
    q_skill: Query<&Skill>,
    q_passive_skill: Query<&PassiveSkill, With<Controller>>,
    res_ui_element_entity: Res<UIElementEntity>,
    res_assets_spell_object: Res<Assets<Spell>>,
    mut res_assets_desaturate_data: ResMut<Assets<LOLUiElementEffectDesaturateData>>,
    res_player_frame_vc: Res<LOLPlayerFrameViewController>,
) {
    let Ok(passive_skill) = q_passive_skill.single() else {
        info!("未找到控制器的被动技能");
        return;
    };

    let Ok(skill) = q_skill.get(passive_skill.entity()) else {
        info!("未找到被动技能实体数据");
        return;
    };

    let Some(spell) = res_assets_spell_object.get(&skill.spell) else {
        info!("未找到技能资源数据: {:?}", skill.spell);
        return;
    };

    let Some(icon_path) = spell.spell_data.as_ref().and_then(|d| d.icon_path.as_ref()) else {
        info!("技能 {:?} 没有图标路径", spell.spell_data);
        return;
    };

    let Some(icon_name) = icon_path.get(0) else {
        info!("技能 {:?} 图标路径为空", spell.spell_data);
        return;
    };

    let Some(content_element) = res_player_frame_vc
        .abilities_ui_data
        .passive
        .content_element
    else {
        info!("被动技能 UI 未配置内容元素");
        return;
    };

    info!("更新被动技能图标: {}", icon_name);

    update_icon_data(
        &content_element.into(),
        icon_name,
        &mut res_assets_desaturate_data,
    );

    commands
        .entity(
            res_ui_element_entity
                .get_entity(&res_player_frame_vc.abilities_ui_data.passive.border_enabled),
        )
        .insert(Visibility::Inherited);

    commands
        .entity(res_ui_element_entity.get_entity(&content_element))
        .insert(Visibility::Inherited);
}

fn update_active_skill_icons(
    mut commands: Commands,
    q_skill: Query<&Skill>,
    q_skills: Query<&Skills, With<Controller>>,
    res_ui_element_entity: Res<UIElementEntity>,
    res_assets_spell_object: Res<Assets<Spell>>,
    mut res_assets_desaturate_data: ResMut<Assets<LOLUiElementEffectDesaturateData>>,
    res_player_frame_vc: Res<LOLPlayerFrameViewController>,
) {
    let Ok(skills) = q_skills.single() else {
        info!("未找到控制器的技能列表");
        return;
    };

    for (index, skill_entity) in skills.iter().enumerate() {
        if index >= res_player_frame_vc.abilities_ui_data.champion_spells.len() {
            info!("技能索引 {} 超过了 UI 槽位限制，停止更新", index);
            break;
        }

        let Ok(skill) = q_skill.get(skill_entity) else {
            info!("未找到技能实体 {:?} 的 Skill 组件", skill_entity);
            continue;
        };

        let Some(spell) = res_assets_spell_object.get(&skill.spell) else {
            info!("未找到技能 {:?} 对应的法术资源", skill.spell);
            continue;
        };

        let Some(icon_path) = spell.spell_data.as_ref().and_then(|d| d.icon_path.as_ref()) else {
            info!("法术 {:?} 缺少图标路径", skill.spell);
            continue;
        };

        let Some(icon_name) = icon_path.get(0) else {
            info!("法术 {:?} 的图标路径为空", skill.spell);
            continue;
        };

        let slot_def = &res_player_frame_vc.abilities_ui_data.champion_spells[index];
        let Some(content_element) = slot_def.content_element else {
            info!("UI 槽位 {} 缺少内容元素定义", index);
            continue;
        };

        info!("更新技能 {} 的图标: {:?}", index, content_element);

        update_icon_data(
            &content_element.into(),
            icon_name,
            &mut res_assets_desaturate_data,
        );

        commands
            .entity(res_ui_element_entity.get_entity(&slot_def.border_enabled))
            .insert(Visibility::Inherited);

        commands
            .entity(res_ui_element_entity.get_entity(&slot_def.cooldown))
            .insert(Visibility::Inherited);

        commands
            .entity(res_ui_element_entity.get_entity(&slot_def.overlay_oom.unwrap()))
            .insert(Visibility::Hidden);

        commands
            .entity(res_ui_element_entity.get_entity(&content_element))
            .insert(Visibility::Inherited);
    }
}

fn update_single_skill_cooldown(
    commands: &mut Commands,
    skill: &Skill,
    cooldown: &CoolDown,
    slot_def: &LOLSpellSlotDetailedUiDefinition,
    res_ui_element_entity: &UIElementEntity,
    q_ui_text_state: &mut Query<(&mut UiTextState, &mut Visibility)>,
) {
    let disabled_entity = res_ui_element_entity.get_entity(&slot_def.overlay_disabled.unwrap());
    let ui_entity = res_ui_element_entity.get_entity(&slot_def.cooldown);
    let border_enabled_entity = res_ui_element_entity.get_entity(&slot_def.border_enabled);
    let border_disabled_entity = res_ui_element_entity.get_entity(&slot_def.border_disabled);

    let Ok((mut text_state, mut visibility)) = q_ui_text_state.get_mut(ui_entity) else {
        return;
    };

    if skill.level == 0 {
        if text_state.text != "" {
            text_state.text = "".to_string();
        }
        if matches!(*visibility, Visibility::Inherited) {
            *visibility = Visibility::Hidden;
        }
        commands
            .entity(disabled_entity)
            .insert(Visibility::Inherited);
        commands
            .entity(border_disabled_entity)
            .insert(Visibility::Inherited);
        commands
            .entity(border_enabled_entity)
            .insert(Visibility::Hidden);
        return;
    }

    let remaining = cooldown
        .timer
        .as_ref()
        .map(|v| v.remaining_secs())
        .unwrap_or(0.0);

    if remaining > 0.0 {
        let text_val = remaining.ceil().to_string();
        if text_state.text != text_val {
            text_state.text = text_val;
        }
        if matches!(*visibility, Visibility::Hidden) {
            *visibility = Visibility::Inherited;
        }
        commands
            .entity(disabled_entity)
            .insert(Visibility::Inherited);
        commands
            .entity(border_disabled_entity)
            .insert(Visibility::Inherited);
        commands
            .entity(border_enabled_entity)
            .insert(Visibility::Hidden);
    } else {
        if text_state.text != "" {
            text_state.text = "".to_string();
        }
        if matches!(*visibility, Visibility::Inherited) {
            *visibility = Visibility::Hidden;
        }
        commands.entity(disabled_entity).insert(Visibility::Hidden);
        commands
            .entity(border_enabled_entity)
            .insert(Visibility::Inherited);
        commands
            .entity(border_disabled_entity)
            .insert(Visibility::Hidden);
    }
}

fn update_single_skill_rank_pips(
    commands: &mut Commands,
    index: usize,
    skill: &Skill,
    res_player_frame_vc: &LOLPlayerFrameViewController,
    res_ui_element_entity: &UIElementEntity,
    q_ui_element: &Query<&UIElement>,
    icon_assets: &mut Assets<LOLUiElementIconData>,
) {
    let pips_list = &res_player_frame_vc
        .abilities_ui_data
        .spell_rank_pips
        .rank_pips;
    let Some(pip_def) = pips_list.get(index) else {
        debug!(
            "【技能等级点】未能在 abilities_ui_data.spell_rank_pips.rank_pips 中找到索引 {} 的定义",
            index
        );
        return;
    };

    let max_level = match index {
        3 => 3,
        _ => 5,
    };

    debug!(
        "【技能等级点】开始处理技能索引: {}, 当前技能等级: {}, 最大规格: {}, empty_pips数量: {}, full_pips数量: {}",
        index,
        skill.level,
        max_level,
        pip_def.empty_pips.len(),
        pip_def.full_pips.len()
    );

    for i in 0..pip_def.empty_pips.len() {
        let empty_key = pip_def.empty_pips[i].0;
        let full_key = pip_def.full_pips[i].0;
        let empty_entity = res_ui_element_entity.map.get(&empty_key).copied();
        let full_entity = res_ui_element_entity.map.get(&full_key).copied();

        if empty_entity.is_none() || full_entity.is_none() {
            debug!(
                "【技能等级点警告】索引 {}, 槽位 {} 实体未找到: empty_key: {} (存在: {}), full_key: {} (存在: {})",
                index,
                i,
                empty_key,
                empty_entity.is_some(),
                full_key,
                full_entity.is_some()
            );
            continue;
        }

        let empty_entity = empty_entity.unwrap();
        let full_entity = full_entity.unwrap();

        // 动态计算每个技能等级点在各自技能槽位下的正确水平分布坐标
        let base_x = 545.0 + index as f32 * 74.0;
        let base_y = 1124.0;
        let pip_x = if max_level == 5 {
            base_x + 1.5 + i as f32 * 13.0
        } else {
            base_x + 9.5 + i as f32 * 18.0
        };
        let pip_y = base_y;

        let new_pos =
            lol_base::ui::LOLEnumUiPosition::UiPositionRect(lol_base::ui::LOLUiPositionRect {
                anchors: Some(lol_base::ui::LOLEnumAnchor::AnchorSingle(
                    lol_base::ui::LOLAnchorSingle {
                        anchor: Vec2::new(0.5, 1.0),
                    },
                )),
                ui_rect: Some(lol_base::ui::LOLUiElementRect {
                    position: Some(Vec2::new(pip_x, pip_y)),
                    size: Some(Vec2::new(9.0, 9.0)),
                    source_resolution_height: Some(1200),
                    source_resolution_width: Some(1600),
                }),
                disable_pixel_snapping_x: Some(true),
                disable_pixel_snapping_y: Some(true),
                disable_resolution_downscale: None,
                ignore_global_scale: None,
                ignore_safe_zone: None,
            });

        // 仅在坐标真正发生变化时才修改 icon_assets 以触发重新布局计算，并且极为安全高效
        if let Ok(UIElement::Icon(empty_handle)) = q_ui_element.get(empty_entity) {
            if let Some(mut empty_data) = icon_assets.get_mut(*empty_handle) {
                let current_pos = match &empty_data.position {
                    lol_base::ui::LOLEnumUiPosition::UiPositionRect(rect) => {
                        rect.ui_rect.as_ref().and_then(|r| r.position)
                    }
                    _ => None,
                };
                if current_pos != Some(Vec2::new(pip_x, pip_y)) {
                    empty_data.position = new_pos.clone();
                }
            }
        }

        if let Ok(UIElement::Icon(full_handle)) = q_ui_element.get(full_entity) {
            if let Some(mut full_data) = icon_assets.get_mut(*full_handle) {
                let current_pos = match &full_data.position {
                    lol_base::ui::LOLEnumUiPosition::UiPositionRect(rect) => {
                        rect.ui_rect.as_ref().and_then(|r| r.position)
                    }
                    _ => None,
                };
                if current_pos != Some(Vec2::new(pip_x, pip_y)) {
                    full_data.position = new_pos.clone();
                }
            }
        }

        if i >= max_level {
            debug!(
                "【技能等级点】索引 {}, 点位 {} 属于未启用等级 (>= {}), 隐藏 empty 和 full",
                index, i, max_level
            );
            commands.entity(empty_entity).insert(Visibility::Hidden);
            commands.entity(full_entity).insert(Visibility::Hidden);
        } else if i < skill.level {
            debug!(
                "【技能等级点】索引 {}, 点位 {} 属于已升级点 (< {}), 显示 full，隐藏 empty",
                index, i, skill.level
            );
            commands.entity(full_entity).insert(Visibility::Inherited);
            commands.entity(empty_entity).insert(Visibility::Hidden);
        } else {
            debug!(
                "【技能等级点】索引 {}, 点位 {} 属于可学习空点, 显示 empty，隐藏 full",
                index, i
            );
            commands.entity(empty_entity).insert(Visibility::Inherited);
            commands.entity(full_entity).insert(Visibility::Hidden);
        }
    }
}

fn update_skill_cooldown(
    mut commands: Commands,
    q_skill: Query<(&Skill, &CoolDown), Changed<CoolDown>>,
    q_skills: Query<&Skills, With<Controller>>,
    res_player_frame_vc: Res<LOLPlayerFrameViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    mut q_ui_text_state: Query<(&mut UiTextState, &mut Visibility)>,
) {
    let Ok(skills) = q_skills.single() else {
        debug!("【技能冷却更新】未找到控制器的技能列表");
        return;
    };

    for (index, skill_entity) in skills.iter().enumerate() {
        if index >= res_player_frame_vc.abilities_ui_data.champion_spells.len() {
            debug!(
                "【技能冷却更新警告】索引 {} 超出 champion_spells 最大长度",
                index
            );
            break;
        }

        let Ok((skill, cooldown)) = q_skill.get(skill_entity) else {
            continue;
        };

        debug!(
            "【技能冷却更新】技能槽位索引: {}, 技能实体: {:?}, 技能等级: {}",
            index, skill_entity, skill.level
        );

        let slot_def = &res_player_frame_vc.abilities_ui_data.champion_spells[index];
        update_single_skill_cooldown(
            &mut commands,
            skill,
            cooldown,
            slot_def,
            &res_ui_element_entity,
            &mut q_ui_text_state,
        );
    }
}

fn update_skill_rank_pips(
    mut commands: Commands,
    q_changed_skill: Query<(Entity, &Skill), Changed<Skill>>,
    q_skills: Query<&Skills, With<Controller>>,
    res_player_frame_vc: Res<LOLPlayerFrameViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    q_ui_element: Query<&UIElement>,
    mut icon_assets: ResMut<Assets<LOLUiElementIconData>>,
) {
    let Ok(skills) = q_skills.single() else {
        return;
    };

    for (skill_entity, skill) in q_changed_skill.iter() {
        let Some(index) = skills.as_slice().iter().position(|&e| e == skill_entity) else {
            continue;
        };
        if index >= res_player_frame_vc.abilities_ui_data.champion_spells.len() {
            continue;
        }
        update_single_skill_rank_pips(
            &mut commands,
            index,
            skill,
            &res_player_frame_vc,
            &res_ui_element_entity,
            &q_ui_element,
            &mut icon_assets,
        );
    }
}

fn update_icon_data(
    handle: &Handle<LOLUiElementEffectDesaturateData>,
    icon_name: &str,
    assets: &mut Assets<LOLUiElementEffectDesaturateData>,
) {
    let Some(mut icon_data) = assets.get_mut(handle) else {
        debug!("未找到图标数据 {:?}", handle);
        return;
    };
    icon_data.texture_data = Some(AtlasData(LOLAtlasData {
        m_texture_name: icon_name.to_string(),
        m_texture_uv: None,
    }));
}

fn update_skill_level_up_buttons(
    mut commands: Commands,
    q_skill_points: Query<
        (Entity, &Level, &SkillPoints, &Skills),
        (Changed<SkillPoints>, With<Controller>),
    >,
    res_player_frame: Res<LOLPlayerFrameViewController>,
    q_skill: Query<&Skill>,
    res_ui_element_entity: Res<UIElementEntity>,
) {
    let Ok((entity, level, skill_points, skills)) = q_skill_points.single() else {
        return;
    };

    let Some(level_up_spells) = &res_player_frame.level_up_display.spells else {
        return;
    };

    for (index, skill_entity) in skills.iter().enumerate() {
        let Some(level_up_data) = level_up_spells.get(index) else {
            continue;
        };

        // 如果没有技能点，或者技能已经满级，或者不满足加点条件，则隐藏/销毁按钮
        let mut should_show = skill_points.0 > 0;

        if should_show {
            if let Ok(skill) = q_skill.get(skill_entity) {
                // 1 级只能加点 q w e，6 级才能加点 r，6 级前一个技能最多加 3 点
                if level.value < 6 {
                    if index == 3 {
                        should_show = false;
                    } else if skill.level >= 3 {
                        should_show = false;
                    }
                }
            }
        }

        if should_show {
            debug!("显示技能升级按钮");
            commands
                .entity(res_ui_element_entity.get_entity(&level_up_data.skill_up_button))
                .insert(Visibility::Visible)
                .observe(move |_: On<Pointer<Click>>, mut commands: Commands| {
                    commands.trigger(CommandSkillLevelUp { entity, index });
                });
        } else {
            commands
                .entity(res_ui_element_entity.get_entity(&level_up_data.skill_up_button))
                .insert(Visibility::Hidden);
        }
    }
    // debug!("技能升级按钮更新完成");
}
