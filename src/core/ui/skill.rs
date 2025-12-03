use bevy::prelude::*;

use crate::{Controller, ResourceCache, Skill, Skills, UIElementEntity};

pub fn update_skill_icon(
    mut commands: Commands,
    mut res_resource_cache: ResMut<ResourceCache>,
    asset_server: Res<AssetServer>,
    q_skills: Query<&Skills, With<Controller>>,
    q_skill: Query<&Skill>,
    res_ui_element_entity: Res<UIElementEntity>,
    q_children: Query<&Children>,
    mut q_image_node: Query<&mut ImageNode>,
) {
    let Ok(skills) = q_skills.single() else {
        return;
    };

    for (index, skill) in skills.iter().enumerate() {
        let key = if index == 0 {
            "ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerSpells/Passive/Passive_IconLoc".to_string()
        } else {
            format!("ClientStates/Gameplay/UX/LoL/PlayerFrame/UIBase/Player_Frame_Root/PlayerSpells/Ability{0}/Ability{0}_IconLoc", index - 1)
        };

        let Some(&entity) = res_ui_element_entity.get_by_string(&key) else {
            continue;
        };
        let &child = q_children.get(entity).unwrap().get(0).unwrap();
        let mut image_node = q_image_node.get_mut(child).unwrap();
        if image_node.rect.is_none() {
            continue;
        }

        let Ok(skill) = q_skill.get(skill) else {
            continue;
        };

        let spell = res_resource_cache.spells.get(&skill.key).unwrap();

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
