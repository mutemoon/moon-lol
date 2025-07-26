use crate::render::{LeagueLoader, LeagueMapGeo};
use bevy::prelude::*;
use binrw::BinRead;
use cdragon_prop::PropFile;
use std::io::Cursor;

#[derive(Resource)]
pub struct WadRes {
    pub loader: LeagueLoader,
}

pub struct PROP(pub PropFile);

unsafe impl Sync for PROP {}
unsafe impl Send for PROP {}

pub struct PluginResource;

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.insert_resource::<WadRes>(WadRes {
            loader: LeagueLoader::new(
                r"C:\Program Files (x86)\WeGameApps\英雄联盟\Game\",
                r"DATA\FINAL\Maps\Shipping\Map11.wad.client",
            )
            .unwrap(),
        });
    }
}

// #[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash, EnumIter)]
// pub enum GltfAsset {
//     MinionMeleeBlue,
//     MinionMeleeRed,
//     MinionRangedBlue,
//     MinionRangedRed,
//     MinionSiegeBlue,
//     MinionSiegeRed,
//     BaseSrx,
//     TurretBlue,
//     TurretRed,
//     NexusBlue,
//     NexusRed,
//     InhibitorBlue,
//     InhibitorRed,
//     FioraSkin22,
// }

// impl GltfAsset {
//     pub fn path(&self) -> &'static str {
//         match self {
//             GltfAsset::MinionMeleeBlue => "transformed_data/minion_melee_blue.gltf",
//             GltfAsset::MinionMeleeRed => "transformed_data/minion_melee_red.gltf",
//             GltfAsset::MinionRangedBlue => "transformed_data/minion_ranged_blue.gltf",
//             GltfAsset::MinionRangedRed => "transformed_data/minion_ranged_red.gltf",
//             GltfAsset::MinionSiegeBlue => "transformed_data/minion_siege_blue.gltf",
//             GltfAsset::MinionSiegeRed => "transformed_data/minion_siege_red.gltf",
//             GltfAsset::BaseSrx => "transformed_data/base_srx.gltf",
//             GltfAsset::TurretBlue => "transformed_data/turret_blue.gltf",
//             GltfAsset::TurretRed => "transformed_data/turret_red.gltf",
//             GltfAsset::NexusBlue => "transformed_data/nexus_blue.gltf",
//             GltfAsset::NexusRed => "transformed_data/nexus_red.gltf",
//             GltfAsset::InhibitorBlue => "transformed_data/inhibitor_blue.gltf",
//             GltfAsset::InhibitorRed => "transformed_data/inhibitor_red.gltf",
//             GltfAsset::FioraSkin22 => "transformed_data/fiora_skin22.gltf",
//         }
//     }

//     pub fn animation_name_idle(&self) -> &'static str {
//         match self {
//             GltfAsset::MinionMeleeBlue => "minion_melee_idle",
//             GltfAsset::MinionMeleeRed => "minion_melee_idle",
//             GltfAsset::MinionRangedBlue => "minion_caster_idle",
//             GltfAsset::MinionRangedRed => "minion_caster_idle",
//             GltfAsset::MinionSiegeBlue => "Cannon_Order_Idle1",
//             GltfAsset::MinionSiegeRed => "Cannon_Order_Idle1",
//             GltfAsset::BaseSrx => "",
//             GltfAsset::TurretBlue => "Idle1.BOBA_ENV",
//             GltfAsset::TurretRed => "Idle1.BOBA_ENV",
//             GltfAsset::NexusBlue => "SRS_OrderNexus_idle.BOBA_ENV",
//             GltfAsset::NexusRed => "SRS_OrderNexus_idle.BOBA_ENV",
//             GltfAsset::InhibitorBlue => "",
//             GltfAsset::InhibitorRed => "",
//             GltfAsset::FioraSkin22 => "fiora_skin22_idle1",
//         }
//     }

