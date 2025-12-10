use bevy::prelude::*;

use league_core::{CharacterRecord, ResourceResolver, SkinCharacterDataProperties};
use league_utils::{get_asset_id_by_hash, get_asset_id_by_path};
use lol_core::Team;

use crate::{
    AbilityResource, AbilityResourceType, Armor, Attack, Bounding, CommandParticleDespawn,
    CommandParticleSpawn, CommandSkinSpawn, Damage, EventDead, EventLevelUp, Health, Level,
    Movement,
};

#[derive(Default)]
pub struct PluginCharacter;

impl Plugin for PluginCharacter {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_character_spawn);
        app.add_observer(on_event_dead);
        app.add_observer(on_command_character_particle_spawn);
        app.add_observer(on_command_character_particle_despawn);
    }
}

/// 角色组件标记
#[derive(Component, Debug)]
pub struct Character {
    pub character_record_key: AssetId<CharacterRecord>,
    pub skin_key: AssetId<SkinCharacterDataProperties>,
}

/// 生成角色的命令
#[derive(EntityEvent, Debug, Clone)]
pub struct CommandCharacterSpawn {
    pub entity: Entity,
    pub character_record_key: AssetId<CharacterRecord>,
    pub skin_key: AssetId<SkinCharacterDataProperties>,
}

#[derive(EntityEvent)]
pub struct CommandCharacterParticleSpawn {
    pub entity: Entity,
    pub hash: u32,
}

#[derive(EntityEvent)]
pub struct CommandCharacterParticleDespawn {
    pub entity: Entity,
    pub hash: u32,
}

/// 处理角色生成命令的观察者
fn on_command_character_spawn(
    trigger: On<CommandCharacterSpawn>,
    mut commands: Commands,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    let entity = trigger.event_target();

    // 查找 character 配置
    let Some(character_record) = res_assets_character_record.get(trigger.character_record_key)
    else {
        return;
    };

    commands.trigger(CommandSkinSpawn {
        entity,
        skin_key: trigger.skin_key.clone(),
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
            skin_key: trigger.skin_key,
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
                            get_asset_id_by_path(&format!(
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
                        .with_missile(Some(get_asset_id_by_path(
                            &format!(
                                "Characters/{}/Spells/{}BasicAttack",
                                character_record.m_character_name,
                                character_record.m_character_name
                            ),
                        ))),
                    );
                }
            }
        }
    }
}

fn on_command_character_particle_spawn(
    trigger: On<CommandCharacterParticleSpawn>,
    res_assets_resource_resolver: Res<Assets<ResourceResolver>>,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
    mut commands: Commands,
    query: Query<&Character>,
) {
    let entity = trigger.event_target();

    let Ok(character) = query.get(entity) else {
        return;
    };

    let skin_character_data_properties = res_assets_skin_character_data_properties
        .get(character.skin_key)
        .unwrap();

    let m_resource_resolver = skin_character_data_properties.m_resource_resolver.unwrap();

    let resource_resolver = res_assets_resource_resolver
        .get(get_asset_id_by_hash(m_resource_resolver))
        .unwrap();

    let Some(record) = resource_resolver
        .resource_map
        .as_ref()
        .unwrap()
        .get(&trigger.hash)
    else {
        return;
    };

    commands.trigger(CommandParticleSpawn {
        entity,
        hash: get_asset_id_by_hash(*record),
    });
}

fn on_command_character_particle_despawn(
    trigger: On<CommandCharacterParticleDespawn>,
    res_assets_resource_resolver: Res<Assets<ResourceResolver>>,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
    mut commands: Commands,
    query: Query<&Character>,
) {
    let entity = trigger.event_target();

    let Ok(character) = query.get(entity) else {
        return;
    };

    let skin_character_data_properties = res_assets_skin_character_data_properties
        .get(character.skin_key)
        .unwrap();

    let m_resource_resolver = skin_character_data_properties.m_resource_resolver.unwrap();

    let resource_resolver = res_assets_resource_resolver
        .get(get_asset_id_by_hash(m_resource_resolver))
        .unwrap();

    let Some(record) = resource_resolver
        .resource_map
        .as_ref()
        .unwrap()
        .get(&trigger.hash)
    else {
        return;
    };

    commands.trigger(CommandParticleDespawn {
        entity,
        hash: get_asset_id_by_hash(*record),
    });
}

fn on_event_dead(
    event: On<EventDead>,
    query: Query<(&GlobalTransform, &Character, &Team)>,
    mut level_query: Query<(Entity, &GlobalTransform, &Team, &mut Level)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
    mut commands: Commands,
) {
    let entity = event.event_target();

    let Ok((transform, character, team)) = query.get(entity) else {
        return;
    };

    let Some(record) = res_assets_character_record.get(character.character_record_key) else {
        return;
    };

    let Some(exp) = record.exp_given_on_death else {
        return;
    };

    let Some(radius) = record.experience_radius else {
        return;
    };

    let position = transform.translation();
    for (target_entity, target_transform, target_team, mut level) in level_query.iter_mut() {
        if target_team != team {
            continue;
        }

        if target_transform.translation().distance(position) > radius {
            continue;
        }

        let levels_gained = level.add_experience(exp as u32);
        if levels_gained == 0 {
            continue;
        }

        commands.trigger(EventLevelUp {
            entity: target_entity,
            level: level.value,
            delta: levels_gained,
        });
    }
}
