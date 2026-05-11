use bevy::prelude::*;
// use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_base::ui::LOLEnumData::AtlasData;
use lol_base::ui::{LOLAtlasData, LOLPlayerFrameViewController, LOLUiElementEffectDesaturateData};
use lol_core::base::level::Level;
use lol_core::character::CharacterReady;
use lol_core::skill::{CommandSkillLevelUp, CoolDown, PassiveSkill, Skill, SkillPoints, Skills};

use crate::controller::Controller;
use crate::ui::button::update_button;
use crate::ui::element::{UIElementEntity, UIState};
use crate::ui::text::UiTextState;

#[derive(Default)]
pub struct PluginUISkill;

impl Plugin for PluginUISkill {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillLevelUpButton>();
        app.add_systems(
            Update,
            (
                update_skill_level_up_button.after(update_button).run_if(
                    in_state(UIState::Loaded)
                        .and_then(any_match_filter::<(With<Controller>, With<CharacterReady>)>),
                ),
                (
                    update_player_passive_skill_icon,
                    update_player_active_skill_icon,
                )
                    .run_if(
                        in_state(UIState::Loaded)
                            .and_then(any_match_filter::<(With<Controller>, With<CharacterReady>)>)
                            .and_then(run_once),
                    ),
                update_skill_cooldown_text.run_if(
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

fn update_player_passive_skill_icon(
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

fn update_player_active_skill_icon(
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

fn update_skill_cooldown_text(
    mut commands: Commands,
    q_skill: Query<&CoolDown>,
    q_skills: Query<&Skills, With<Controller>>,
    res_player_frame_vc: Res<LOLPlayerFrameViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    mut q_ui_text_state: Query<(&mut UiTextState, &mut Visibility)>,
) {
    let Ok(skills) = q_skills.single() else {
        info!("未找到控制器的技能列表");
        return;
    };

    for (index, skill_entity) in skills.iter().enumerate() {
        if index >= res_player_frame_vc.abilities_ui_data.champion_spells.len() {
            break;
        }

        let Ok(cooldown) = q_skill.get(skill_entity) else {
            continue;
        };

        let slot_def = &res_player_frame_vc.abilities_ui_data.champion_spells[index];
        let disabled_entity = res_ui_element_entity.get_entity(&slot_def.overlay_disabled.unwrap());

        let ui_entity = res_ui_element_entity.get_entity(&slot_def.cooldown);

        if let Ok((mut text_state, mut visibility)) = q_ui_text_state.get_mut(ui_entity) {
            let remaining = cooldown
                .timer
                .as_ref()
                .map(|v| v.remaining_secs())
                .unwrap_or(0.0);
            if remaining > 0.0 {
                text_state.text = remaining.ceil().to_string();
                if matches!(*visibility, Visibility::Hidden) {
                    *visibility = Visibility::Inherited;
                }
                commands
                    .entity(disabled_entity)
                    .insert(Visibility::Inherited);
            } else {
                text_state.text = "".to_string();
                if matches!(*visibility, Visibility::Inherited) {
                    *visibility = Visibility::Hidden;
                }
                commands.entity(disabled_entity).insert(Visibility::Hidden);
            }
        }
    }
}

fn update_icon_data(
    handle: &Handle<LOLUiElementEffectDesaturateData>,
    icon_name: &str,
    assets: &mut Assets<LOLUiElementEffectDesaturateData>,
) {
    let Some(mut icon_data) = assets.get_mut(handle) else {
        info!("未找到图标数据 {:?}", handle);
        return;
    };
    icon_data.texture_data = Some(AtlasData(LOLAtlasData {
        m_texture_name: icon_name.to_string(),
        m_texture_uv: None,
    }));
}

fn update_skill_level_up_button(
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
            info!("显示技能升级按钮");
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
    // info!("技能升级按钮更新完成");
}
