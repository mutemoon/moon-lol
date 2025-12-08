use std::collections::{HashMap, HashSet};

use heck::{ToPascalCase, ToSnakeCase};
use league_utils::hash_to_type_name;

use crate::{BinParser, BinType, EntryData, Error, PropFile};

#[derive(Debug, Clone)]
pub enum ClassData {
    Base(String),
    Struct(u32),
    Enum(HashSet<u32>),
    List(Box<ClassData>),
    Map(Box<ClassData>, Box<ClassData>),
    Option(Box<ClassData>),
}

#[derive(Debug, Clone)]
struct EnumInfo {
    name: String,
    hashes: HashSet<u32>,
}

type ClassMap = HashMap<u32, HashMap<u32, ClassData>>;

fn collect_enum_info_recursively(
    parent_class_name: &str,
    field_hash: u32,
    field_data: &ClassData,
    hashes: &HashMap<u32, String>,
    enum_infos: &mut Vec<EnumInfo>,
) {
    match field_data {
        ClassData::Enum(enum_hashes) => {
            let field_name = hashes
                .get(&field_hash)
                .map(|s| s.as_str())
                .unwrap_or("UnknownEnumField");

            let enum_name = format!("{}{}", parent_class_name, field_name.to_pascal_case());

            enum_infos.push(EnumInfo {
                name: enum_name,
                hashes: enum_hashes.clone(),
            });
        }

        ClassData::List(element_data) => {
            collect_enum_info_recursively(
                parent_class_name,
                field_hash,
                element_data,
                hashes,
                enum_infos,
            );
        }
        ClassData::Map(key_data, value_data) => {
            collect_enum_info_recursively(
                parent_class_name,
                field_hash,
                key_data,
                hashes,
                enum_infos,
            );
            collect_enum_info_recursively(
                parent_class_name,
                field_hash,
                value_data,
                hashes,
                enum_infos,
            );
        }
        ClassData::Option(value_data) => {
            collect_enum_info_recursively(
                parent_class_name,
                field_hash,
                value_data,
                hashes,
                enum_infos,
            );
        }
        ClassData::Base(_) | ClassData::Struct(_) => {}
    }
}

fn merge_enums_with_intersection(enum_infos: Vec<EnumInfo>) -> Vec<EnumInfo> {
    let mut merged_enums = Vec::new();
    let mut processed = vec![false; enum_infos.len()];

    for i in 0..enum_infos.len() {
        if processed[i] {
            continue;
        }

        let mut current_merged = enum_infos[i].clone();
        processed[i] = true;

        let mut j = i + 1;
        while j < enum_infos.len() {
            if processed[j] {
                j += 1;
                continue;
            }

            // 检查是否有交集
            if !current_merged.hashes.is_disjoint(&enum_infos[j].hashes) {
                // 合并枚举，名字使用第一个
                current_merged.hashes.extend(enum_infos[j].hashes.iter());
                processed[j] = true;
                // 从头开始重新检查，以处理传递合并 a-b, c-d, b-c
                j = i + 1;
            } else {
                j += 1;
            }
        }
        merged_enums.push(current_merged);
    }

    merged_enums
}

fn contains_enum_reference(
    field_data: &ClassData,
    target_enum_hashes: &HashSet<u32>,
    class_map: &ClassMap,
) -> bool {
    match field_data {
        ClassData::Enum(enum_hashes) => {
            // 检查是否有交集，即该字段引用了目标枚举
            !enum_hashes.is_disjoint(target_enum_hashes)
        }
        ClassData::Struct(struct_hash) => {
            // 递归检查结构体字段
            if let Some(struct_fields) = class_map.get(struct_hash) {
                for (_, nested_field_data) in struct_fields {
                    if contains_enum_reference(nested_field_data, target_enum_hashes, class_map) {
                        return true;
                    }
                }
            }
            false
        }
        ClassData::List(element_data) => {
            contains_enum_reference(element_data, target_enum_hashes, class_map)
        }
        ClassData::Map(_, value_data) => {
            contains_enum_reference(value_data, target_enum_hashes, class_map)
        }
        ClassData::Option(value_data) => {
            contains_enum_reference(value_data, target_enum_hashes, class_map)
        }
        ClassData::Base(_) => false,
    }
}

