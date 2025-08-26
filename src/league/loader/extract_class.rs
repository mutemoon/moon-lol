use std::collections::{HashMap, HashSet};

use heck::{ToPascalCase, ToSnakeCase};

use crate::league::{BinParser, BinType, EntryData, LeagueLoaderError, PropFile};

#[derive(Debug, Clone)]
pub enum ClassData {
    Base(String),
    Struct(u32),
    Enum(HashSet<u32>),
    List(Box<ClassData>),
    Map(Box<ClassData>, Box<ClassData>),
    Option(Box<ClassData>),
}

type ClassMap = HashMap<u32, HashMap<u32, ClassData>>;

// 辅助函数，增加了 parent_class_name 和 class_map 参数
fn generate_enums_recursively(
    parent_class_name: &str,
    field_hash: &u32,
    field_data: &ClassData,
    class_map: &mut ClassMap, // <--- 新增参数: 传入完整的 class_map
    hashes: &HashMap<u32, String>,
    generated_enums: &mut HashSet<String>,
    all_definitions: &mut String,
) {
    match field_data {
        ClassData::Enum(enum_hashes) => {
            let field_name = hashes
                .get(field_hash)
                .map(|s| s.as_str())
                .unwrap_or("UnknownEnumField");

            let enum_name = format!("{}{}", parent_class_name, field_name.to_pascal_case());

            if generated_enums.contains(&enum_name) {
                return;
            }

            let mut enum_def = String::new();
            enum_def.push_str("#[derive(Serialize, Deserialize, Debug)]\n");
            enum_def.push_str(&format!("pub enum {} {{\n", enum_name));

            for variant_hash in enum_hashes {
                let mut variant_name = hashes
                    .get(variant_hash)
                    .map(|s| s.to_pascal_case())
                    .unwrap_or(format!("Unk0x{:x}", variant_hash));

                if variant_name == "Self" {
                    variant_name = "MySelf".to_string();
                }

                // --- 这是核心修改 ---
                // 检查这个变体对应的 Struct 是否为空。
                // map_or(false, ...) 的意思是：如果 get 返回 None (struct 不存在)，则视为空；
                // 如果返回 Some(fields)，则检查 fields.is_empty()。
                let is_empty_struct = class_map
                    .get(variant_hash)
                    .map_or(true, |fields| fields.is_empty());

                if is_empty_struct {
                    // 如果 struct 为空，则生成简单变体，例如: MyVariant,
                    enum_def.push_str(&format!("    {},\n", variant_name));
                    class_map.remove(variant_hash);
                } else {
                    // 否则，保持原样，生成 newtype 变体，例如: MyVariant(MyVariant),
                    enum_def.push_str(&format!("    {}({}),\n", variant_name, variant_name));
                }
            }

            enum_def.push_str("}\n\n");
            all_definitions.push_str(&enum_def);
            generated_enums.insert(enum_name);
        }
        // --- 递归处理 (将 class_map 参数传递下去) ---
        ClassData::List(element_data) => {
            generate_enums_recursively(
                parent_class_name,
                field_hash,
                element_data,
                class_map, // <--- 传递
                hashes,
                generated_enums,
                all_definitions,
            );
        }
        ClassData::Map(key_data, value_data) => {
            generate_enums_recursively(
                parent_class_name,
                field_hash,
                key_data,
                class_map, // <--- 传递
                hashes,
                generated_enums,
                all_definitions,
            );
            generate_enums_recursively(
                parent_class_name,
                field_hash,
                value_data,
                class_map, // <--- 传递
                hashes,
                generated_enums,
                all_definitions,
            );
        }
        ClassData::Option(value_data) => {
            generate_enums_recursively(
                parent_class_name,
                field_hash,
                value_data,
                class_map, // <--- 传递
                hashes,
                generated_enums,
                all_definitions,
            );
        }
        ClassData::Base(_) | ClassData::Struct(_) => {}
    }
}

