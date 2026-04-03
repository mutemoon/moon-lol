use bevy::animation::animation_curves::{AnimatableCurve, AnimatableKeyframeCurve};
use bevy::animation::{animated_field, AnimationClip, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use league_file::animation::AnimationFile;
use league_to_lol::animation::load_animation_file;

use super::error::Error;

#[derive(Default)]
pub struct LeagueLoaderAnimationClip;

impl AssetLoader for LeagueLoaderAnimationClip {
    type Asset = AnimationClip;

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
        let (_, animation_file) =
            AnimationFile::parse(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        let animation = load_animation_file(animation_file);

        let mut clip = AnimationClip::default();
        for (i, join_hash) in animation.joint_hashes.iter().enumerate() {
            let translates = animation.translates.get(i).unwrap();
            let rotations = animation.rotations.get(i).unwrap();
            let scales = animation.scales.get(i).unwrap();

            if translates.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::translation),
                        AnimatableKeyframeCurve::new(translates.clone()).unwrap(),
                    ),
                );
            }

            if rotations.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::rotation),
                        AnimatableKeyframeCurve::new(rotations.clone()).unwrap(),
                    ),
                );
            }

            if scales.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::scale),
                        AnimatableKeyframeCurve::new(scales.clone().into_iter()).unwrap(),
                    ),
                );
            }
        }
        Ok(clip)
    }

    fn extensions(&self) -> &[&str] {
        &["anm"]
    }
}