fn generate_enum_definitions(
    merged_enums: &[EnumInfo],
    class_map: &mut ClassMap,
    hashes: &HashMap<u32, String>,
) -> String {
    let mut all_definitions = String::new();
    let mut generated_names = HashSet::new();

    for enum_info in merged_enums {
        if generated_names.contains(&enum_info.name) {
            continue;
        }

        let mut enum_def = String::new();
        enum_def.push_str("#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]\n");
        enum_def.push_str(&format!("pub enum {} {{\n", enum_info.name));

        for variant_hash in &enum_info.hashes {
            let mut variant_name = hash_to_type_name(variant_hash, hashes);

            if variant_name == "Self" {
                variant_name = "MySelf".to_string();
            }

            let is_empty_struct = class_map
                .get(variant_hash)
                .map_or(true, |fields| fields.is_empty());

            if is_empty_struct {
                enum_def.push_str(&format!("    {},\n", variant_name));
                class_map.remove(variant_hash);
            } else {
                // 不再检查递归引用，直接生成变体
                enum_def.push_str(&format!("    {}({}),\n", variant_name, variant_name));
            }
        }

        enum_def.push_str("}\n\n");
        all_definitions.push_str(&enum_def);
        generated_names.insert(enum_info.name.clone());
    }

    all_definitions
}

fn map_class_data_to_rust_type(
    parent_class_name: &str,
    field_data: &ClassData,
    field_hash: &u32,
    hashes: &HashMap<u32, String>,
    enum_hash_to_name: &HashMap<u32, String>,
) -> String {
    match field_data {
        ClassData::Base(type_string) => type_string.clone(),
        ClassData::Struct(struct_hash) => hash_to_type_name(struct_hash, hashes),
        ClassData::Enum(enum_hashes) => {
            // 从枚举的哈希集合中取第一个哈希，查找合并后的枚举名称
            if let Some(&first_hash) = enum_hashes.iter().next() {
                enum_hash_to_name
                    .get(&first_hash)
                    .cloned()
                    .unwrap_or_else(|| {
                        let field_name = hashes
                            .get(field_hash)
                            .map(|s| s.to_pascal_case())
                            .unwrap_or(format!("UnkEnum{:x}", field_hash));
                        format!("{}{}", parent_class_name, field_name)
                    })
            } else {
                // 空枚举的情况
                let field_name = hashes
                    .get(field_hash)
                    .map(|s| s.to_pascal_case())
                    .unwrap_or(format!("UnkEnum{:x}", field_hash));
                format!("{}{}", parent_class_name, field_name)
            }
        }
        ClassData::List(element_data) => {
            format!(
                "Vec<{}>",
                map_class_data_to_rust_type(
                    parent_class_name,
                    element_data,
                    field_hash,
                    hashes,
                    enum_hash_to_name
                )
            )
        }
        ClassData::Map(key_data, value_data) => {
            format!(
                "HashMap<{}, {}>",
                map_class_data_to_rust_type(
                    parent_class_name,
                    key_data,
                    field_hash,
                    hashes,
                    enum_hash_to_name
                ),
                map_class_data_to_rust_type(
                    parent_class_name,
                    value_data,
                    field_hash,
                    hashes,
                    enum_hash_to_name
                )
            )
        }
        ClassData::Option(value_data) => {
            format!(
                "Option<{}>",
                map_class_data_to_rust_type(
                    parent_class_name,
                    value_data,
                    field_hash,
                    hashes,
                    enum_hash_to_name
                )
            )
        }
    }
}

// ... a lot of unchanged functions
fn map_base_type(bin_type: &BinType) -> String {
    match bin_type {
        BinType::None => "()".to_string(),
        BinType::Bool => "bool".to_string(),
        BinType::S8 => "i8".to_string(),
        BinType::U8 => "u8".to_string(),
        BinType::S16 => "i16".to_string(),
        BinType::U16 => "u16".to_string(),
        BinType::S32 => "i32".to_string(),
        BinType::U32 => "u32".to_string(),
        BinType::S64 => "i64".to_string(),
        BinType::U64 => "u64".to_string(),
        BinType::Float => "f32".to_string(),
        BinType::Vec2 => "Vec2".to_string(),
        BinType::Vec3 => "Vec3".to_string(),
        BinType::Vec4 => "Vec4".to_string(),
        BinType::Matrix => "Mat4".to_string(),
        BinType::Color => "[u8; 4]".to_string(),
        BinType::String => "String".to_string(),
        BinType::Path => "u64".to_string(),
        BinType::Hash => "u32".to_string(),
        BinType::Link => "u32".to_string(),
        BinType::Flag => "bool".to_string(),
        _ => panic!("不是基础类型: {:?}", bin_type),
    }
}

