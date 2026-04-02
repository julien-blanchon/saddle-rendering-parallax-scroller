use bevy::prelude::*;
use bevy_e2e::{E2EPlugin, E2ESet, action::Action, init_scenario};
use saddle_rendering_parallax_scroller::ParallaxScrollerSystems;

use crate::scenarios;

pub struct ParallaxScrollerLabE2EPlugin;

impl Plugin for ParallaxScrollerLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(Update, E2ESet.before(ParallaxScrollerSystems::TrackCamera));

        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = parse_e2e_args(&args);

        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenarios::scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                }
                init_scenario(app, scenario);
            } else {
                error!(
                    "[saddle_rendering_parallax_scroller_lab:e2e] Unknown scenario '{name}'. Available: {:?}",
                    scenarios::list_scenarios()
                );
            }
        }
    }
}

fn parse_e2e_args(args: &[String]) -> (Option<String>, bool) {
    let mut scenario_name = None;
    let mut handoff = false;

    for arg in args.iter().skip(1) {
        if arg == "--handoff" {
            handoff = true;
        } else if !arg.starts_with('-') && scenario_name.is_none() {
            scenario_name = Some(arg.clone());
        }
    }

    if !handoff {
        handoff = std::env::var("E2E_HANDOFF").is_ok_and(|value| value == "1" || value == "true");
    }

    (scenario_name, handoff)
}
