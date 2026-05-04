use bevy::animation::AnimationClip;
use bevy::animation::graph::AnimationGraph;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use lol_base::animation::{ConfigAnimationNode, LOLAnimationGraph};

use crate::error::Error;

#[derive(Default, TypePath)]
pub struct LoaderConfigAnimationLoader;

impl AssetLoader for LoaderConfigAnimationLoader {
    type Asset = LOLAnimationGraph;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let content = String::from_utf8(buf).map_err(|e| Error::Parse(e.to_string()))?;
        let config_animation: LOLAnimationGraph =
            ron::from_str(&content).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(config_animation)
    }

    fn extensions(&self) -> &[&str] {
        &[".ron"]
    }
}

#[derive(Default, TypePath)]
pub struct LoaderAnimationLoader;

impl AssetLoader for LoaderAnimationLoader {
    type Asset = AnimationGraph;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let content = String::from_utf8(buf).map_err(|e| Error::Parse(e.to_string()))?;
        let mut config_animation: LOLAnimationGraph =
            ron::from_str(&content).map_err(|e| Error::Parse(e.to_string()))?;

        let mut animation_graph = AnimationGraph::new();

        let mut clips: Vec<_> = config_animation
            .hash_to_node
            .iter_mut()
            .filter_map(|(_, v)| {
                if let ConfigAnimationNode::Clip { node_index } = v {
                    Some(*node_index)
                } else {
                    None
                }
            })
            .collect();
        clips.sort_by_key(|idx| idx.index());

        for node_index in clips {
            let handle = load_context.load::<AnimationClip>(format!(
                "{}#Animation{}",
                &config_animation.gltf_path,
                node_index.index() - 1
            ));
            animation_graph.add_clip(handle, 1.0, animation_graph.root);
        }

        Ok(animation_graph)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