//     pub fn animation_name_moving(&self) -> &'static str {
//         match self {
//             GltfAsset::MinionMeleeBlue => "minion_melee_run",
//             GltfAsset::MinionMeleeRed => "minion_melee_run",
//             GltfAsset::MinionRangedBlue => "minion_caster_run",
//             GltfAsset::MinionRangedRed => "minion_caster_run",
//             GltfAsset::MinionSiegeBlue => "Cannon_Order_Run1",
//             GltfAsset::MinionSiegeRed => "Cannon_Order_Run1",
//             GltfAsset::BaseSrx => "",
//             GltfAsset::TurretBlue => "",
//             GltfAsset::TurretRed => "",
//             GltfAsset::NexusBlue => "",
//             GltfAsset::NexusRed => "",
//             GltfAsset::InhibitorBlue => "",
//             GltfAsset::InhibitorRed => "",
//             GltfAsset::FioraSkin22 => "fiora_skin22_run",
//         }
//     }

//     pub fn animation_name_attack(&self) -> &'static str {
//         match self {
//             GltfAsset::MinionMeleeBlue => "minion_melee_attack",
//             GltfAsset::MinionMeleeRed => "minion_melee_attack",
//             GltfAsset::MinionRangedBlue => "minion_caster_attack1",
//             GltfAsset::MinionRangedRed => "minion_caster_attack1",
//             GltfAsset::MinionSiegeBlue => "Cannon_Order_Attack1",
//             GltfAsset::MinionSiegeRed => "Cannon_Order_Attack1",
//             GltfAsset::BaseSrx => "",
//             GltfAsset::TurretBlue => "Attack_0.BOBA_ENV",
//             GltfAsset::TurretRed => "Attack_0.BOBA_ENV",
//             GltfAsset::NexusBlue => "",
//             GltfAsset::NexusRed => "",
//             GltfAsset::InhibitorBlue => "",
//             GltfAsset::InhibitorRed => "",
//             GltfAsset::FioraSkin22 => "fiora_skin22_attack1",
//         }
//     }

//     pub fn get_animation_name(&self, animation_type: &AnimationType) -> &'static str {
//         match animation_type {
//             AnimationType::Idle => self.animation_name_idle(),
//             AnimationType::Moving => self.animation_name_moving(),
//             AnimationType::Attack => self.animation_name_attack(),
//         }
//     }
// }

// #[derive(Component)]
// pub struct GltfSceneHandle(pub GltfAsset);

// #[derive(Component)]
// pub struct GltfAnimationHandle(pub GltfAsset);

// #[derive(Resource)]
// pub struct GltfAssets {
//     gltfs: HashMap<GltfAsset, Handle<Gltf>>,
//     animations: HashMap<
//         GltfAsset,
//         (
//             Handle<AnimationGraph>,
//             HashMap<Box<str>, AnimationNodeIndex>,
//         ),
//     >,
// }

// impl Default for GltfAssets {
//     fn default() -> Self {
//         Self {
//             gltfs: HashMap::new(),
//             animations: HashMap::new(),
//         }
//     }
// }

// impl GltfAssets {
//     pub fn get(&self, asset_type: GltfAsset) -> Option<&Handle<Gltf>> {
//         self.gltfs.get(&asset_type)
//     }

//     pub fn load_all(&mut self, asset_server: &AssetServer) {
//         GltfAsset::iter().for_each(|asset_type| {
//             self.load(asset_server, asset_type);
//         });
//     }

//     pub fn load(&mut self, asset_server: &AssetServer, asset_type: GltfAsset) {
//         self.gltfs
//             .insert(asset_type, asset_server.load(asset_type.path()));
//     }

//     pub fn insert_scene_handle(
//         &mut self,
//         entity: Entity,
//         asset_type: GltfAsset,
//         commands: &mut Commands,
//     ) {
//         let mut entity = commands.entity(entity);
//         entity.insert(GltfSceneHandle(asset_type));
//         entity.insert(asset_type);
//         if !asset_type.animation_name_idle().is_empty() {
//             entity.insert(GltfAnimationHandle(asset_type));
//         }
//     }

