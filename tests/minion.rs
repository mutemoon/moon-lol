#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{prelude::*, time::TimeUpdateStrategy};
    use moon_lol::{
        PluginAction, PluginAttack, PluginAttackAuto, PluginBarrack, PluginDamage, PluginLife,
        PluginMinion, PluginMovement, PluginResource,
    };

    #[test]
    fn test_complete_attack_cycle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(AnimationPlugin::default());

        app.add_plugins(PluginMinion);

        app.add_plugins(PluginAttack);
        app.add_plugins(PluginAction);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginDamage);
        app.add_plugins(PluginLife);

        app.add_plugins(PluginAttackAuto);
        app.add_plugins(PluginBarrack);
        app.add_plugins(PluginResource {
            game_config_path: "games/null.ron".to_owned(),
        });

        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));

        app.run();
    }
}
