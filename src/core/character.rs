use bevy::prelude::*;

use crate::{Armor, Attack, Bounding, CommandSkinSpawn, Damage, Health, Movement, ResourceCache};

/// 生成角色的命令
#[derive(EntityEvent, Debug, Clone)]
pub struct CommandSpawnCharacter {
    pub entity: Entity,
    pub character_record_key: String,
    pub skin_path: String,
}

/// 角色组件标记
#[derive(Component, Debug)]
pub struct Character;

#[derive(Default)]
pub struct PluginCharacter;

impl Plugin for PluginCharacter {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_spawn_character);
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

    commands
        .entity(entity)
        .insert((Character, health, movement, damage, armor, bounding));

    if let Some(attack_range) = &character_record.attack_range {
        if let Some(basic_attack) = &character_record.basic_attack {
            if let Some(cast_time) = basic_attack.m_attack_cast_time {
                if let Some(total_time) = basic_attack.m_attack_total_time {
                    commands.entity(entity).insert(Attack::new(
                        *attack_range,
                        cast_time,
                        total_time,
                    ));
                }
            } else if let Some(m_attack_delay_cast_offset_percent) =
                basic_attack.m_attack_delay_cast_offset_percent
            {
                if let Some(attack_speed) = character_record.attack_speed {
                    commands.entity(entity).insert(Attack::from_legacy(
                        *attack_range,
                        attack_speed,
                        m_attack_delay_cast_offset_percent,
                    ));
                }
            }
        }
    }
}
