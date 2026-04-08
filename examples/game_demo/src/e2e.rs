use bevy::prelude::*;
use saddle_bevy_e2e::{E2EPlugin, E2ESet, action::Action, init_scenario, scenario::Scenario};
use saddle_rendering_parallax_scroller::ParallaxScrollerSystems;

use crate::Player;

pub struct GameDemoE2EPlugin;

impl Plugin for GameDemoE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(Update, E2ESet.before(ParallaxScrollerSystems::TrackCamera));

        let args: Vec<String> = std::env::args().collect();
        let scenario_name = args.get(1).cloned();

        if let Some(name) = scenario_name {
            if !name.starts_with('-') {
                if let Some(scenario) = scenario_by_name(&name) {
                    init_scenario(app, scenario);
                } else {
                    error!(
                        "[game_demo:e2e] Unknown scenario '{name}'. Available: {:?}",
                        list_scenarios()
                    );
                }
            }
        }
    }
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec!["game_demo_playthrough"]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "game_demo_playthrough" => Some(playthrough()),
        _ => None,
    }
}

/// Move the player rightward by setting velocity directly.
fn push_player_right(world: &mut World, speed: f32) {
    let mut query = world.query::<&mut Player>();
    for mut player in query.iter_mut(world) {
        player.velocity.x = speed;
        player.facing = 1.0;
    }
}

/// Make the player jump.
fn make_player_jump(world: &mut World) {
    let mut query = world.query::<&mut Player>();
    for mut player in query.iter_mut(world) {
        if player.grounded {
            player.velocity.y = 450.0;
            player.grounded = false;
        }
    }
}

/// Stop horizontal movement.
fn stop_player(world: &mut World) {
    let mut query = world.query::<&mut Player>();
    for mut player in query.iter_mut(world) {
        player.velocity.x = 0.0;
    }
}

/// Full playthrough: walk right, jump, walk more, take screenshots throughout.
fn playthrough() -> Scenario {
    Scenario::builder("game_demo_playthrough")
        .description(
            "Automated playthrough: move player right, jump, showcase parallax \
             depth effect across all 8 layers with camera lead.",
        )
        // Let the scene initialize and settle
        .then(Action::WaitFrames(30))
        .then(Action::Screenshot("01_start".into()))
        .then(Action::WaitFrames(1))

        // Walk right at normal speed
        .then(Action::Custom(Box::new(|world| {
            push_player_right(world, 280.0);
        })))
        .then(Action::WaitFrames(120)) // ~2 seconds of walking
        .then(Action::Screenshot("02_walking_right".into()))
        .then(Action::WaitFrames(1))

        // Jump while walking
        .then(Action::Custom(Box::new(|world| {
            make_player_jump(world);
        })))
        .then(Action::WaitFrames(20)) // mid-air
        .then(Action::Screenshot("03_mid_jump".into()))
        .then(Action::WaitFrames(1))

        // Keep walking, wait for landing
        .then(Action::WaitFrames(40))
        .then(Action::Screenshot("04_after_landing".into()))
        .then(Action::WaitFrames(1))

        // Run fast (2x speed)
        .then(Action::Custom(Box::new(|world| {
            push_player_right(world, 280.0 * 1.8);
        })))
        .then(Action::WaitFrames(150)) // ~2.5 seconds of running
        .then(Action::Screenshot("05_running_far".into()))
        .then(Action::WaitFrames(1))

        // Jump again while running
        .then(Action::Custom(Box::new(|world| {
            make_player_jump(world);
        })))
        .then(Action::WaitFrames(15))
        .then(Action::Screenshot("06_run_jump".into()))
        .then(Action::WaitFrames(1))

        // Continue and let parallax settle
        .then(Action::WaitFrames(60))

        // Stop and let camera catch up
        .then(Action::Custom(Box::new(|world| {
            stop_player(world);
        })))
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("07_stopped_final".into()))
        .then(Action::WaitFrames(1))

        // Walk left briefly to show direction change + camera lead flip
        .then(Action::Custom(Box::new(|world| {
            let mut query = world.query::<&mut Player>();
            for mut player in query.iter_mut(world) {
                player.velocity.x = -280.0;
                player.facing = -1.0;
            }
        })))
        .then(Action::WaitFrames(120))
        .then(Action::Screenshot("08_walking_left".into()))
        .then(Action::WaitFrames(1))

        .then(Action::Custom(Box::new(|world| {
            stop_player(world);
        })))
        .then(Action::WaitFrames(30))
        .then(Action::Screenshot("09_final".into()))
        .then(Action::WaitFrames(1))
        .build()
}