// 同样增加了 parent_class_name 参数
fn map_class_data_to_rust_type(
    parent_class_name: &str, // 新增参数
    field_data: &ClassData,
    field_hash: &u32,
    hashes: &HashMap<u32, String>,
) -> String {
    match field_data {
        ClassData::Base(type_string) => type_string.clone(),
        ClassData::Struct(struct_hash) => hashes
            .get(struct_hash)
            .map(|s| s.to_pascal_case())
            .unwrap_or(format!("Unk0x{:x}", struct_hash)),
        ClassData::Enum(_) => {
            // -- 这是关键修改 --
            // 同样使用父结构体名和字段名来构造类型名
            let field_name = hashes
                .get(field_hash)
                .map(|s| s.to_pascal_case())
                .unwrap_or(format!("UnkEnum{:x}", field_hash));
            format!("{}{}", parent_class_name, field_name)
        }
        ClassData::List(element_data) => {
            // 递归调用时传递 parent_class_name
            format!(
                "Vec<{}>",
                map_class_data_to_rust_type(parent_class_name, element_data, field_hash, hashes)
            )
        }
        ClassData::Map(key_data, value_data) => {
            // 递归调用时传递 parent_class_name
            format!(
                "HashMap<{}, {}>",
                map_class_data_to_rust_type(parent_class_name, key_data, field_hash, hashes),
                map_class_data_to_rust_type(parent_class_name, value_data, field_hash, hashes)
            )
        }
        ClassData::Option(value_data) => {
            // 递归调用时传递 parent_class_name
            format!(
                "Option<{}>",
                map_class_data_to_rust_type(parent_class_name, value_data, field_hash, hashes)
            )
        }
    }
}

/// 辅助函数：将 BinType 映射为 Rust 类型字符串
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
        BinType::Vec2 => "Vec2".to_string(), // 假设这些是已定义的类型
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

/// 辅助函数：将 ClassData 包装为可选类型，同时避免嵌套 Option。
fn make_optional(data: ClassData) -> ClassData {
    // 如果已经是 Option 类型，直接返回，防止 Option<Option<T>>
    if matches!(data, ClassData::Option(_)) {
        data
    } else {
        ClassData::Option(Box::new(data))
    }
}

fn merge_class_data(old: ClassData, new: ClassData) -> ClassData {
    // 规则：一个具体的类型总是比一个空的占位符类型 "()" 更优。
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
        // --- 核心修改：处理 Struct 和 Enum 的合并 ---

        // 情况1: 两个都是 Struct。如果类型不同，则提升为 Enum。
        (ClassData::Struct(old_hash), ClassData::Struct(new_hash)) => {
            if old_hash == new_hash {
                ClassData::Struct(old_hash) // 类型相同，无需改变
            } else {
                // 类型不同，创建一个包含两种类型的 Enum
                let mut hashes = HashSet::new();
                hashes.insert(old_hash);
                hashes.insert(new_hash);
                ClassData::Enum(hashes)
            }
        }

        // 情况2: 旧的是 Enum，新的是 Struct。将 Struct 添加到 Enum 中。
        (ClassData::Enum(mut old_hashes), ClassData::Struct(new_hash)) => {
            old_hashes.insert(new_hash);
            ClassData::Enum(old_hashes)
        }

        // 情况3: 旧的是 Struct，新的是 Enum。将 Struct 添加到 Enum 中。
        (ClassData::Struct(old_hash), ClassData::Enum(mut new_hashes)) => {
            new_hashes.insert(old_hash);
            ClassData::Enum(new_hashes)
        }

        // --- 保留原有的合并逻辑 ---

        // 如果两个都是 Enum，合并它们的哈希集合（取并集）。
        (ClassData::Enum(mut old_hashes), ClassData::Enum(new_hashes)) => {
            old_hashes.extend(new_hashes);
            ClassData::Enum(old_hashes)
        }

        // 如果两个都是 List，递归合并它们的元素类型。
        (ClassData::List(old_inner), ClassData::List(new_inner)) => {
            ClassData::List(Box::new(merge_class_data(*old_inner, *new_inner)))
        }

        // 如果两个都是 Map，递归合并它们的键和值类型。
        (ClassData::Map(old_key, old_val), ClassData::Map(new_key, new_val)) => {
            let merged_key = merge_class_data(*old_key, *new_key);
            let merged_val = merge_class_data(*old_val, *new_val);
            ClassData::Map(Box::new(merged_key), Box::new(merged_val))
        }

        // 如果两个都是 Option，递归合并它们内部的类型。
        (ClassData::Option(old_inner), ClassData::Option(new_inner)) => {
            ClassData::Option(Box::new(merge_class_data(*old_inner, *new_inner)))
        }

        // --- 新增逻辑：处理 Option<T> 和 T 的合并 ---
        // 规则：只要一方是 Option，结果就必须是 Option。

        // 情况1: 旧的是 Option，新的是具体类型
        (ClassData::Option(old_inner), new_data) => {
            // 合并内部类型，然后用 Option 包装结果
            ClassData::Option(Box::new(merge_class_data(*old_inner, new_data)))
        }

        // 情况2: 旧的是具体类型，新的是 Option
        (old_data, ClassData::Option(new_inner)) => {
            // 合并内部类型，然后用 Option 包装结果
            ClassData::Option(Box::new(merge_class_data(old_data, *new_inner)))
        }

        // 如果类型不匹配（且不是上面处理的 Struct/Enum 组合），则遵循“后者覆盖前者”的原则。
        (_, new_data) => new_data,
    }
}

