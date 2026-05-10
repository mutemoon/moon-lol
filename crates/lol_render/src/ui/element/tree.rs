use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;

use bevy::prelude::info;
use lol_base::ui::{
    LOLUiElementEffectAnimationData, LOLUiElementEffectDesaturateData,
    LOLUiElementEffectInstancedData, LOLUiElementGroupButtonData, LOLUiElementIconData,
    LOLUiElementRegionData, LOLUiElementTextData, LOLUiSceneData,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct UITreeNode {
    name: String,
    kind: String,
    visibility: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    layer: Option<i32>,
    children: Vec<UITreeNode>,
}

pub fn save_ui_tree_to_json(
    scenes: &HashMap<u32, LOLUiSceneData>,
    elements: &HashMap<u32, LOLUiElementIconData>,
    buttons: &HashMap<u32, LOLUiElementGroupButtonData>,
    animations: &HashMap<u32, LOLUiElementEffectAnimationData>,
    regions: &HashMap<u32, LOLUiElementRegionData>,
    texts: &HashMap<u32, LOLUiElementTextData>,
    instanceds: &HashMap<u32, LOLUiElementEffectInstancedData>,
) {
    let mut scene_children: HashMap<u32, Vec<u32>> = HashMap::new();
    for (hash, scene) in scenes {
        if let Some(parent) = scene.parent_scene {
            scene_children.entry(parent).or_default().push(*hash);
        }
    }

    let mut scene_content: HashMap<u32, Vec<UITreeNode>> = HashMap::new();
    let mut processed_hashes = HashSet::new();

    // 建立 Icon -> Button 的映射，用于层级嵌套
    let mut icon_to_button: HashMap<u32, u32> = HashMap::new();
    for (hash, button) in buttons {
        for icon_hash in &button.elements {
            icon_to_button.insert(icon_hash.0.0, *hash);
        }
    }

    // 先处理 Button，因为它们可能包含 Icon
    for (hash, button) in buttons {
        if !processed_hashes.insert(*hash) {
            continue;
        }
        let mut button_node = UITreeNode {
            name: button.name.clone(),
            kind: "Button".to_string(),
            visibility: if button.is_enabled.unwrap_or(true) {
                "V"
            } else {
                "H"
            }
            .to_string(),
            layer: None,
            children: vec![],
        };

        // 将属于该 Button 的 Icon 加入其 children
        for icon_hash in &button.elements {
            if let Some(icon) = elements.get(&icon_hash.0.0) {
                button_node.children.push(UITreeNode {
                    name: icon.name.clone(),
                    kind: "Icon".to_string(),
                    visibility: if icon.enabled { "V" } else { "H" }.to_string(),
                    layer: icon.layer.map(|l| l as i32),
                    children: vec![],
                });
                processed_hashes.insert(icon_hash.0.0);
            }
        }

        scene_content
            .entry(button.scene.0.0)
            .or_default()
            .push(button_node);
    }

    // 处理剩余的 Icon (不属于任何 Button 的)
    for (hash, element) in elements {
        if processed_hashes.contains(hash) {
            continue;
        }
        if !processed_hashes.insert(*hash) {
            continue;
        }
        scene_content
            .entry(element.scene.0.0)
            .or_default()
            .push(UITreeNode {
                name: element.name.clone(),
                kind: "Icon".to_string(),
                visibility: if element.enabled { "V" } else { "H" }.to_string(),
                layer: element.layer.map(|l| l as i32),
                children: vec![],
            });
    }

    for (hash, anim) in animations {
        if !processed_hashes.insert(*hash) {
            continue;
        }
        scene_content
            .entry(anim.scene)
            .or_default()
            .push(UITreeNode {
                name: anim.name.clone(),
                kind: "Anim".to_string(),
                visibility: "V".to_string(),
                layer: anim.layer.map(|l| l as i32),
                children: vec![],
            });
    }

    for (hash, region) in regions {
        if !processed_hashes.insert(*hash) {
            continue;
        }
        scene_content
            .entry(region.scene)
            .or_default()
            .push(UITreeNode {
                name: region.name.clone(),
                kind: "Region".to_string(),
                visibility: "V".to_string(),
                layer: None,
                children: vec![],
            });
    }

    for (hash, text) in texts {
        if !processed_hashes.insert(*hash) {
            continue;
        }
        scene_content
            .entry(text.scene)
            .or_default()
            .push(UITreeNode {
                name: text.name.clone(),
                kind: "Text".to_string(),
                visibility: if text.enabled { "V" } else { "H" }.to_string(),
                layer: text.layer.map(|l| l as i32),
                children: vec![],
            });
    }

    for (hash, instanced) in instanceds {
        if !processed_hashes.insert(*hash) {
            continue;
        }
        scene_content
            .entry(instanced.scene)
            .or_default()
            .push(UITreeNode {
                name: instanced.name.clone(),
                kind: "Instanced".to_string(),
                visibility: if instanced.enabled { "V" } else { "H" }.to_string(),
                layer: Some(instanced.layer as i32),
                children: vec![],
            });
    }

    fn build_node(
        hash: u32,
        scenes: &HashMap<u32, LOLUiSceneData>,
        scene_children: &HashMap<u32, Vec<u32>>,
        scene_content: &HashMap<u32, Vec<UITreeNode>>,
    ) -> Option<UITreeNode> {
        let scene = scenes.get(&hash)?;
        let mut node = UITreeNode {
            name: scene.name.clone(),
            kind: "Scene".to_string(),
            visibility: if scene.enabled { "V" } else { "H" }.to_string(),
            layer: scene.layer.map(|l| l as i32),
            children: vec![],
        };

        // 添加内容元素
        if let Some(contents) = scene_content.get(&hash) {
            for content in contents {
                node.children.push(UITreeNode {
                    name: content.name.clone(),
                    kind: content.kind.clone(),
                    visibility: content.visibility.clone(),
                    layer: content.layer,
                    children: vec![],
                });
            }
        }

        // 添加子场景
        if let Some(children) = scene_children.get(&hash) {
            for &child_hash in children {
                if let Some(child_node) =
                    build_node(child_hash, scenes, scene_children, scene_content)
                {
                    node.children.push(child_node);
                }
            }
        }

        Some(node)
    }

    let mut roots = vec![];
    for (hash, scene) in scenes {
        if scene.parent_scene.is_none() || !scenes.contains_key(&scene.parent_scene.unwrap()) {
            if let Some(root_node) = build_node(*hash, scenes, &scene_children, &scene_content) {
                roots.push(root_node);
            }
        }
    }

    if let Ok(json) = serde_json::to_string_pretty(&roots) {
        if let Ok(mut file) = File::create("ui_tree.json") {
            let _ = file.write_all(json.as_bytes());
            info!("UI 树已保存至 ui_tree.json");
        }
    }
}
