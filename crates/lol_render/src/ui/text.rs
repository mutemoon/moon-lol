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
    mut commands: Commands,
    q_ui_text_state: Query<(&UiTextState, &Children), Changed<UiTextState>>,
) {
    for (state, children) in q_ui_text_state.iter() {
        for child in children.iter() {
            commands.entity(child).insert(Text::new(state.text.clone()));
        }
    }
}