/// 将一个新的 ClassMap 深度合并到一个基础 ClassMap 中。
///
/// 新的合并逻辑:
/// 1. 对于一个 Struct，合并其在新旧 map 中所有出现过的字段。
/// 2. 如果一个字段在两边都存在，则递归合并其类型 (使用 merge_class_data)。
/// 3. 如果一个字段只在其中一边存在，那么它的类型必须被更新为 Option<T>。
pub fn merge_class_maps(base: &mut ClassMap, new: ClassMap) {
    // 为了能够高效地移出 new map 中的值，我们先将其变为可变
    let mut new = new;

    // 遍历所有在 base 中已经存在的 class_hash
    for (class_hash, base_fields) in base.iter_mut() {
        if *class_hash == 0xf36e13f7 {
            // println!("base_fields: {:?}", base_fields);
        }

        // 如果这个 class 在 new map 中也存在
        if let Some(new_fields) = new.remove(class_hash) {
            if *class_hash == 0xf36e13f7 {
                // println!("new_fields: {:?}", new_fields);
            }

            // 获取新旧字段的 key 的并集，这样我们就能处理所有字段
            let all_field_hashes: HashSet<u32> = base_fields
                .keys()
                .cloned()
                .chain(new_fields.keys().cloned())
                .collect();

            // 为了高效处理，将 new_fields 变为可变
            let mut new_fields = new_fields;

            for field_hash in all_field_hashes {
                let base_val_opt = base_fields.remove(&field_hash);
                let new_val_opt = new_fields.remove(&field_hash);

                let final_data = match (base_val_opt, new_val_opt) {
                    // 情况1: 字段在两边都存在，正常合并类型
                    (Some(base_data), Some(new_data)) => merge_class_data(base_data, new_data),
                    // 情况2: 字段只在 base 中存在，说明在新数据里它是缺失的，所以必须是可选的
                    (Some(base_data), None) => make_optional(base_data),
                    // 情况3: 字段只在 new 中存在，说明在之前的数据里它是缺失的，所以也必须是可选的
                    (None, Some(new_data)) => make_optional(new_data),
                    // 不可能出现的情况
                    (None, None) => unreachable!(),
                };
                base_fields.insert(field_hash, final_data);
            }
        }
    }

    // 处理那些只在 new map 中存在的 class
    // 对于这些全新的 class，其所有字段都来自于同一个来源，所以暂时不需要变为 Option
    // （除非在未来的合并中发现它们是可选的）
    for (class_hash, new_fields) in new {
        base.insert(class_hash, new_fields);
    }
}

pub fn get_hashes(paths: &[&str]) -> HashMap<u32, String> {
    let mut hashes = HashMap::new();

    for path in paths {
        let content = std::fs::read_to_string(path).unwrap();
        let lines = content.lines().collect::<Vec<_>>();

        for line in lines {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() != 2 {
                continue;
            }

            let hash = u32::from_str_radix(parts[0], 16).unwrap();
            let path = parts[1].to_string();
            hashes.insert(hash, path);
        }
    }

    hashes
}

pub fn get_hashes_u64(paths: &[&str]) -> HashMap<u64, String> {
    let mut hashes = HashMap::new();

    for path in paths {
        let content = std::fs::read_to_string(path).unwrap();
        let lines = content.lines().collect::<Vec<_>>();

        for line in lines {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() != 2 {
                continue;
            }

            let hash = u64::from_str_radix(parts[0], 16).unwrap();
            let path = parts[1].to_string();
            hashes.insert(hash, path);
        }
    }

    hashes
}