//     pub fn insert_scene_root(
//         &mut self,
//         entity: Entity,
//         gltf_scene_handle: &GltfSceneHandle,
//         gltfs: &Res<Assets<Gltf>>,
//         commands: &mut Commands,
//     ) {
//         let Some(handle) = self.gltfs.get(&gltf_scene_handle.0) else {
//             return;
//         };
//         let Some(gltf) = gltfs.get(handle) else {
//             return;
//         };
//         let mut entity = commands.entity(entity);
//         let scene = gltf.scenes.first().unwrap().clone();
//         entity.insert(SceneRoot(scene));
//         entity.remove::<GltfSceneHandle>();
//     }

//     pub fn init_animation(
//         &mut self,
//         animation_player_entity: Entity,
//         asset_type: GltfAsset,
//         commands: &mut Commands,
//         entity: Entity,
//         gltfs: &Res<Assets<Gltf>>,
//         graphs: &mut ResMut<Assets<AnimationGraph>>,
//     ) {
//         let animation_name = asset_type.animation_name_idle();
//         if animation_name.is_empty() {
//             return;
//         }

//         let Some(handle) = self.get(asset_type) else {
//             return;
//         };

//         let Some(gltf) = gltfs.get(handle) else {
//             return;
//         };

//         let graph = if let Some(anim) = self.animations.get(&asset_type) {
//             &anim.0
//         } else {
//             let named_animations = &gltf.named_animations;
//             let animations = named_animations.values().cloned().collect::<Vec<_>>();
//             let (graph, node_indices) = AnimationGraph::from_clips(animations);

//             let mut animation_named_indices = HashMap::new();
//             for (index, name) in named_animations.keys().enumerate() {
//                 let Some(node) = node_indices.get(index) else {
//                     panic!("Animation node index not found");
//                 };
//                 animation_named_indices.insert(name.clone(), node.clone());
//             }

//             self.animations
//                 .insert(asset_type, (graphs.add(graph), animation_named_indices));
//             &self.animations.get(&asset_type).unwrap().0
//         };

//         commands.entity(animation_player_entity).insert((
//             AnimationGraphHandle(graph.clone()),
//             AnimationTransitions::new(),
//         ));
//         commands.entity(entity).remove::<GltfAnimationHandle>();
//     }

//     pub fn is_loaded(&self, asset_type: GltfAsset, gltfs: &Res<Assets<Gltf>>) -> bool {
//         let Some(handle) = self.get(asset_type) else {
//             return false;
//         };
//         gltfs.get(handle).is_some()
//     }

//     pub fn is_all_loaded(&self, gltfs: &Res<Assets<Gltf>>) -> bool {
//         GltfAsset::iter().all(|asset_type| self.is_loaded(asset_type, gltfs))
//     }
// }

// #[derive(Debug)]
// pub enum AnimationType {
//     Idle,
//     Moving,
//     Attack,
// }

// #[derive(Component)]
// pub struct AnimationInfo {
//     pub animation_type: AnimationType,
// }

// impl AnimationInfo {
//     pub fn play(
//         &self,
//         gltf_assets: &Res<GltfAssets>,
//         asset_type: &GltfAsset,
//         animation_type: &AnimationType,
//         animation_transitions: &mut AnimationTransitions,
//         animation_player: &mut AnimationPlayer,
//     ) {
//         let Some((_, named_indices)) = gltf_assets.animations.get(asset_type) else {
//             return;
//         };
//         let Some(animation_index) =
//             named_indices.get(asset_type.get_animation_name(animation_type))
//         else {
//             return;
//         };
//         animation_transitions
//             .play(
//                 animation_player,
//                 animation_index.clone(),
//                 Duration::from_millis(0),
//             )
//             .repeat();
//     }
// }