fn make_optional(data: ClassData) -> ClassData {
    if matches!(data, ClassData::Option(_)) {
        data
    } else {
        ClassData::Option(Box::new(data))
    }
}

fn merge_class_data(old: ClassData, new: ClassData) -> ClassData {
    if let ClassData::Base(s) = &old {
        if s == "()" {
            return new;
        }
    }
    if let ClassData::Base(s) = &new {
        if s == "()" {
            return old;
        }
    }

    match (old, new) {
        (ClassData::Struct(old_hash), ClassData::Struct(new_hash)) => {
            if old_hash == new_hash {
                ClassData::Struct(old_hash)
            } else {
                let mut hashes = HashSet::new();
                hashes.insert(old_hash);
                hashes.insert(new_hash);
                ClassData::Enum(hashes)
            }
        }

        (ClassData::Enum(mut old_hashes), ClassData::Struct(new_hash)) => {
            old_hashes.insert(new_hash);
            ClassData::Enum(old_hashes)
        }

        (ClassData::Struct(old_hash), ClassData::Enum(mut new_hashes)) => {
            new_hashes.insert(old_hash);
            ClassData::Enum(new_hashes)
        }

        (ClassData::Enum(mut old_hashes), ClassData::Enum(new_hashes)) => {
            old_hashes.extend(new_hashes);
            ClassData::Enum(old_hashes)
        }

        (ClassData::List(old_inner), ClassData::List(new_inner)) => {
            ClassData::List(Box::new(merge_class_data(*old_inner, *new_inner)))
        }

        (ClassData::Map(old_key, old_val), ClassData::Map(new_key, new_val)) => {
            let merged_key = merge_class_data(*old_key, *new_key);
            let merged_val = merge_class_data(*old_val, *new_val);
            ClassData::Map(Box::new(merged_key), Box::new(merged_val))
        }

        (ClassData::Option(old_inner), ClassData::Option(new_inner)) => {
            ClassData::Option(Box::new(merge_class_data(*old_inner, *new_inner)))
        }

        (ClassData::Option(old_inner), new_data) => {
            ClassData::Option(Box::new(merge_class_data(*old_inner, new_data)))
        }

        (old_data, ClassData::Option(new_inner)) => {
            ClassData::Option(Box::new(merge_class_data(old_data, *new_inner)))
        }

        (_, new_data) => new_data,
    }
}

pub fn merge_class_maps(base: &mut ClassMap, new: ClassMap) {
    let mut new = new;

    for (class_hash, base_fields) in base.iter_mut() {
        if let Some(new_fields) = new.remove(class_hash) {
            let all_field_hashes: HashSet<u32> = base_fields
                .keys()
                .cloned()
                .chain(new_fields.keys().cloned())
                .collect();

            let mut new_fields = new_fields;

            for field_hash in all_field_hashes {
                let base_val_opt = base_fields.remove(&field_hash);
                let new_val_opt = new_fields.remove(&field_hash);

                let final_data = match (base_val_opt, new_val_opt) {
                    (Some(base_data), Some(new_data)) => merge_class_data(base_data, new_data),

                    (Some(base_data), None) => make_optional(base_data),

                    (None, Some(new_data)) => make_optional(new_data),

                    (None, None) => unreachable!(),
                };
                base_fields.insert(field_hash, final_data);
            }
        }
    }

    for (class_hash, new_fields) in new {
        base.insert(class_hash, new_fields);
    }
}

pub fn get_hashes(paths: &[&str]) -> HashMap<u32, String> {
    let mut hashes = HashMap::new();

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    if let Ok(hash) = u32::from_str_radix(parts[0], 16) {
                        hashes.insert(hash, parts[1].to_string());
                    }
                }
            }
        }
    }

    hashes
}

pub fn get_hashes_u64(paths: &[&str]) -> HashMap<u64, String> {
    let mut hashes = HashMap::new();

    for path in paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    if let Ok(hash) = u64::from_str_radix(parts[0], 16) {
                        hashes.insert(hash, parts[1].to_string());
                    }
                }
            }
        }
    }

    hashes
}