pub async fn class_map_to_rust_code(
    class_map: &mut ClassMap,
    hashes: &HashMap<u32, String>,
) -> Result<String, LeagueLoaderError> {
    let mut all_definitions = String::new();
    let mut generated_enums = HashSet::new();

    // --- 第一阶段: 生成所有需要的 Enum 定义 ---
    let mut enums_info = Vec::new();
    for (class_hash, class_fields) in class_map.iter() {
        let parent_class_name = hashes
            .get(class_hash)
            .map(|s| s.to_pascal_case())
            .unwrap_or(format!("Unk0x{:x}", class_hash));

        for (field_hash, field_data) in class_fields.iter() {
            enums_info.push((parent_class_name.clone(), *field_hash, field_data.clone()));
        }
    }

    for (parent_class_name, field_hash, field_data) in enums_info {
        generate_enums_recursively(
            &parent_class_name,
            &field_hash,
            &field_data,
            class_map, // <--- 修改点: 将 class_map 自身传入
            hashes,
            &mut generated_enums,
            &mut all_definitions,
        );
    }

    // --- 第二阶段: 生成 Struct 定义 ---
    for (class_hash, class_data) in class_map.iter() {
        let mut class_name = hashes
            .get(class_hash)
            .map(|s| s.to_pascal_case())
            .unwrap_or(format!("Unk0x{:x}", class_hash));

        if class_name == "Self" {
            class_name = "r#Self".to_string();
        }

        let mut struct_def = String::new();
        struct_def.push_str("#[derive(Serialize, Deserialize, Debug)]\n");
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

            let type_name =
                map_class_data_to_rust_type(&class_name, field_data, field_hash, hashes);

            struct_def.push_str(&format!("    pub {}: {},\n", field_name_snake, type_name));
        }
        struct_def.push_str("}\n\n");
        all_definitions.push_str(&struct_def);
    }

    Ok(all_definitions)
}

pub async fn extract_all_class(prop_file: &PropFile) -> Result<ClassMap, LeagueLoaderError> {
    let mut class_map = HashMap::new();
    for (class_hash, entry) in prop_file.iter_class_hash_and_entry() {
        let class_map_entry = extract_entry_class(class_hash, entry).await.unwrap();
        merge_class_maps(&mut class_map, class_map_entry);
    }
    Ok(class_map)
}

pub async fn extract_entry_class(
    class_hash: u32,
    entry: &EntryData,
) -> Result<ClassMap, LeagueLoaderError> {
    let mut parser = BinParser::from_bytes(&entry.data);

    let data_map = parser.read_fields().unwrap();

    let class_map = extract_struct_class(class_hash, &data_map).unwrap();

    Ok(class_map)
}

// 3. extract_struct_class 被大大简化
pub fn extract_struct_class(
    class_hash: u32,
    data_map: &HashMap<u32, (BinType, &[u8])>,
) -> Result<ClassMap, LeagueLoaderError> {
    let mut class_map = HashMap::new();
    let mut struct_map = HashMap::new();

    for (hash, (vtype, value_slice)) in data_map.iter() {
        // 对每个字段，调用新的核心函数来提取其完整的类型信息
        let class_data = extract_type_data(*vtype, value_slice, &mut class_map).unwrap();
        struct_map.insert(*hash, class_data);
    }

    class_map.insert(class_hash, struct_map);

    Ok(class_map)
}

// 4. 新的核心递归函数
/// 根据 vtype 和数据切片，递归地提取完整的 ClassData 信息
/// 同时会用新发现的子结构定义来填充 class_map
fn extract_type_data(
    vtype: BinType,
    value_slice: &[u8],
    class_map: &mut ClassMap, // 传入可变引用以收集嵌套的 struct 定义
) -> Result<ClassData, LeagueLoaderError> {
    // 对于基础类型，直接返回
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
                // 对于空的 struct，可以返回一个特殊的基础类型或进行其他处理
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

            // 特殊情况：List<Struct> 被视为一个多态枚举 (Enum)
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
                // 通用情况：递归解析列表的第一个元素来确定列表的类型
                let element_class_data = if let Some(first_element) = list.get(0) {
                    extract_type_data(element_type, first_element, class_map)?
                } else {
                    // 如果列表为空，我们无法确定其内部结构，返回一个占位符
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

                    let value_slice = parser.skip_value(vtype).unwrap();
                    let value_class_data =
                        extract_type_data(vtype, value_slice, class_map).unwrap();

                    if let ClassData::Struct(class_hash) = value_class_data {
                        class_hashes.insert(class_hash);
                    }
                }
                Ok(ClassData::Map(
                    Box::new(ClassData::Base(map_base_type(&ktype))),
                    Box::new(ClassData::Enum(class_hashes)),
                ))
            } else {
                let key_slice = parser.skip_value(ktype).unwrap();
                let value_slice = parser.skip_value(vtype).unwrap();

                let key_class_data = extract_type_data(ktype, key_slice, class_map).unwrap();
                let value_class_data = extract_type_data(vtype, value_slice, class_map).unwrap();

                Ok(ClassData::Map(
                    Box::new(key_class_data),
                    Box::new(value_class_data),
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
        // 其他所有类型都已经在函数开头的 if 判断中作为基础类型处理
        _ => unreachable!(),
    }
}
