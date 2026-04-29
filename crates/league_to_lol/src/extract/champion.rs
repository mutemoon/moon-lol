use bevy::prelude::*;
use league_core::extract::{AbilityResourceSlotInfo, CharacterRecord, SpellObject};
use league_loader::game::{Data, LeagueLoader, PropGroup};
use lol_base::spell::Spell;
use lol_core::attack::{Attack, WindupConfig};
use lol_core::base::ability_resource::{AbilityResource, AbilityResourceType};
use lol_core::base::bounding::Bounding;
use lol_core::base::level::ExperienceDrop;
use lol_core::character::Character;
use lol_core::damage::{Armor, Damage};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::movement::Movement;
use lol_core::skill::{CoolDown, Skill, SkillCooldownMode, SkillOf, SkillSlot};

use super::skin::extract_skin_for_champion;
use super::spell::extract_spells_for_champion;
use super::utils::write_to_file;

/// Champion 记录数据
#[derive(Clone)]
pub struct ChampionRecordData {
    pub char_record_path: String,
    pub skin_path: Option<String>,
}

/// 将 skin 路径转换为 skin bin 文件路径
/// 例如: "Characters/Aatrox/Skins/Skin0" -> "data/characters/aatrox/skins/skin0.bin"
pub fn skin_path_to_skin_bin_path(character_name: &str, skin_path: &str) -> String {
    // skin_path 格式: "Characters/{Name}/Skins/{SkinName}"
    // 例如: "Characters/Aatrox/Skins/Skin0"
    // 输出: "data/characters/{name}/skins/{skinname}.bin"
    let parts: Vec<&str> = skin_path.split('/').collect();
    if parts.len() >= 4 {
        let skin_name = parts[parts.len() - 1].to_lowercase();
        format!(
            "data/characters/{}/skins/{}.bin",
            character_name.to_lowercase(),
            skin_name
        )
    } else {
        // Fallback: 假设格式是 "{SkinName}"，直接拼接
        format!(
            "data/characters/{}/skins/{}.bin",
            character_name.to_lowercase(),
            skin_path.to_lowercase()
        )
    }
}

/// 从 CharacterRecord 创建所有组件（占位符，待摸清对应关系）
pub fn create_champion_components_from_record(
    record: &CharacterRecord,
) -> (
    Attack,
    Health,
    Damage,
    Armor,
    Movement,
    Option<ExperienceDrop>,
    Option<AbilityResource>,
) {
    // Attack
    let range = record.acquisition_range.unwrap_or(0.0);
    let windup_config = if let Some(basic_attack) = &record.basic_attack {
        let cast_time = basic_attack.m_attack_cast_time.unwrap_or(0.3);
        let total_time = basic_attack.m_attack_total_time.unwrap_or(0.7);
        WindupConfig::Modern {
            attack_cast_time: cast_time,
            attack_total_time: total_time,
        }
    } else {
        WindupConfig::Legacy { attack_offset: 0.3 }
    };
    let attack = match windup_config {
        WindupConfig::Modern {
            attack_cast_time,
            attack_total_time,
        } => Attack::new(range, attack_cast_time, attack_total_time),
        WindupConfig::Legacy { attack_offset } => Attack::from_legacy(range, 1.0, attack_offset),
    };

    let health = Health::new(
        record
            .base_hp_modifiable
            .as_ref()
            .map(|v| v.base_value)
            .unwrap_or(0.0),
    );

    let damage = Damage(
        record
            .base_damage_modifiable
            .as_ref()
            .map(|v| v.base_value)
            .unwrap_or(0.0),
    );

    let armor = Armor(
        record
            .armor_per_level_modifiable
            .as_ref()
            .map(|v| v.base_value)
            .unwrap_or(0.0),
    );

    let movement = Movement {
        speed: record
            .base_move_speed_modifiable
            .as_ref()
            .map(|v| v.base_value)
            .unwrap_or(0.0),
    };

    // 经验掉落
    let experience_drop =
        if let (Some(exp), Some(radius)) = (record.exp_given_on_death, record.experience_radius) {
            Some(ExperienceDrop {
                exp_given_on_death: exp,
                experience_radius: radius,
            })
        } else {
            None
        };

    // 资源数据（蓝条/能量条等）
    let ability_resource = create_ability_resource_from_record(&record.primary_ability_resource);

    (
        attack,
        health,
        damage,
        armor,
        movement,
        experience_drop,
        ability_resource,
    )
}

/// 从 AbilityResourceSlotInfo 创建 AbilityResource 组件
fn create_ability_resource_from_record(
    slot_info: &Option<AbilityResourceSlotInfo>,
) -> Option<AbilityResource> {
    let slot_info = slot_info.as_ref()?;

    let ar_type = AbilityResourceType::from(slot_info.ar_type);
    let base = slot_info
        .unk_0x3a509002
        .as_ref()
        .map(|v| v.base_value)
        .unwrap_or(0.0);
    let per_level = slot_info
        .unk_0x452033bb
        .as_ref()
        .map(|v| v.base_value)
        .unwrap_or(0.0);
    let base_static_regen = slot_info
        .unk_0x6216bf7b
        .as_ref()
        .map(|v| v.base_value)
        .unwrap_or(0.0);
    let regen_per_level = slot_info
        .unk_0x726ee5cd
        .as_ref()
        .map(|v| v.base_value)
        .unwrap_or(0.0);

    Some(AbilityResource {
        ar_type,
        value: base,
        max: base,
        base,
        per_level,
        base_static_regen,
        regen_per_level,
    })
}

