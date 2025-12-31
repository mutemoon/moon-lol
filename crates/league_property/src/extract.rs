use std::collections::{HashMap, HashSet};

use heck::ToPascalCase;
use league_utils::{hash_bin, hash_to_field_name, hash_to_type_name};

use crate::{detect_cyclic_types, BinParser, BinType, EntryData, Error};

#[derive(Debug, Clone)]
pub enum ClassData {
    Base(String),
    Struct(u32),
    Enum(HashSet<u32>),
    List(Box<ClassData>),
    Map(Box<ClassData>, Box<ClassData>),
    Option(Box<ClassData>),
}

pub type ClassMap = HashMap<u32, HashMap<u32, ClassData>>;

#[derive(Debug)]
pub struct EnumInfo {
    pub name: String,
    pub variants: HashSet<u32>,
}

struct GeneratedItem {
    name: String,
    code: String,
}

pub fn class_map_to_rust_code(
    class_map: &mut ClassMap,
    hashes: &HashMap<u32, String>,
    entry_hashes: &HashSet<u32>,
) -> Result<(String, String), Error> {
    // 1. 收集所有枚举信息
    let mut enums = Vec::new();
    for (_, class_fields) in class_map.iter() {
        for (_, field_data) in class_fields.iter() {
            collect_enums(field_data, &mut enums);
        }
    }

    // 2. 合并有交集的枚举
    let merged_enums = merge_enums(enums);

    let mut merged_enum_map: HashMap<u32, EnumInfo> = HashMap::new();
    for variants in merged_enums {
        let (hash, enum_info) = generate_enum_info(variants, hashes);
        if merged_enum_map.contains_key(&hash) {
            merged_enum_map
                .get_mut(&hash)
                .unwrap()
                .variants
                .extend(enum_info.variants.clone());

            println!(
                "--------- {:?} already exists\n{:?}\n{:?}",
                enum_info.name,
                enum_info
                    .variants
                    .iter()
                    .map(|v| hash_to_type_name(v, hashes))
                    .collect::<Vec<_>>(),
                merged_enum_map
                    .get(&hash)
                    .unwrap()
                    .variants
                    .iter()
                    .map(|v| hash_to_type_name(v, hashes))
                    .collect::<Vec<_>>()
            );
            continue;
        }
        merged_enum_map.insert(hash, enum_info);
    }
    let merged_enums = merged_enum_map;

    // 3. 检测循环类型
    let cyclic_types = detect_cyclic_types(class_map, &merged_enums);

    // 用于收集所有的定义，最后统一排序
    let mut all_generated_items: Vec<GeneratedItem> = Vec::new();

    // 4. 生成枚举定义
    let generated_enum_items = generate_enum_definitions(&merged_enums, class_map, hashes);
    all_generated_items.extend(generated_enum_items);

    let mut init_code = "pub fn init_league_asset(app: &mut App, asset_loader_registry: &mut AssetLoaderRegistry) {".to_string();

    // 6. 生成结构体定义
    for (class_hash, class_data) in class_map.iter() {
        let class_name = hash_to_type_name(class_hash, hashes);

        let mut struct_def = String::new();
        if entry_hashes.contains(class_hash) {
            struct_def
                .push_str("#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]\n");
            init_code.push_str(&format!(
                "app.init_asset::<{}>();\nasset_loader_registry.register::<{}>();\n",
                class_name, class_name
            ));
        } else {
            struct_def.push_str("#[derive(Serialize, Deserialize, Debug, Clone)]\n");
        }
        struct_def.push_str("#[serde(rename_all = \"camelCase\")]\n");
        struct_def.push_str(&format!("pub struct {} {{\n", class_name));

        // 对字段按名称排序
        let mut sorted_fields: Vec<_> = class_data.iter().collect();
        sorted_fields.sort_by(|(h1, _), (h2, _)| {
            let name1 = hash_to_field_name(*h1, hashes);
            let name2 = hash_to_field_name(*h2, hashes);
            name1.cmp(&name2)
        });

        for (field_hash, field_data) in sorted_fields {
            let field_name = hash_to_field_name(field_hash, hashes);

            let mut current_cyclic_types = Vec::new();
            for scc in &cyclic_types {
                if scc.contains(&class_hash) {
                    current_cyclic_types.push(scc.clone());
                }
            }

            let type_name = field_type_to_string(
                class_map,
                field_data,
                hashes,
                &merged_enums,
                &current_cyclic_types,
            );

            struct_def.push_str(&format!("    pub {}: {},\n", field_name, type_name));
        }
        struct_def.push_str("}\n\n");

        all_generated_items.push(GeneratedItem {
            name: class_name,
            code: struct_def,
        });
    }

    // 7. 对所有生成的项（Enum 和 Struct）按名称排序
    all_generated_items.sort_by(|a, b| a.name.cmp(&b.name));

    // 8. 拼接结果
    let mut all_definitions = String::new();
    for item in all_generated_items {
        all_definitions.push_str(&item.code);
    }

    init_code.push_str("}\n");

    Ok((all_definitions, init_code))
}

