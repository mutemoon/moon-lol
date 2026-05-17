use bevy::prelude::*;

#[derive(Component)]
pub struct UiTextState {
    pub text: String,
}

#[derive(Default)]
pub struct PluginUIText;

impl Plugin for PluginUIText {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_ui_text);
    }
}

fn update_ui_text(
    q_ui_text_state: Query<(&UiTextState, &Children), Changed<UiTextState>>,
    mut q_text: Query<&mut Text>,
) {
    for (state, children) in q_ui_text_state.iter() {
        info!("更新ui文本: {}", state.text);
        for child in children.iter() {
            if let Ok(mut text) = q_text.get_mut(child) {
                info!("更新ui文本 确实更新了: {}", text.0);
                text.0 = state.text.clone();
            }
        }
    }
}
