use bevy::prelude::*;
use league_utils::HashKey;
use lol_base::prop::LoadHashKeyTrait;
use lol_base::ui::{LOLEnumData, LOLUiElementEffectAnimationData};

#[derive(Component)]
pub struct UiAnimationState {
    pub handle: HashKey<LOLUiElementEffectAnimationData>,
    pub current_frame: u32,
    pub timer: f32,
}

#[derive(Default)]
pub struct PluginUIAnimation;

impl Plugin for PluginUIAnimation {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_ui_animation);
    }
}

fn update_ui_animation(
    mut commands: Commands,
    mut q_ui_animation_state: Query<(Entity, &mut UiAnimationState)>,
    anim_assets: Res<Assets<LOLUiElementEffectAnimationData>>,
    q_children: Query<&Children>,
    mut q_image_node: Query<&mut ImageNode>,
    time: Res<Time>,
) {
    for (entity, mut ui_animation_state) in q_ui_animation_state.iter_mut() {
        let Some(ui_animation) = anim_assets.load_hash(&ui_animation_state.handle) else {
            continue;
        };

        let frames_per_second = ui_animation.frames_per_second.unwrap_or(30.0);

        ui_animation_state.timer += time.delta_secs();
        if ui_animation_state.timer < 1.0 / frames_per_second {
            continue;
        }
        ui_animation_state.timer -= 1.0 / frames_per_second;

        ui_animation_state.current_frame += 1;

        let is_loop = ui_animation.finish_behavior.unwrap_or(0) == 1;

        if ui_animation_state.current_frame
            >= ui_animation.total_number_of_frames.unwrap_or(1.0) as u32
        {
            if is_loop {
                ui_animation_state.current_frame = 0;
            } else {
                commands.entity(entity).despawn();
                continue;
            }
        }

        let Some(LOLEnumData::AtlasData(atlas_data)) = ui_animation.texture_data.as_ref() else {
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

        let Ok(children) = q_children.get(entity) else {
            // warn!("未找到动画实体的子实体");
            continue;
        };
        let Some(&child) = children.first() else {
            warn!("未找到动画实体的第一个子实体");
            continue;
        };
        let Ok(mut image_node) = q_image_node.get_mut(child) else {
            warn!("未找到动画实体的第一个子实体的 image_node");
            continue;
        };
        image_node.rect = Some(Rect::new(x, y, z, w));
    }
}
