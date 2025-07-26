use crate::{system_debug, system_info};
use bevy::{
    app::{App, Plugin},
    prelude::*,
};
use vleue_navigator::prelude::PrimitiveObstacle;

use crate::{
    combat::*,
    config::*,
    entities::{Inhibitor, Nexus, Turret},
    game::GameState,
    map::Lane,
};

pub struct PluginClassic;

impl Plugin for PluginClassic {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_systems(OnEnter(GameState::Setup), setup);
        app.add_systems(FixedPreUpdate, add_obstacle);
    }
}

pub fn setup(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    system_info!("classic_setup", "Setting up classic game mode");

    let teams = [Team::Red, Team::Blue];
    let lanes = [Lane::Top, Lane::Mid, Lane::Bottom];

    for &team in teams.iter() {
        system_debug!("classic_setup", "Creating structures for team {:?}", team);
        create_nexus(&mut commands, team);

        for &lane in lanes.iter() {
            system_debug!(
                "classic_setup",
                "Creating structures for team {:?} in lane {:?}",
                team,
                lane
            );
            create_turret_nexus(&mut commands, team, lane);
            create_first_turret(&mut commands, team, lane);
            create_second_turret(&mut commands, team, lane);
            create_third_turret(&mut commands, team, lane);
            create_inhibitor(&mut commands, team, lane);
        }
    }

    system_info!(
        "classic_setup",
        "Classic game setup complete, transitioning to Playing state"
    );
    next_state.set(GameState::Playing);
}

pub fn add_obstacle(
    mut commands: Commands,
    q_obstacle: Query<(Entity, &Bounding), (With<Obstacle>, Without<PrimitiveObstacle>)>,
) {
    let obstacle_count = q_obstacle.iter().count();
    if obstacle_count > 0 {
        system_debug!(
            "add_obstacle",
            "Adding primitive obstacles to {} entities",
            obstacle_count
        );
    }

    q_obstacle.iter().for_each(|v| {
        system_debug!(
            "add_obstacle",
            "Adding RegularPolygon obstacle to entity {:?} (radius={:.1}, sides={})",
            v.0,
            v.1.radius,
            v.1.sides
        );
        commands
            .entity(v.0)
            .insert(PrimitiveObstacle::RegularPolygon(RegularPolygon::new(
                v.1.radius, v.1.sides,
            )));
    });

    if obstacle_count > 0 {
        system_info!(
            "add_obstacle",
            "Successfully added {} primitive obstacles",
            obstacle_count
        );
    }
}

fn create_nexus(commands: &mut Commands, team: Team) {
    let position = if team == Team::Red {
        NEXUS_RED_POSITION
    } else {
        NEXUS_BLUE_POSITION
    };
    commands.spawn((
        Nexus,
        Health {
            value: NEXUS_HEALTH,
            max: NEXUS_HEALTH,
        },
        Transform::from_translation(position),
        team,
    ));
}

fn create_turret_nexus(commands: &mut Commands, team: Team, position: Lane) {
    let position = match team {
        Team::Red => match position {
            Lane::Top => TURRET_RED_NEXUS_TOP_POSITION,
            Lane::Bottom => TURRET_RED_NEXUS_BOTTOM_POSITION,
            _ => return,
        },
        Team::Blue => match position {
            Lane::Top => TURRET_BLUE_NEXUS_TOP_POSITION,
            Lane::Bottom => TURRET_BLUE_NEXUS_BOTTOM_POSITION,
            _ => return,
        },
    };
    commands.spawn((
        Turret,
        Health {
            value: TURRET_NEXUS_HEALTH,
            max: TURRET_NEXUS_HEALTH,
        },
        Transform::from_translation(position),
        team,
    ));
}

fn create_inhibitor(commands: &mut Commands, team: Team, position: Lane) {
    let position = match team {
        Team::Red => match position {
            Lane::Top => INHIBITOR_RED_TOP_POSITION,
            Lane::Mid => INHIBITOR_RED_MID_POSITION,
            Lane::Bottom => INHIBITOR_RED_BOTTOM_POSITION,
        },
        Team::Blue => match position {
            Lane::Top => INHIBITOR_BLUE_TOP_POSITION,
            Lane::Mid => INHIBITOR_BLUE_MID_POSITION,
            Lane::Bottom => INHIBITOR_BLUE_BOTTOM_POSITION,
        },
    };
    commands.spawn((Inhibitor, Transform::from_translation(position), team));
}

fn create_first_turret(commands: &mut Commands, team: Team, position: Lane) {
    let position = match team {
        Team::Red => match position {
            Lane::Top => TURRET_RED_FIRST_TOP_POSITION,
            Lane::Mid => TURRET_RED_FIRST_MID_POSITION,
            Lane::Bottom => TURRET_RED_FIRST_BOTTOM_POSITION,
        },
        Team::Blue => match position {
            Lane::Top => TURRET_BLUE_FIRST_TOP_POSITION,
            Lane::Mid => TURRET_BLUE_FIRST_MID_POSITION,
            Lane::Bottom => TURRET_BLUE_FIRST_BOTTOM_POSITION,
        },
    };
    commands.spawn((
        Turret,
        Health {
            value: TURRET_FIRST_HEALTH,
            max: TURRET_FIRST_HEALTH,
        },
        Transform::from_translation(position),
        team,
    ));
}

fn create_second_turret(commands: &mut Commands, team: Team, position: Lane) {
    let position = match team {
        Team::Red => match position {
            Lane::Top => TURRET_RED_SECOND_TOP_POSITION,
            Lane::Mid => TURRET_RED_SECOND_MID_POSITION,
            Lane::Bottom => TURRET_RED_SECOND_BOTTOM_POSITION,
        },
        Team::Blue => match position {
            Lane::Top => TURRET_BLUE_SECOND_TOP_POSITION,
            Lane::Mid => TURRET_BLUE_SECOND_MID_POSITION,
            Lane::Bottom => TURRET_BLUE_SECOND_BOTTOM_POSITION,
        },
    };
    commands.spawn((
        Turret,
        Health {
            value: TURRET_SECOND_HEALTH,
            max: TURRET_SECOND_HEALTH,
        },
        Transform::from_translation(position),
        team,
    ));
}

fn create_third_turret(commands: &mut Commands, team: Team, position: Lane) {
    let position = match team {
        Team::Red => match position {
            Lane::Top => TURRET_RED_THIRD_TOP_POSITION,
            Lane::Mid => TURRET_RED_THIRD_MID_POSITION,
            Lane::Bottom => TURRET_RED_THIRD_BOTTOM_POSITION,
        },
        Team::Blue => match position {
            Lane::Top => TURRET_BLUE_THIRD_TOP_POSITION,
            Lane::Mid => TURRET_BLUE_THIRD_MID_POSITION,
            Lane::Bottom => TURRET_BLUE_THIRD_BOTTOM_POSITION,
        },
    };
    commands.spawn((
        Turret,
        Health {
            value: TURRET_THIRD_HEALTH,
            max: TURRET_THIRD_HEALTH,
        },
        Transform::from_translation(position),
        team,
    ));
}