pub fn class_map_to_rust_code(
    class_map: &mut ClassMap,
    hashes: &HashMap<u32, String>,
    entry_hashes: &HashSet<u32>,
) -> Result<String, Error> {
    let mut all_definitions = String::new();

    // 1. 收集所有枚举信息
    let mut enum_infos = Vec::new();
    for (class_hash, class_fields) in class_map.iter() {
        let parent_class_name = hash_to_type_name(class_hash, hashes);

        for (field_hash, field_data) in class_fields.iter() {
            collect_enum_info_recursively(
                &parent_class_name,
                *field_hash,
                field_data,
                hashes,
                &mut enum_infos,
            );
        }
    }

    // 2. 合并有交集的枚举
    let merged_enums = merge_enums_with_intersection(enum_infos);

    // 3. 创建枚举映射表，基于枚举哈希值映射到合并后的枚举名称
    let mut enum_hash_to_name = HashMap::new();
    for enum_info in &merged_enums {
        for &hash in &enum_info.hashes {
            enum_hash_to_name.insert(hash, enum_info.name.clone());
        }
    }

    // 4. 新增：识别导致递归的字段并标记它们
    // 该集合存储需要Box的 (struct_hash, field_hash)
    let mut recursive_fields = HashSet::new();
    for enum_info in &merged_enums {
        for &variant_hash in &enum_info.hashes {
            // 对于作为结构体的每个变体...
            if let Some(struct_fields) = class_map.get(&variant_hash) {
                // ...检查它的字段
                for (field_hash, field_data) in struct_fields {
                    // 检查此字段的类型是否包含对该枚举的引用
                    if contains_enum_reference(
                        field_data,
                        &enum_info.hashes, // 枚举中所有变体的集合
                        class_map,
                    ) {
                        // 标记此字段以便Box处理
                        recursive_fields.insert((variant_hash, *field_hash));
                    }
                }
            }
        }
    }

    // 5. 生成枚举定义 (简化版，无Box)
    let enum_definitions = generate_enum_definitions(&merged_enums, class_map, hashes);
    all_definitions.push_str(&enum_definitions);

    // 6. 生成结构体定义
    for (class_hash, class_data) in class_map.iter() {
        let mut class_name = hash_to_type_name(class_hash, hashes);

        if class_name == "Self" {
            class_name = "r#Self".to_string();
        }

        let mut struct_def = String::new();
        if entry_hashes.contains(class_hash) {
            struct_def.push_str("#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]\n");
        } else {
            struct_def.push_str("#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]\n");
        }
        struct_def.push_str("#[serde(rename_all = \"camelCase\")]\n");
        struct_def.push_str(&format!("pub struct {} {{\n", class_name));

        for (field_hash, field_data) in class_data.iter() {
            let field_name_original = hashes
                .get(field_hash)
                .cloned()
                .unwrap_or(format!("unk_0x{:x}", field_hash));

            let mut field_name_snake = field_name_original.to_snake_case();

            if field_name_snake == "type" {
                field_name_snake = "r#type".to_string();
            }
            if field_name_snake == "move" {
                field_name_snake = "r#move".to_string();
            }
            if field_name_snake == "loop" {
                field_name_snake = "r#loop".to_string();
            }
            if field_name_snake == "trait" {
                field_name_snake = "r#trait".to_string();
            }

            let mut type_name = map_class_data_to_rust_type(
                &class_name,
                field_data,
                field_hash,
                hashes,
                &enum_hash_to_name,
            );

            // 新增：如果字段被标记为递归，则应用Box<>
            if recursive_fields.contains(&(*class_hash, *field_hash)) {
                type_name = format!("Box<{}>", type_name);
            }

            struct_def.push_str(&format!("    pub {}: {},\n", field_name_snake, type_name));
        }
        struct_def.push_str("}\n\n");
        all_definitions.push_str(&struct_def);
    }

    Ok(all_definitions)
}

pub fn extract_all_class(prop_file: &PropFile) -> Result<ClassMap, Error> {
    let mut class_map = HashMap::new();
    for (class_hash, entry) in prop_file.iter_class_hash_and_entry() {
        let class_map_entry = extract_entry_class(class_hash, entry).unwrap();
        merge_class_maps(&mut class_map, class_map_entry);
    }
    Ok(class_map)
}

pub fn extract_entry_class(class_hash: u32, entry: &EntryData) -> Result<ClassMap, Error> {
    let mut parser = BinParser::from_bytes(&entry.data);

    let data_map = parser.read_fields().unwrap();

    let class_map = extract_struct_class(class_hash, &data_map).unwrap();

    Ok(class_map)
}

pub fn extract_struct_class(
    class_hash: u32,
    data_map: &HashMap<u32, (BinType, &[u8])>,
) -> Result<ClassMap, Error> {
    let mut class_map = HashMap::new();
    let mut struct_map = HashMap::new();

    for (hash, (vtype, value_slice)) in data_map.iter() {
        let class_data = extract_type_data(*vtype, value_slice, &mut class_map).unwrap();
        struct_map.insert(*hash, class_data);
    }

    class_map.insert(class_hash, struct_map);

    Ok(class_map)
}

