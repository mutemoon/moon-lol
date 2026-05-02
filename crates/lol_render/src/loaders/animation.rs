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
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let content = String::from_utf8(buf).map_err(|e| Error::Parse(e.to_string()))?;
        let mut config_animation: LOLAnimationGraph =
            ron::from_str(&content).map_err(|e| Error::Parse(e.to_string()))?;

        // Build AnimationGraph from gltf_path
        let mut animation_graph = AnimationGraph::new();
        animation_graph.root;

        config_animation.hash_to_node.iter_mut().for_each(|(_, v)| {
            if let ConfigAnimationNode::Clip { node_index } = v {
                let handle = load_context.load::<AnimationClip>(format!(
                    "{}#Animation{}",
                    &config_animation.gltf_path,
                    node_index.index()
                ));

                *node_index = animation_graph.add_clip(handle, 1.0, animation_graph.root);
            }
        });

        load_context.add_labeled_asset("animation_graph", animation_graph);

        Ok(config_animation)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
