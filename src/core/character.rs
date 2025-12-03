use bevy::prelude::*;
use league_utils::hash_bin;

use crate::{
    AbilityResource, AbilityResourceType, Armor, Attack, Bounding, CommandSkinSpawn, Damage,
    EventDead, Health, Level, Movement, ResourceCache,
};

/// 生成角色的命令
#[derive(EntityEvent, Debug, Clone)]
pub struct CommandSpawnCharacter {
    pub entity: Entity,
    pub character_record_key: String,
    pub skin_path: String,
}

/// 角色组件标记
#[derive(Component, Debug)]
pub struct Character {
    pub character_record_key: String,
    pub skin_key: String,
}

#[derive(Default)]
pub struct PluginCharacter;

impl Plugin for PluginCharacter {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_spawn_character);
        app.add_observer(on_event_dead);
    }
}

/// 处理角色生成命令的观察者
fn on_command_spawn_character(
    trigger: On<CommandSpawnCharacter>,
    mut commands: Commands,
    resource_cache: Res<ResourceCache>,
) {
    let entity = trigger.event_target();

    // 查找 character 配置
    let character_record = match resource_cache
        .character_records
        .get(&trigger.character_record_key)
    {
        Some(record) => record,
        None => {
            error!(
                "Character record not found: {}",
                trigger.character_record_key
            );
            return;
        }
    };

    commands.trigger(CommandSkinSpawn {
        entity,
        skin_path: trigger.skin_path.clone(),
    });

    if let Some(primary_ability_resource) = &character_record.primary_ability_resource {
        // info!(
        //     "{} 的 primary_ability_resource: {:#?}",
        //     character_record.m_character_name, primary_ability_resource.ar_type
        // );
        let ar_type = AbilityResourceType::from(primary_ability_resource.ar_type);

        let ar = AbilityResource {
            ar_type,
            value: primary_ability_resource.ar_base.unwrap_or(0.0),
            max: primary_ability_resource.ar_base.unwrap_or(0.0),
            base: primary_ability_resource.ar_base.unwrap_or(0.0),
            per_level: primary_ability_resource.ar_per_level.unwrap_or(0.0),
            base_static_regen: primary_ability_resource.ar_base_static_regen.unwrap_or(0.0),
            regen_per_level: primary_ability_resource.ar_regen_per_level.unwrap_or(0.0),
        };

        commands.entity(entity).insert(ar);
    }
    // 根据 character_record 创建组件
    let health = Health::new(character_record.base_hp.unwrap_or(0.0));
    let damage = Damage(character_record.base_damage.unwrap_or(0.0));
    let armor = Armor(character_record.base_armor.unwrap_or(0.0));
    let movement = Movement {
        speed: character_record.base_move_speed.unwrap_or(0.0),
    };
    let bounding = Bounding {
        radius: character_record.pathfinding_collision_radius.unwrap_or(0.0),
        height: character_record.health_bar_height.unwrap_or(200.0),
    };

    commands.entity(entity).insert((
        Character {
            character_record_key: trigger.character_record_key.clone(),
            skin_key: trigger.skin_path.clone(),
        },
        health,
        movement,
        damage,
        armor,
        bounding,
    ));

    if let Some(attack_range) = &character_record.attack_range {
        if let Some(basic_attack) = &character_record.basic_attack {
            if let Some(cast_time) = basic_attack.m_attack_cast_time {
                if let Some(total_time) = basic_attack.m_attack_total_time {
                    commands.entity(entity).insert(
                        Attack::new(*attack_range, cast_time, total_time).with_missile(Some(
                            hash_bin(&format!(
                                "Characters/{}/Spells/{}BasicAttack",
                                character_record.m_character_name,
                                character_record.m_character_name
                            )),
                        )),
                    );
                }
            } else if let Some(m_attack_delay_cast_offset_percent) =
                basic_attack.m_attack_delay_cast_offset_percent
            {
                if let Some(attack_speed) = character_record.attack_speed {
                    commands.entity(entity).insert(
                        Attack::from_legacy(
                            *attack_range,
                            attack_speed,
                            m_attack_delay_cast_offset_percent,
                        )
                        .with_missile(Some(hash_bin(&format!(
                            "Characters/{}/Spells/{}BasicAttack",
                            character_record.m_character_name, character_record.m_character_name
                        )))),
                    );
                }
            }
        }
    }
}

fn on_event_dead(
    trigger: On<EventDead>,
    query: Query<(&Character, &GlobalTransform)>,
    mut level_query: Query<(&GlobalTransform, &mut Level)>,
    resource_cache: Res<ResourceCache>,
) {
    let entity = trigger.event_target();

    let Ok((character, transform)) = query.get(entity) else {
        return;
    };

    let Some(record) = resource_cache
        .character_records
        .get(&character.character_record_key)
    else {
        return;
    };

    let Some(exp) = record.exp_given_on_death else {
        return;
    };

    let Some(radius) = record.experience_radius else {
        return;
    };

    let position = transform.translation();
    for (target_transform, mut level) in level_query.iter_mut() {
        if target_transform.translation().distance(position) <= radius {
            level.add_experience(exp as u32);
        }
    }
}