fn extract_type_data(
    vtype: BinType,
    value_slice: &[u8],
    class_map: &mut ClassMap,
) -> Result<ClassData, Error> {
    if !matches!(
        vtype,
        BinType::Struct
            | BinType::Embed
            | BinType::List
            | BinType::List2
            | BinType::Map
            | BinType::Option
    ) {
        return Ok(ClassData::Base(map_base_type(&vtype)));
    }

    let mut parser = BinParser::from_bytes(value_slice);
    match vtype {
        BinType::Struct | BinType::Embed => {
            let Some(header) = parser.read_struct_header()? else {
                return Ok(ClassData::Struct(0));
            };
            let fields = parser.read_fields().unwrap();

            let sub_class_map = extract_struct_class(header.class_hash, &fields).unwrap();
            merge_class_maps(class_map, sub_class_map);
            Ok(ClassData::Struct(header.class_hash))
        }
        BinType::List | BinType::List2 => {
            let element_type = parser.read_type().unwrap();
            let _bytes_count = parser.read_u32().unwrap();
            let list = parser.read_list(element_type).unwrap();

            if element_type == BinType::Struct || element_type == BinType::Embed {
                let mut class_hashes = HashSet::new();
                for struct_data in list {
                    let mut item_parser = BinParser::from_bytes(struct_data);
                    let Some(header) = item_parser.read_struct_header()? else {
                        class_hashes.insert(0);
                        continue;
                    };
                    class_hashes.insert(header.class_hash);

                    let item_fields = item_parser.read_fields().unwrap();
                    let item_class_map =
                        extract_struct_class(header.class_hash, &item_fields).unwrap();
                    merge_class_maps(class_map, item_class_map);
                }
                if class_hashes.len() == 1 {
                    Ok(ClassData::List(Box::new(ClassData::Struct(
                        *class_hashes.iter().next().unwrap(),
                    ))))
                } else {
                    Ok(ClassData::List(Box::new(ClassData::Enum(class_hashes))))
                }
            } else {
                let element_class_data = if let Some(first_element) = list.get(0) {
                    extract_type_data(element_type, first_element, class_map)?
                } else {
                    ClassData::Base("()".to_string())
                };
                Ok(ClassData::List(Box::new(element_class_data)))
            }
        }
        BinType::Map => {
            let ktype = parser.read_type().unwrap();
            let vtype = parser.read_type().unwrap();

            let _bytes_count = parser.read_u32().unwrap();
            let count = parser.read_u32().unwrap();

            if vtype == BinType::Struct || vtype == BinType::Embed {
                let mut class_hashes = HashSet::new();
                for _ in 0..count {
                    parser.skip_value(ktype).unwrap();
                    let val_slice = parser.skip_value(vtype).unwrap();
                    let mut parser = BinParser::from_bytes(val_slice);
                    let Some(header) = parser.read_struct_header()? else {
                        class_hashes.insert(0);
                        continue;
                    };
                    class_hashes.insert(header.class_hash);

                    let item_fields = parser.read_fields().unwrap();
                    let item_class_map =
                        extract_struct_class(header.class_hash, &item_fields).unwrap();
                    merge_class_maps(class_map, item_class_map);
                }
                if class_hashes.len() == 1 {
                    Ok(ClassData::Map(
                        Box::new(ClassData::Base(map_base_type(&ktype))),
                        Box::new(ClassData::Struct(*class_hashes.iter().next().unwrap())),
                    ))
                } else {
                    Ok(ClassData::Map(
                        Box::new(ClassData::Base(map_base_type(&ktype))),
                        Box::new(ClassData::Enum(class_hashes)),
                    ))
                }
            } else {
                Ok(ClassData::Map(
                    Box::new(ClassData::Base(map_base_type(&ktype))),
                    Box::new(ClassData::Base(map_base_type(&vtype))),
                ))
            }
        }
        BinType::Option => {
            let vtype = parser.read_type().unwrap();
            let some = parser.read_bool().unwrap();
            if some {
                let value_slice = parser.skip_value(vtype).unwrap();
                let value_data = extract_type_data(vtype, value_slice, class_map).unwrap();
                Ok(ClassData::Option(Box::new(value_data)))
            } else {
                Ok(ClassData::Option(Box::new(ClassData::Base(
                    "()".to_string(),
                ))))
            }
        }

        _ => unreachable!(),
    }
}