fn collect_enums(field_data: &ClassData, enums: &mut Vec<HashSet<u32>>) {
    match field_data {
        ClassData::Enum(enum_hashes) => {
            enums.push(enum_hashes.clone());
        }
        ClassData::List(element_data) => {
            collect_enums(element_data, enums);
        }
        ClassData::Map(key_data, value_data) => {
            collect_enums(key_data, enums);
            collect_enums(value_data, enums);
        }
        ClassData::Option(value_data) => {
            collect_enums(value_data, enums);
        }
        _ => {}
    }
}

fn merge_enums(enums: Vec<HashSet<u32>>) -> Vec<HashSet<u32>> {
    let mut merged_groups: Vec<HashSet<u32>> = Vec::new();

    for new_set in enums {
        // 1. 找出所有与当前 new_set 有交集的组的索引
        let mut connected_indices: Vec<usize> = Vec::new();
        for (idx, group) in merged_groups.iter().enumerate() {
            if !group.is_disjoint(&new_set) {
                connected_indices.push(idx);
            }
        }

        if connected_indices.is_empty() {
            // 没有交集，作为新组添加
            merged_groups.push(new_set);
            continue;
        }

        // 2. 有交集，需要将 new_set 和所有相关的组融合成一个
        // 按索引从大到小排序，以便从后往前移除，防止索引失效
        connected_indices.sort_by(|a, b| b.cmp(a));

        // 取出这些索引中最小的一个作为“目标宿主”
        let target_idx = connected_indices.pop().unwrap();

        // 先把 new_set 融合进宿主
        merged_groups[target_idx].extend(new_set);

        // 把其他有交集的组全部吸收到 target_idx 中，并从列表中移除
        for remove_idx in connected_indices {
            let removed_group = merged_groups.remove(remove_idx);
            merged_groups[target_idx].extend(removed_group);
        }
    }

    // 转换结果为 EnumInfo
    merged_groups
}

// 提取出来的辅助函数，用于生成 EnumInfo，减少主函数缩进
fn generate_enum_info(
    variants_set: HashSet<u32>,
    hashes: &HashMap<u32, String>,
) -> (u32, EnumInfo) {
    let mut variant_names: Vec<String> = variants_set
        .iter()
        .filter_map(|v| hashes.get(v).map(|s| s.to_pascal_case()))
        .collect();

    let name = if variant_names.is_empty() {
        // 如果没有名字，使用哈希值的十六进制表示兜底
        let mut unnamed: Vec<String> = variants_set
            .iter()
            .map(|v| hash_to_type_name(v, hashes))
            .collect();
        unnamed.sort();
        unnamed.first().cloned().unwrap()
    } else {
        variant_names.sort();
        // 尝试寻找公共子串作为枚举名
        if let Some(name) = find_longest_common_capitalized_substring(variant_names.clone()) {
            if name.len() < 3 {
                variant_names.first().unwrap().clone()
            } else {
                name
            }
        } else {
            variant_names.first().unwrap().clone()
        }
    };

    let name = format!("Enum{}", name);

    let enum_hash = hash_bin(&name);
    (
        enum_hash,
        EnumInfo {
            name,
            variants: variants_set,
        },
    )
}

pub fn find_longest_common_capitalized_substring(list: Vec<String>) -> Option<String> {
    let shortest = list.iter().min_by_key(|s| s.chars().count())?;
    let chars: Vec<char> = shortest.chars().collect();
    let max_len = chars.len();

    for len in (1..=max_len).rev() {
        let result = check_substrings_of_len(&list, &chars, len);
        if result.is_some() {
            return result;
        }
    }

    None
}

fn check_substrings_of_len(list: &[String], chars: &[char], len: usize) -> Option<String> {
    for window in chars.windows(len) {
        if !is_start_uppercase(window) {
            continue;
        }

        let candidate: String = window.iter().collect();
        let all_contain = list.iter().all(|s| s.contains(&candidate));

        if all_contain {
            return Some(candidate);
        }
    }
    None
}

fn is_start_uppercase(window: &[char]) -> bool {
    if let Some(first_char) = window.first() {
        return first_char.is_uppercase();
    }
    false
}

pub fn collect_dependencies(
    class_data: &ClassData,
    merged_enums: &HashMap<u32, EnumInfo>,
    deps: &mut HashSet<u32>,
) {
    match class_data {
        ClassData::Struct(h) => {
            deps.insert(*h);
        }
        ClassData::Enum(hashes) => {
            deps.insert(
                merged_enums
                    .iter()
                    .find_map(|(hash, info)| {
                        if hashes.is_subset(&info.variants) {
                            Some(*hash)
                        } else {
                            None
                        }
                    })
                    .unwrap(),
            );
        }
        ClassData::List(inner) | ClassData::Option(inner) => {
            collect_dependencies(inner, merged_enums, deps);
        }
        ClassData::Map(k, v) => {
            collect_dependencies(k, merged_enums, deps);
            collect_dependencies(v, merged_enums, deps);
        }
        _ => {}
    }
}

