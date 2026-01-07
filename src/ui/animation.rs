use bevy::prelude::*;
use league_core::{EnumData, EnumUiPosition, UiElementEffectAnimationData};
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::spawn_ui_atom;

#[derive(Component)]
pub struct UiAnimationState {
    key: String,
    current_frame: u32,
    timer: f32,
}

#[derive(Event)]
pub struct CommandUiAnimationStart {
    pub key: String,
}

#[derive(Default)]
pub struct PluginUIAnimation;

impl Plugin for PluginUIAnimation {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_ui_animation);
        app.add_observer(on_command_ui_animation_start);
    }
}

fn on_command_ui_animation_start(
    event: On<CommandUiAnimationStart>,
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    res_ui_animation: Res<Assets<UiElementEffectAnimationData>>,
) {
    let ui_animation = res_ui_animation.load_hash(&hash_bin(&event.key)).unwrap();
    let Some(entity) = spawn_ui_atom(
        &mut commands,
        &res_asset_server,
        &ui_animation.name,
        &EnumUiPosition::UiPositionRect(ui_animation.position.clone()),
        &ui_animation.layer,
        &Some(ui_animation.texture_data.clone()),
    ) else {
        return;
    };

    commands.entity(entity).insert((
        UiAnimationState {
            key: event.key.clone(),
            current_frame: 0,
            timer: 0.0,
        },
        Visibility::Visible,
    ));
}

fn update_ui_animation(
    mut commands: Commands,
    mut q_ui_animation_state: Query<(Entity, &mut UiAnimationState)>,
    res_ui_animation: Res<Assets<UiElementEffectAnimationData>>,
    q_children: Query<&Children>,
    mut q_image_node: Query<&mut ImageNode>,
    time: Res<Time>,
) {
    for (entity, mut ui_animation_state) in q_ui_animation_state.iter_mut() {
        let ui_animation = res_ui_animation
            .load_hash(&hash_bin(&ui_animation_state.key))
            .unwrap();

        let frames_per_second = ui_animation.frames_per_second.unwrap_or(30.0);

        ui_animation_state.timer += time.delta_secs();
        if ui_animation_state.timer < 1.0 / frames_per_second {
            continue;
        }
        ui_animation_state.timer -= 1.0 / frames_per_second;

        ui_animation_state.current_frame += 1;

        // let is_loop = ui_animation.m_finish_behavior.unwrap_or(0) == 1;
        let is_loop = true;

        if ui_animation_state.current_frame
            >= ui_animation.total_number_of_frames.unwrap_or(1.0) as u32
        {
            if is_loop {
                ui_animation_state.current_frame = 0;
            } else {
                commands.entity(entity).despawn();
            }
        }

        let EnumData::AtlasData(ref atlas_data) = ui_animation.texture_data else {
            continue;
        };
        let Some(m_texture_uv) = atlas_data.m_texture_uv else {
            continue;
        };
        let number_of_frames_per_row_in_atlas = ui_animation
            .number_of_frames_per_row_in_atlas
            .unwrap_or(1.0) as u32;

        let row_index = ui_animation_state.current_frame / number_of_frames_per_row_in_atlas;
        let col_index = ui_animation_state.current_frame % number_of_frames_per_row_in_atlas;

        let width = m_texture_uv.z - m_texture_uv.x + 1.0;
        let height = m_texture_uv.w - m_texture_uv.y + 1.0;

        let x = m_texture_uv.x + col_index as f32 * width;
        let y = m_texture_uv.y + row_index as f32 * height;
        let z = x + width;
        let w = y + height;

        let &child = q_children.get(entity).unwrap().first().unwrap();
        let mut image_node = q_image_node.get_mut(child).unwrap();
        image_node.rect = Some(Rect::new(x, y, z, w));
    }
}