/// 从 CharacterRecord 提取 character 并导出场景
pub fn extract_character_from_record(
    loader: &LeagueLoader,
    character_name: &str,
    is_champion: bool,
    skin_bin_path: Option<&str>,
) -> bool {
    let bin_path = format!("data/characters/{}/{}.bin", character_name, character_name);

    let Ok(prop_group) = loader.get_prop_group_by_paths(vec![&bin_path]) else {
        println!("[WARN] 无法加载 bin 文件: {}", bin_path);
        return false;
    };

    let Some(record) = prop_group.get_by_class::<CharacterRecord>() else {
        println!("[WARN] 无法获取 CharacterRecord: {}", bin_path);
        return false;
    };

    // 提取技能数据到文件
    extract_spells_for_champion(character_name, &prop_group);

    // 创建 App 用于获取 AssetServer
    let mut app = App::new();
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(TaskPoolPlugin::default());
    app.init_asset::<Spell>();
    app.finish();
    app.cleanup();

    let asset_server = app.world().resource::<AssetServer>().clone();

    let bounding = Bounding {
        radius: record.pathfinding_collision_radius.unwrap_or(0.0),
        height: record.health_bar_height.unwrap_or(200.0),
    };

    let (attack, health, damage, armor, movement, experience_drop, ability_resource) =
        create_champion_components_from_record(&record);

    // 从 spells 哈希创建 Skill 组件（使用 AssetServer 加载路径以获得正确的 Handle）
    let skills = create_skills_from_record(&prop_group, character_name, &record, &asset_server);

    let world = app.world_mut();

    let character_name_string = character_name.to_string();
    let mut builder = world.spawn((
        Character,
        Name::new(character_name_string.clone()),
        bounding,
        attack,
        health,
        damage,
        armor,
        movement,
    ));

    if is_champion {
        builder.insert(Champion);
    }

    let champion_entity = builder.id();

    if let Some(exp_drop) = experience_drop {
        world.entity_mut(champion_entity).insert(exp_drop);
    }

    if let Some(ar) = ability_resource {
        world.entity_mut(champion_entity).insert(ar);
    }

    // 为每个技能创建独立的技能实体
    for skill in skills {
        world.entity_mut(champion_entity).with_related::<SkillOf>((
            skill,
            CoolDown {
                timer: None,
                duration: 0.0,
            },
        ));
    }

    let scene = DynamicWorld::from_world(world);
    let type_registry = app.world().resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    let output_path = format!(
        "assets/characters/{}/config.ron",
        character_name.to_lowercase()
    );
    write_to_file(&output_path, serialized_scene);

    extract_skin_for_champion(loader, character_name, skin_bin_path);

    true
}

/// 从 CharacterRecord 创建 Skill 组件
fn create_skills_from_record(
    prop_group: &PropGroup,
    character_name: &str,
    record: &CharacterRecord,
    asset_server: &AssetServer,
) -> Vec<Skill> {
    let mut skills = Vec::new();

    // 获取 Q/W/E/R 技能
    if let Some(spell_hashes) = &record.spells {
        let slots = [SkillSlot::Q, SkillSlot::W, SkillSlot::E, SkillSlot::R];
        for (i, hash) in spell_hashes.iter().enumerate() {
            if i >= slots.len() {
                break;
            }
            let slot = slots[i];

            if let Some(spell_obj) = prop_group.get_data_option::<SpellObject>(*hash) {
                let spell_path = format!(
                    "characters/{}/spells/{}.ron",
                    character_name.to_lowercase(),
                    spell_obj.object_name
                );
                let spell_handle: Handle<Spell> = asset_server.load(&spell_path);
                skills.push(Skill {
                    spell: spell_handle,
                    level: 1,
                    slot,
                    cooldown_mode: SkillCooldownMode::AfterCast,
                });
                println!(
                    "[INFO] 技能 {} -> {}",
                    slot_to_string(slot),
                    spell_obj.object_name
                );
            }
        }
    }

    // 获取被动技能
    if let Some(passive_hash) = record.m_character_passive_spell {
        if let Some(spell_obj) = prop_group.get_data_option::<SpellObject>(passive_hash) {
            let spell_path = format!(
                "characters/{}/spells/{}.ron",
                character_name.to_lowercase(),
                spell_obj.object_name
            );
            let spell_handle: Handle<Spell> = asset_server.load(&spell_path);
            skills.push(Skill {
                spell: spell_handle,
                level: 1,
                slot: SkillSlot::Passive,
                cooldown_mode: SkillCooldownMode::AfterCast,
            });
            println!("[INFO] 被动技能 -> {}", spell_obj.object_name);
        }
    }

    skills
}

/// SkillSlot 转字符串
fn slot_to_string(slot: SkillSlot) -> &'static str {
    match slot {
        SkillSlot::Passive => "Passive",
        SkillSlot::Q => "Q",
        SkillSlot::W => "W",
        SkillSlot::E => "E",
        SkillSlot::R => "R",
        SkillSlot::Custom(_) => "Custom",
    }
}