fn generate_enum_definitions(
    merged_enums: &HashMap<u32, EnumInfo>,
    class_map: &mut ClassMap,
    hashes: &HashMap<u32, String>,
) -> Vec<GeneratedItem> {
    let mut generated_items = Vec::new();

    for (_, info) in merged_enums {
        let mut enum_def = String::new();
        enum_def.push_str("#[derive(Serialize, Deserialize, Debug, Clone)]\n");

        enum_def.push_str(&format!("pub enum {} {{\n", info.name));

        let mut variant_map = info
            .variants
            .iter()
            .map(|v| (*v, hash_to_type_name(v, hashes)))
            .collect::<Vec<(u32, String)>>();

        variant_map.sort_by_key(|v| v.1.clone());

        for (variant_hash, variant_name) in variant_map {
            let is_empty_struct = class_map
                .get(&variant_hash)
                .map_or(true, |fields| fields.is_empty());

            if is_empty_struct {
                enum_def.push_str(&format!("    {},\n", variant_name));
                class_map.remove(&variant_hash);
            } else {
                enum_def.push_str(&format!("    {}({}),\n", variant_name, variant_name));
            }
        }

        enum_def.push_str("}\n\n");
        generated_items.push(GeneratedItem {
            name: info.name.clone(),
            code: enum_def,
        });
    }

    generated_items
}

fn field_type_to_string(
    class_map: &ClassMap,
    field_data: &ClassData,
    hashes: &HashMap<u32, String>,
    merged_enums: &HashMap<u32, EnumInfo>,
    cyclic_types: &Vec<HashSet<u32>>,
) -> String {
    match field_data {
        ClassData::Base(type_string) => type_string.clone(),
        ClassData::Struct(struct_hash) => {
            let (hash, name) = if class_map.get(struct_hash).is_some() {
                (struct_hash, hash_to_type_name(struct_hash, hashes))
            } else {
                let enum_info = merged_enums
                    .iter()
                    .find(|v| v.1.variants.contains(struct_hash))
                    .unwrap();

                (enum_info.0, enum_info.1.name.clone())
            };

            if cyclic_types.iter().any(|scc| scc.contains(hash)) {
                format!("Box<{}>", name)
            } else {
                name
            }
        }
        ClassData::Enum(enum_hashes) => {
            let (hash, enum_name) = merged_enums
                .iter()
                .find_map(|(hash, info)| {
                    if enum_hashes.is_subset(&info.variants) {
                        Some((hash, info.name.clone()))
                    } else {
                        None
                    }
                })
                .unwrap();

            if cyclic_types.iter().any(|scc| scc.contains(hash)) {
                format!("Box<{}>", enum_name)
            } else {
                enum_name
            }
        }
        ClassData::List(element_data) => {
            format!(
                "Vec<{}>",
                field_type_to_string(class_map, element_data, hashes, merged_enums, cyclic_types)
            )
        }
        ClassData::Map(key_data, value_data) => {
            let key = field_type_to_string(class_map, key_data, hashes, merged_enums, cyclic_types);
            let value =
                field_type_to_string(class_map, value_data, hashes, merged_enums, cyclic_types);
            let key = if key == "f32" { "u32".to_string() } else { key };
            format!("HashMap<{}, {}>", key, value)
        }
        ClassData::Option(value_data) => {
            format!(
                "Option<{}>",
                field_type_to_string(class_map, value_data, hashes, merged_enums, cyclic_types)
            )
        }
    }
}

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

fn extract_type_data(
    vtype: BinType,
    value_slice: &[u8],
    class_map: &mut ClassMap,
) -> Result<ClassData, Error> {
    let mut parser = BinParser::from_bytes(value_slice);
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

    match vtype {
        BinType::Struct | BinType::Embed => {
            let Some(header) = parser.read_struct_header()? else {
                return Ok(ClassData::Base("()".to_string()));
            };
            let fields = parser.read_fields()?;

            let sub_class_map = extract_struct_class(header.class_hash, &fields).unwrap();
            merge_class_maps(class_map, sub_class_map);
            Ok(ClassData::Struct(header.class_hash))
        }
        BinType::List | BinType::List2 => {
            let element_type = parser.read_type()?;
            let _bytes_count = parser.read_u32()?;
            let list = parser.read_list(element_type)?;

            if element_type == BinType::Struct || element_type == BinType::Embed {
                let mut class_hashes = HashSet::new();
                for struct_data in list {
                    let mut item_parser = BinParser::from_bytes(struct_data);
                    let Some(header) = item_parser.read_struct_header()? else {
                        continue;
                    };

                    class_hashes.insert(header.class_hash);

                    let item_fields = item_parser.read_fields()?;
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
                        continue;
                    };

                    class_hashes.insert(header.class_hash);

                    let item_fields = parser.read_fields()?;
                    let item_class_map = extract_struct_class(header.class_hash, &item_fields)?;
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
