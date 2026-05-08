use bevy::prelude::*;
use lol_base::spell::Spell;
use lol_core::base::level::Level;
use lol_core::character::CharacterReady;
use lol_core::skill::{CommandSkillLevelUp, PassiveSkill, Skill, SkillPoints, Skills};

use crate::controller::Controller;
use crate::skin::skin::SkinReady;
use crate::ui::animation::CommandUiAnimationStart;
use crate::ui::button::{CommandDespawnButton, CommandSpawnButton};
use crate::ui::element::{UIElementEntity, UIState};

#[derive(Default)]
pub struct PluginUISkill;

impl Plugin for PluginUISkill {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillLevelUpButton>();
        app.add_systems(
            Update,
            (
                update_skill_level_up_button.run_if(
                    in_state(UIState::Loaded)
                        .and_then(any_match_filter::<(With<Controller>, With<CharacterReady>)>),
                ),
                update_player_skill_icon.run_if(
                    in_state(UIState::Loaded)
                        .and_then(any_match_filter::<(With<Controller>, With<CharacterReady>)>)
                        .and_then(run_once),
                ),
            ),
        );
    }
}

#[derive(Resource, Default)]
struct SkillLevelUpButton {
    pub entities: [Option<Entity>; 4],
}

fn update_player_skill_icon(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut q_image_node: Query<&mut ImageNode>,
    q_children: Query<&Children>,
    q_skill: Query<&Skill>,
    q_skills: Query<&Skills, With<Controller>>,
    q_passive_skill: Query<&PassiveSkill, With<Controller>>,
    res_assets_spell_object: Res<Assets<Spell>>,
    res_ui_element_entity: Res<UIElementEntity>,
) {
    let Ok(passive_skill) = q_passive_skill.single() else {
        info!("未找到控制器的被动技能");
        return;
    };

    let Ok(skills) = q_skills.single() else {
        info!("未找到控制器的技能列表");
        return;
    };

    for (index, skill) in passive_skill.iter().chain(skills.iter()).enumerate() {
        let key = if index == 0 {
            "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerSpells/Passive/Passive_IconLoc".to_string()
        } else {
            format!(
                "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerSpells/Ability{0}/Ability{0}_IconLoc",
                index - 1
            )
        };

        let Some(&entity) = res_ui_element_entity.get_by_string(&key) else {
            info!("未找到技能图标 UI 元素 {}", key);
            continue;
        };

        let Ok(children) = q_children.get(entity) else {
            continue;
        };

        let &child = children.get(0).unwrap();
        let mut image_node = q_image_node.get_mut(child).unwrap();
        if image_node.rect.is_none() {
            info!("技能图标 {} 的 rect 为空", index);
            continue;
        }

        let Ok(skill) = q_skill.get(skill) else {
            info!("未找到技能实体 {} 的技能组件", index);
            continue;
        };

        let spell = res_assets_spell_object.get(&skill.spell).unwrap();

        let icon_name = spell
            .spell_data
            .as_ref()
            .unwrap()
            .icon_path
            .as_ref()
            .unwrap()
            .get(0)
            .unwrap()
            .clone();

        image_node.image = asset_server.load(&icon_name);
        image_node.rect = None;

        commands.entity(entity).insert(Visibility::Visible);
    }
}

fn update_skill_level_up_button(
    mut commands: Commands,
    q_skill_points: Query<(Entity, &Level, &SkillPoints, &Skills), With<Controller>>,
    mut res_skill_level_up_button: ResMut<SkillLevelUpButton>,
    q_skill: Query<&Skill>,
) {
    let Ok((entity, level, skill_points, skills)) = q_skill_points.single() else {
        info!("未找到控制器的技能点信息");
        return;
    };

    for (index, skill_entity) in skills.iter().skip(1).enumerate() {
        let key_str = format!(
            "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/LevelUp/LevelUp{}_Button",
            index
        );

        let animation_key_str = format!(
            "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/LevelUpFxIn/LevelUp{}_ButtonIn",
            index
        );

        let key = league_utils::hash_bin(&key_str);

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
            if res_skill_level_up_button.entities[index].is_some() {
                continue;
            }

            info!("生成技能升级按钮 实体 {} 索引 {}", entity, index);
            let entity_button = commands
                .spawn_empty()
                .observe(move |_event: On<Pointer<Click>>, mut commands: Commands| {
                    commands.trigger(CommandSkillLevelUp { entity, index });
                })
                .id();

            res_skill_level_up_button.entities[index] = Some(entity_button);
            commands.trigger(CommandSpawnButton {
                hash: key,
                entity: Some(entity_button),
            });
            commands.trigger(CommandUiAnimationStart {
                key: animation_key_str,
            });
        } else {
            if let Some(entity_button) = res_skill_level_up_button.entities[index] {
                info!("销毁技能升级按钮 实体 {} 索引 {}", entity, index);
                res_skill_level_up_button.entities[index] = None;
                commands.trigger(CommandDespawnButton {
                    entity: entity_button,
                });
            }
        }
    }
    // info!("技能升级按钮更新完成");
}
