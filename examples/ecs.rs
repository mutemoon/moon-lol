use std::time::Duration;

use bevy::{diagnostic::FrameCount, prelude::*, time::TimeUpdateStrategy};

fn main() {
    let mut app = App::new();

    let fixed_update_timestep = Time::<Fixed>::default().timestep();
    let time_step = fixed_update_timestep / 2 + Duration::from_millis(1);

    println!(
        "fixed_update_timestep: {:?}, time_step: {:?}",
        fixed_update_timestep, time_step
    );

    app.add_plugins(MinimalPlugins)
        .init_resource::<FixedFrameCount>()
        .add_systems(FixedUpdate, fixed_update_2)
        .add_systems(FixedUpdate, fixed_update)
        .add_systems(FixedPostUpdate, fixed_post_update)
        .add_systems(FixedLast, fixed_update_frame)
        .add_systems(PreUpdate, pre_update)
        .add_systems(Update, update)
        .add_observer(observer)
        .add_observer(observer2)
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step));

    app.update();
    app.update();
    app.update();
    app.update();
}

#[derive(Component)]
struct NeedUpdateImmediately;

#[derive(Component)]
struct NeedUpdateImmediately2;

#[derive(Event, Debug)]
struct CommandAddNeedUpdateImmediately;

#[derive(Event, Debug)]
struct CommandAddNeedUpdateImmediately2;

#[derive(Resource, Default)]
pub struct FixedFrameCount(pub u32);

fn fixed_update_frame(mut frame: ResMut<FixedFrameCount>) {
    frame.0 += 1;
}

fn fixed_update(mut commands: Commands, frame: Res<FrameCount>, fixed_frame: Res<FixedFrameCount>) {
    println!(
        "frame: {:?} fixed_frame: {:?} [FixedUpdate]",
        frame.0, fixed_frame.0,
    );
    commands.trigger(CommandAddNeedUpdateImmediately);
}

fn fixed_update_2(frame: Res<FrameCount>, fixed_frame: Res<FixedFrameCount>) {
    println!(
        "frame: {:?} fixed_frame: {:?} [FixedUpdate2]",
        frame.0, fixed_frame.0,
    );
}

fn fixed_post_update(frame: Res<FrameCount>, fixed_frame: Res<FixedFrameCount>) {
    println!(
        "frame: {:?} fixed_frame: {:?} [FixedPostUpdate]",
        frame.0, fixed_frame.0,
    );
}

fn pre_update(
    frame: Res<FrameCount>,
    fixed_frame: Res<FixedFrameCount>,
    q_need: Query<Entity, With<NeedUpdateImmediately>>,
) {
    println!(
        "frame: {:?} fixed_frame: {:?} [PreUpdate] NeedUpdateImmediately 数量: {:?}",
        frame.0,
        fixed_frame.0,
        q_need.iter().count(),
    );
}

fn update(
    frame: Res<FrameCount>,
    fixed_frame: Res<FixedFrameCount>,
    q_need: Query<Entity, With<NeedUpdateImmediately>>,
) {
    println!(
        "frame: {:?} fixed_frame: {:?} [Update] NeedUpdateImmediately 数量: {:?}",
        frame.0,
        fixed_frame.0,
        q_need.iter().count(),
    );
}

fn observer(
    _trigger: On<CommandAddNeedUpdateImmediately>,
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed_frame: Res<FixedFrameCount>,
) {
    println!(
        "frame: {:?} fixed_frame: {:?} [Observer] start",
        frame.0, fixed_frame.0,
    );
    commands.spawn(NeedUpdateImmediately);
    commands.trigger(CommandAddNeedUpdateImmediately2);
}

fn observer2(
    _trigger: On<CommandAddNeedUpdateImmediately2>,
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed_frame: Res<FixedFrameCount>,
) {
    println!(
        "frame: {:?} fixed_frame: {:?} [Observer2]",
        frame.0, fixed_frame.0,
    );
    commands.spawn(NeedUpdateImmediately2);
}
