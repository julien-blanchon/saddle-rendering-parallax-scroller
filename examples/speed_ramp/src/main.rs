//! Speed ramp — demonstrates `ParallaxRig::speed_multiplier` for endless-runner difficulty ramp.
//!
//! The scene auto-scrolls and gradually accelerates. The speed multiplier
//! increases every few seconds, affecting only auto-scroll rates while keeping
//! the spatial parallax ratios unchanged.
//!
//! Controls:
//! - **Up/Down arrows**: manually adjust speed multiplier
//! - **R**: reset to 1.0×
//! - **Space**: toggle pause (speed_multiplier = 0)

use bevy::prelude::*;

use common::{configure_app, demo_textures};
use saddle_pane::prelude::*;
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerStrategy, ParallaxRig,
    ParallaxSegmented,
};
use saddle_rendering_parallax_scroller_example_common as common;

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.init_resource::<RampState>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, (speed_ramp_input, auto_ramp));
    app.run();
}

#[derive(Component)]
struct SpeedRig;

#[derive(Resource)]
struct RampState {
    auto_ramp: bool,
    ramp_timer: f32,
    ramp_interval: f32,
    ramp_increment: f32,
    paused_multiplier: f32,
}

impl Default for RampState {
    fn default() -> Self {
        Self {
            auto_ramp: true,
            ramp_timer: 0.0,
            ramp_interval: 3.0,
            ramp_increment: 0.25,
            paused_multiplier: 0.0,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Speed Ramp", position = "top-right")]
struct SpeedPane {
    #[pane(slider, min = 0.0, max = 5.0, step = 0.05)]
    speed_multiplier: f32,
    #[pane(monitor)]
    distance: f32,
    #[pane(monitor)]
    auto_ramp: f32,
}

impl Default for SpeedPane {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            distance: 0.0,
            auto_ramp: 1.0,
        }
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);

    commands.insert_resource(SpeedPane::default());

    let camera = commands
        .spawn((
            Name::new("Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            Transform::default(),
        ))
        .id();

    let rig = commands
        .spawn((
            Name::new("Parallax Rig"),
            SpeedRig,
            ParallaxRig::default().with_speed_multiplier(1.0),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Sky — slow auto-scroll
    commands.spawn((
        Name::new("Sky"),
        ChildOf(rig),
        Sprite::from_image(textures.sky.clone()),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::ONE)
            .with_auto_scroll(Vec2::new(-15.0, 0.0))
            .with_repeat(ParallaxAxes::both())
            .with_coverage_margin(Vec2::new(96.0, 48.0))
            .with_scale(Vec2::splat(2.0))
            .with_origin(Vec2::new(0.0, 24.0))
            .with_depth(0.0),
    ));

    // Mountains — medium auto-scroll
    commands.spawn((
        Name::new("Mountains"),
        ChildOf(rig),
        Sprite::from_image(textures.mountains.clone()),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::ZERO)
            .with_auto_scroll(Vec2::new(-50.0, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -96.0))
            .with_depth(1.0)
            .with_scale(Vec2::splat(1.4))
            .with_tint(Color::srgb(0.34, 0.47, 0.56))
            .with_source_size(Vec2::new(320.0, 96.0)),
    ));

    // Canopy — fast auto-scroll
    commands.spawn((
        Name::new("Canopy"),
        ChildOf(rig),
        Sprite::from_image(textures.canopy.clone()),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::ZERO)
            .with_auto_scroll(Vec2::new(-120.0, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -200.0))
            .with_depth(2.0)
            .with_scale(Vec2::new(1.5, 2.0))
            .with_tint(Color::srgb(0.14, 0.28, 0.14))
            .with_source_size(Vec2::new(256.0, 64.0))
            .with_strategy(ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(2, 0),
            })),
    ));
}

fn speed_ramp_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut rigs: Query<&mut ParallaxRig, With<SpeedRig>>,
    mut state: ResMut<RampState>,
    mut pane: ResMut<SpeedPane>,
) {
    let step = 0.25;

    if keys.just_pressed(KeyCode::ArrowUp) {
        state.auto_ramp = false;
        for mut rig in &mut rigs {
            rig.speed_multiplier = (rig.speed_multiplier + step).min(5.0);
        }
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        state.auto_ramp = false;
        for mut rig in &mut rigs {
            rig.speed_multiplier = (rig.speed_multiplier - step).max(0.0);
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        state.auto_ramp = true;
        state.ramp_timer = 0.0;
        for mut rig in &mut rigs {
            rig.speed_multiplier = 1.0;
        }
    }
    if keys.just_pressed(KeyCode::Space) {
        for mut rig in &mut rigs {
            if rig.speed_multiplier > 0.0 {
                state.paused_multiplier = rig.speed_multiplier;
                rig.speed_multiplier = 0.0;
            } else {
                rig.speed_multiplier = if state.paused_multiplier > 0.0 {
                    state.paused_multiplier
                } else {
                    1.0
                };
            }
        }
    }

    // Sync pane
    if let Ok(rig) = rigs.single() {
        if (pane.speed_multiplier - rig.speed_multiplier).abs() > 0.001 {
            pane.speed_multiplier = rig.speed_multiplier;
        }
    }
    pane.auto_ramp = if state.auto_ramp { 1.0 } else { 0.0 };
}

fn auto_ramp(
    time: Res<Time>,
    mut state: ResMut<RampState>,
    mut rigs: Query<&mut ParallaxRig, With<SpeedRig>>,
    mut pane: ResMut<SpeedPane>,
) {
    if !state.auto_ramp {
        return;
    }

    state.ramp_timer += time.delta_secs();
    if state.ramp_timer >= state.ramp_interval {
        state.ramp_timer -= state.ramp_interval;
        for mut rig in &mut rigs {
            rig.speed_multiplier = (rig.speed_multiplier + state.ramp_increment).min(5.0);
        }
    }

    // Update distance counter
    pane.distance += time.delta_secs() * 120.0 * rigs.single().map_or(1.0, |r| r.speed_multiplier);
}
