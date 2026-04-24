use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_loader::game::LeagueLoader;
use lol_core::attack::{Attack, WindupConfig};
use lol_core::base::bounding::Bounding;
use lol_core::base::level::ExperienceDrop;
use lol_core::character::Character;
use lol_core::damage::{Armor, Damage};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::movement::Movement;

use super::skin::extract_skin_for_champion;
use super::utils::write_to_file;

/// Champion 记录数据
#[derive(Clone)]
pub struct ChampionRecordData {
    pub char_record_path: String,
    pub skin_path: Option<String>,
}

/// 将 skin 路径转换为 skin bin 文件路径
/// 例如: "Characters/Aatrox/Skins/Skin0" -> "data/characters/aatrox/skins/skin0.bin"
pub fn skin_path_to_skin_bin_path(champ_name: &str, skin_path: &str) -> String {
    // skin_path 格式: "Characters/{Name}/Skins/{SkinName}"
    // 例如: "Characters/Aatrox/Skins/Skin0"
    // 输出: "data/characters/{name}/skins/{skinname}.bin"
    let parts: Vec<&str> = skin_path.split('/').collect();
    if parts.len() >= 4 {
        let skin_name = parts[parts.len() - 1].to_lowercase();
        format!(
            "data/characters/{}/skins/{}.bin",
            champ_name.to_lowercase(),
            skin_name
        )
    } else {
        // Fallback: 假设格式是 "{SkinName}"，直接拼接
        format!(
            "data/characters/{}/skins/{}.bin",
            champ_name.to_lowercase(),
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

    // TODO: 摸清字段对应关系后替换为正确的值
    // 占位符：使用 exp_given_on_death 作为 health（待替换）
    let health = Health::new(record.exp_given_on_death.unwrap_or(1000.0));

    // 占位符：使用 unk_0x43135375 作为 damage（待替换）
    let damage = Damage(record.unk_0x43135375.unwrap_or(50.0));

    // 占位符：使用 unk_0x4af40dc3 中的某个值作为 armor（待替换）
    let armor = Armor(record.area_indicator_radius.unwrap_or(30.0));

    // 占位符：使用 perception_bubble_radius 作为 move speed（待替换）
    let movement = Movement {
        speed: record.perception_bubble_radius.unwrap_or(5.0),
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

    (attack, health, damage, armor, movement, experience_drop)
}

/// 导出 champion 实体为 Scene 文件
pub fn export_champion_scene(
    champ_name: &str,
    bounding: Bounding,
    attack: Attack,
    health: Health,
    damage: Damage,
    armor: Armor,
    movement: Movement,
    experience_drop: Option<ExperienceDrop>,
) {
    let mut world = World::new();

    // 注册类型
    let type_registry = AppTypeRegistry::default();
    type_registry.write().register::<Champion>();
    type_registry.write().register::<Bounding>();
    type_registry.write().register::<Transform>();
    type_registry.write().register::<GlobalTransform>();
    type_registry.write().register::<Name>();
    type_registry.write().register::<Attack>();
    type_registry.write().register::<WindupConfig>();
    type_registry.write().register::<Health>();
    type_registry.write().register::<Damage>();
    type_registry.write().register::<Armor>();
    type_registry.write().register::<Movement>();
    type_registry.write().register::<ExperienceDrop>();
    world.insert_resource(type_registry);

    let champ_name_string = champ_name.to_string();
    let entity = world
        .spawn((
            Character,
            Name::new(champ_name_string),
            Champion,
            bounding,
            attack,
            health,
            damage,
            armor,
            movement,
        ))
        .id();

    if let Some(exp_drop) = experience_drop {
        world.entity_mut(entity).insert(exp_drop);
    }

    let scene = DynamicWorld::from_world(&world);
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    let output_path = format!("assets/characters/{}/config.ron", champ_name.to_lowercase());
    write_to_file(&output_path, serialized_scene);
}

/// 从 CharacterRecord 提取 champion 并导出场景
pub fn extract_champion_from_record(
    loader: &LeagueLoader,
    champ_name: &str,
    skin_bin_path: Option<&str>,
) -> bool {
    let bin_path = format!("data/characters/{}/{}.bin", champ_name, champ_name);

    let Ok(prop_group) = loader.get_prop_group_by_paths(vec![&bin_path]) else {
        println!("[WARN] 无法加载 bin 文件: {}", bin_path);
        return false;
    };

    let Some(record) = prop_group.get_by_class::<CharacterRecord>() else {
        println!("[WARN] 无法获取 CharacterRecord: {}", bin_path);
        return false;
    };

    let bounding = Bounding {
        radius: record.pathfinding_collision_radius.unwrap_or(0.0),
        height: record.health_bar_height.unwrap_or(200.0),
    };

    let (attack, health, damage, armor, movement, experience_drop) =
        create_champion_components_from_record(&record);
    export_champion_scene(
        champ_name,
        bounding,
        attack,
        health,
        damage,
        armor,
        movement,
        experience_drop,
    );

    // 提取皮肤 GLB 和皮肤场景
    extract_skin_for_champion(loader, champ_name, skin_bin_path);

    true
}
