use bevy::prelude::*;

use crate::ui::element::layout::update_on_add_ui_element;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct UiTextState {
    pub text: String,
}

#[derive(Default)]
pub struct PluginUIText;

impl Plugin for PluginUIText {
    fn build(&self, app: &mut App) {
        app.register_type::<UiTextState>();
        app.add_systems(
            Update,
            (update_ui_text, sync_pending_ui_text).after(update_on_add_ui_element),
        );
    }
}

/// 把 `UiTextState.text` 同步到文本元素的首个子实体（`Text`）。
///
/// 子实体在构建期由 `init.rs` 预置（`UIElementChild + Node + Text`），但 `UiTextState`
/// 可能先于子实体就绪；未就绪时打 `PendingUiText` 标记，子实体出现后再补同步。
fn update_ui_text(
    mut commands: Commands,
    q_ui_text_state: Query<(Entity, &UiTextState), Changed<UiTextState>>,
    q_children: Query<&Children>,
    mut q_text: Query<&mut Text>,
) {
    for (entity, state) in q_ui_text_state.iter() {
        let child = q_children.get(entity).ok().and_then(|c| c.first().copied());
        match child {
            // 子实体就绪：直接写文本。
            Some(child) => {
                if let Ok(mut text) = q_text.get_mut(child)
                    && text.0 != state.text
                {
                    text.0 = state.text.clone();
                }
                commands.entity(entity).remove::<PendingUiText>();
            }
            // 子实体未就绪（元素还没布局）：记下待同步，布局完成后补。
            None => {
                commands.entity(entity).insert(PendingUiText);
            }
        }
    }
}

/// 文本元素的子实体可能在 `UiTextState` 变更之后才出现（启动期实体异步加载）。
/// 子实体一出现就补同步，避免文本永远停在占位值。
fn sync_pending_ui_text(
    mut commands: Commands,
    q_pending: Query<Entity, With<PendingUiText>>,
    q_children: Query<&Children>,
    q_ui_text_state: Query<&UiTextState>,
    mut q_text: Query<&mut Text>,
) {
    for entity in q_pending.iter() {
        let Some(child) = q_children.get(entity).ok().and_then(|c| c.first().copied()) else {
            continue;
        };
        let Ok(state) = q_ui_text_state.get(entity) else {
            commands.entity(entity).remove::<PendingUiText>();
            continue;
        };
        if let Ok(mut text) = q_text.get_mut(child)
            && text.0 != state.text
        {
            text.0 = state.text.clone();
        }
        commands.entity(entity).remove::<PendingUiText>();
    }
}

/// 文本状态还没能写到子实体上（子实体尚未出现）。
#[derive(Component)]
struct PendingUiText;

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    /// 文本元素在 `UiTextState` 写好之后才长出子实体时，也要把文本补上。
    #[test]
    fn pending_state_syncs_once_child_appears() {
        let mut app = app();
        let element = app
            .world_mut()
            .spawn(UiTextState { text: "25".into() })
            .id();

        app.world_mut().run_system_once(update_ui_text).unwrap();
        assert!(app.world().get::<PendingUiText>(element).is_some());
        assert!(app.world().get::<Children>(element).is_none());

        // 布局完成：子实体带 `Text` 长出。
        let child = app.world_mut().spawn(Text::new("10")).id();
        app.world_mut().entity_mut(element).add_child(child);

        app.world_mut()
            .run_system_once(sync_pending_ui_text)
            .unwrap();
        assert_eq!(app.world().get::<Text>(child).unwrap().0, "25");
        assert!(app.world().get::<PendingUiText>(element).is_none());
    }

    /// 元素已有子实体时，状态变化直接写进 `Text`。
    #[test]
    fn state_change_writes_child_text() {
        let mut app = app();
        let child = app.world_mut().spawn(Text::new("10")).id();
        let element = app
            .world_mut()
            .spawn(UiTextState {
                text: "0/0/0".into(),
            })
            .id();
        app.world_mut().entity_mut(element).add_child(child);

        app.world_mut().run_system_once(update_ui_text).unwrap();
        assert_eq!(app.world().get::<Text>(child).unwrap().0, "0/0/0");
        assert!(app.world().get::<PendingUiText>(element).is_none());
    }
}
