use bevy::prelude::*;
use league_core::{CharacterRecord, SkinCharacterDataProperties};
use lol_config::{HashKey, LeagueProperties};
use lol_core::Team;

use crate::{
    AbilityResource, AbilityResourceType, Armor, Attack, Bounding, CommandLoadPropBin,
    CommandSkinSpawn, Damage, EventDead, EventLevelUp, Health, Level, Loading, Movement,
};

#[derive(Default)]
pub struct PluginCharacter;

impl Plugin for PluginCharacter {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_character_spawn);
        app.add_observer(on_event_dead);

        app.add_systems(Update, update_character_spawn);
    }
}

/// 角色组件标记
#[derive(Component, Debug)]
pub struct Character {
    pub key: HashKey<CharacterRecord>,
}

/// 生成角色的命令
#[derive(EntityEvent, Debug, Clone)]
pub struct CommandCharacterSpawn {
    pub entity: Entity,
    pub character_record: String,
    pub skin: String,
}

#[derive(TypePath)]
pub struct CharacterSpawn(pub HashKey<CharacterRecord>);

/// 处理角色生成命令的观察者
fn on_command_character_spawn(trigger: On<CommandCharacterSpawn>, mut commands: Commands) {
    let entity = trigger.event_target();

    commands.trigger(CommandSkinSpawn {
        entity,
        key: trigger.skin.clone(),
    });

    let name = trigger.character_record.split('/').skip(1).next().unwrap();

    let paths = vec![format!("data/characters/{name}/{name}.bin")];

    commands.trigger(CommandLoadPropBin { paths });

    commands.entity(entity).insert(Loading::new(CharacterSpawn(
        trigger.character_record.clone().into(),
    )));
}

fn update_character_spawn(
    mut commands: Commands,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
    res_league_properties: Res<LeagueProperties>,
    q_loading: Query<(Entity, &Loading<CharacterSpawn>)>,
) {
    for (entity, loading) in q_loading.iter() {
        let Some(character_record) =
            res_league_properties.get(&res_assets_character_record, loading.0)
        else {
            return;
        };

        commands.entity(entity).remove::<Loading<CharacterSpawn>>();

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
            Character { key: loading.0 },
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
                                (&format!(
                                    "Characters/{}/Spells/{}BasicAttack",
                                    character_record.m_character_name,
                                    character_record.m_character_name
                                ))
                                    .into(),
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
                            .with_missile(Some(
                                (&format!(
                                    "Characters/{}/Spells/{}BasicAttack",
                                    character_record.m_character_name,
                                    character_record.m_character_name
                                ))
                                    .into(),
                            )),
                        );
                    }
                }
            }
        }
    }
}

fn on_event_dead(
    event: On<EventDead>,
    query: Query<(&GlobalTransform, &Character, &Team)>,
    mut level_query: Query<(Entity, &GlobalTransform, &Team, &mut Level)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
    res_league_properties: Res<LeagueProperties>,
    mut commands: Commands,
) {
    let entity = event.event_target();

    let Ok((transform, character, team)) = query.get(entity) else {
        return;
    };

    let Some(record) = res_league_properties.get(&res_assets_character_record, character.key)
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
