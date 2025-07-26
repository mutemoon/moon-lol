#[cfg(test)]
mod tests {
    use bevy::{prelude::*, state::app::StatesPlugin};
    use moon_lol::{
        classic::PluginClassic,
        combat::PluginMove,
        combat::{Health, PluginCombat},
        entities::{Minion, PluginMinion},
    };
    use std::time::{Duration, Instant};

    const TEST_FIXED_UPDATE_TIMES: f32 = 200000.0;

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_systems(Startup, setup);
        // app.add_plugins(PluginMinion);
        app.add_plugins((PluginMove, PluginClassic, PluginCombat));
        app
    }

    fn setup(mut virtual_time: ResMut<Time<Virtual>>) {
        virtual_time.set_relative_speed(TEST_FIXED_UPDATE_TIMES);
        virtual_time.set_max_delta(Duration::MAX);
    }

    #[test]
    fn test_minion() {
        let mut app = create_test_app();
        let start = Instant::now();
        app.update();
        // 第二次 update 需要运行 TEST_FIXED_UPDATE_TIMES 次 FixedUpdate
        app.update();

        let virtual_time = app.world().resource::<Time<Fixed>>().elapsed();
        let real_time = start.elapsed();
        println!("游戏时间: {:?}", virtual_time);
        println!("实际时间: {:?}", real_time);
        println!(
            "加速比: {:.1}",
            virtual_time.as_secs_f32() / real_time.as_secs_f32()
        );

        let world = app.world_mut();

        for v in world.query_filtered::<&Health, With<Minion>>().iter(&world) {
            println!("minion health: {}", v.value);
        }

        assert!(
            world
                .query_filtered::<&Health, With<Minion>>()
                .iter(&world)
                .count()
                > 0
        );
    }
}
