use bevy::{
    asset::{Asset, UntypedHandle},
    reflect::TypePath,
};

#[derive(Asset, TypePath)]
pub struct LeagueProperty(pub Vec<UntypedHandle>);
