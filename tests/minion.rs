#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::log::LogPlugin;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use lol_core::action::PluginAction;
    use lol_core::attack::PluginAttack;
    use lol_core::attack_auto::PluginAttackAuto;
    use lol_core::character::PluginCharacter;
    use lol_core::damage::PluginDamage;
    use lol_core::entities::barrack::PluginBarrack;
    use lol_core::entities::minion::PluginMinion;
    use lol_core::life::PluginLife;
    use lol_core::map::MinionPath;
    use lol_core::movement::PluginMovement;
    use lol_core::navigation::navigation::PluginNavigaton;
    use lol_core::resource::PluginResource;

    #[test]
    fn test_complete_attack_cycle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(LogPlugin::default());
        app.add_plugins(AnimationPlugin::default());
        app.init_asset::<Shader>();

        app.add_plugins(PluginMinion);

        app.add_plugins(PluginAttack);
        app.add_plugins(PluginAction);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginNavigaton);
        app.add_plugins(PluginDamage);
        app.add_plugins(PluginLife);
        app.add_plugins(PluginCharacter);

        app.add_plugins(PluginAttackAuto);
        app.add_plugins(PluginBarrack);
        app.add_plugins(PluginResource {
            game_config_path: "games/null.ron".to_owned(),
        });

        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));
        app.insert_resource(MinionPath::default());

        app.finish();
        app.cleanup();

        for _ in 0..5 {
            app.update();
        }
    }
}
