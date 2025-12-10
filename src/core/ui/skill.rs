use bevy::prelude::*;
use league_core::SpellObject;
use lol_config::LeagueProperties;

use crate::{
    CommandDespawnButton, CommandSkillLevelUp, CommandSpawnButton, Controller, Level,
    ResourceCache, Skill, SkillPoints, Skills, UIElementEntity,
};

pub fn update_skill_icon(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut q_image_node: Query<&mut ImageNode>,
    mut res_resource_cache: ResMut<ResourceCache>,
    q_children: Query<&Children>,
    q_skill: Query<&Skill>,
    q_skills: Query<&Skills, With<Controller>>,
    res_assets_spell_object: Res<Assets<SpellObject>>,
    res_league_properties: Res<LeagueProperties>,
    res_ui_element_entity: Res<UIElementEntity>,
) {
    let Some(skills) = q_skills.iter().next() else {
        debug!("未找到控制器的技能列表");
        return;
    };

    for (index, skill) in skills.iter().enumerate() {
        let key = if index == 0 {
            "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerSpells/Passive/Passive_IconLoc".to_string()
        } else {
            format!("ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerSpells/Ability{0}/Ability{0}_IconLoc", index - 1)
        };

        let Some(&entity) = res_ui_element_entity.get_by_string(&key) else {
            debug!("未找到技能图标 UI 元素 {}", key);
            continue;
        };

        let Ok(children) = q_children.get(entity) else {
            continue;
        };
        let &child = children.get(0).unwrap();
        let mut image_node = q_image_node.get_mut(child).unwrap();
        if image_node.rect.is_none() {
            debug!("技能图标 {} 的 rect 为空", index);
            continue;
        }

        let Ok(skill) = q_skill.get(skill) else {
            debug!("未找到技能实体 {} 的技能组件", index);
            continue;
        };

        let spell = res_league_properties
            .get(&res_assets_spell_object, skill.key_spell_object)
            .unwrap();

        let icon_name = spell
            .m_spell
            .as_ref()
            .unwrap()
            .m_img_icon_name
            .as_ref()
            .unwrap()
            .get(0)
            .unwrap()
            .clone();

        image_node.image = res_resource_cache.get_image(&asset_server, &icon_name);
        image_node.rect = None;

        commands.entity(entity).insert(Visibility::Visible);
    }
}

#[derive(Resource, Default)]
pub struct SkillLevelUpButton {
    pub entities: [Option<Entity>; 4],
}

pub fn update_skill_level_up_button(
    mut commands: Commands,
    q_skill_points: Query<(Entity, &Level, &SkillPoints, &Skills), With<Controller>>,
    mut res_skill_level_up_button: ResMut<SkillLevelUpButton>,
    q_skill: Query<&Skill>,
) {
    let Ok((entity, level, skill_points, skills)) = q_skill_points.single() else {
        debug!("未找到控制器的技能点信息");
        return;
    };

    for (index, skill_entity) in skills.iter().skip(1).enumerate() {
        let key_str = format!(
            "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/LevelUp/LevelUp{}_Button",
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

            debug!("生成技能升级按钮 实体 {} 索引 {}", entity, index);
            let entity_button = commands
                .spawn_empty()
                .observe(move |_event: On<Pointer<Click>>, mut commands: Commands| {
                    commands.trigger(CommandSkillLevelUp { entity, index });
                })
                .id();
            res_skill_level_up_button.entities[index] = Some(entity_button);
            commands.trigger(CommandSpawnButton {
                key: key.into(),
                entity: Some(entity_button),
            });
        } else {
            if let Some(entity_button) = res_skill_level_up_button.entities[index] {
                debug!("销毁技能升级按钮 实体 {} 索引 {}", entity, index);
                res_skill_level_up_button.entities[index] = None;
                commands.trigger(CommandDespawnButton {
                    entity: entity_button,
                });
            }
        }
    }
    debug!("技能升级按钮更新完成");
}
