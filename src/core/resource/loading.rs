use std::fmt::Debug;
use std::ops::Deref;

use bevy::prelude::*;
use league_core::{AnimationGraphData, CharacterRecord, SkinCharacterDataProperties};
use league_file::LeagueSkeleton;
use lol_config::HashKey;
use lol_core::LeagueSkinMesh;

pub struct PluginResourceLoading;

impl Plugin for PluginResourceLoading {
    fn build(&self, app: &mut App) {
        app.register_loading::<HashKey<CharacterRecord>>()
            .register_loading::<HashKey<AnimationGraphData>>()
            .register_loading::<HashKey<SkinCharacterDataProperties>>()
            .register_loading::<Handle<LeagueSkeleton>>()
            .register_loading::<(Handle<LeagueSkinMesh>, Handle<StandardMaterial>)>();
    }
}

pub trait RegisterLoadingExt {
    fn register_loading<T: Send + Sync + Debug + 'static>(&mut self) -> &mut Self;
}

impl RegisterLoadingExt for App {
    fn register_loading<T: Send + Sync + Debug + 'static>(&mut self) -> &mut Self {
        self.add_systems(Update, update_loading::<T>);
        self
    }
}

#[derive(Component, Resource)]
pub struct Loading<F> {
    pub timer: Timer,
    pub value: F,
}

impl<F> Loading<F> {
    pub fn new(value: F) -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Once),
            value,
        }
    }

    pub fn is_timeout(&self) -> bool {
        self.timer.is_finished()
    }

    pub fn update(&mut self, time: &Time) {
        self.timer.tick(time.delta());
    }

    pub fn set(&mut self, value: F) {
        self.value = value;
    }
}

impl<T> Deref for Loading<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

fn update_loading<T: Send + Sync + Debug + 'static>(
    mut commands: Commands,
    mut q_loading: Query<(Entity, &mut Loading<T>)>,
    time: Res<Time>,
) {
    for (entity, mut loading) in q_loading.iter_mut() {
        loading.update(&time);

        if loading.is_timeout() {
            info!("加载超时: {:?}", loading.value);
            commands.entity(entity).remove::<Loading<T>>();
        }
    }
}
