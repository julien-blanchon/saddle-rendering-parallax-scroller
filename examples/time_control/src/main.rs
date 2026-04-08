//! Time control — demonstrates `ParallaxTimeScale` for pause, slow-mo, and speed-up.
//!
//! Controls:
//! - **Up/Down arrows**: adjust time scale (0.0 to 3.0)
//! - **Space**: toggle pause (0.0 ↔ previous value)
//! - **1/2/3**: preset speeds (0.25×, 1.0×, 2.0×)

use bevy::prelude::*;

use common::{configure_app, demo_textures};
use saddle_pane::prelude::*;
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerStrategy, ParallaxRig,
    ParallaxSegmented, ParallaxTimeScale,
};
use saddle_rendering_parallax_scroller_example_common as common;

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.init_resource::<PauseMemory>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, time_control_input);
    app.run();
}

#[derive(Resource, Default)]
struct PauseMemory {
    saved_scale: f32,
}

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Time Control", position = "top-right")]
struct TimePane {
    #[pane(slider, min = 0.0, max = 3.0, step = 0.05)]
    time_scale: f32,
    #[pane(monitor)]
    paused: f32,
}

impl Default for TimePane {
    fn default() -> Self {
        Self {
            time_scale: 1.0,
            paused: 0.0,
        }
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);

    commands.insert_resource(TimePane::default());

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
            ParallaxRig::default(),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Sky — tiled, auto-scrolling slowly
    commands.spawn((
        Name::new("Sky"),
        ChildOf(rig),
        Sprite::from_image(textures.sky.clone()),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::ONE)
            .with_auto_scroll(Vec2::new(-10.0, 0.0))
            .with_repeat(ParallaxAxes::both())
            .with_coverage_margin(Vec2::new(96.0, 48.0))
            .with_scale(Vec2::splat(2.0))
            .with_origin(Vec2::new(0.0, 24.0))
            .with_depth(0.0),
    ));

    // Mountains — auto-scrolling
    commands.spawn((
        Name::new("Mountains"),
        ChildOf(rig),
        Sprite::from_image(textures.mountains.clone()),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::ZERO)
            .with_auto_scroll(Vec2::new(-40.0, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -96.0))
            .with_depth(1.0)
            .with_scale(Vec2::splat(1.4))
            .with_tint(Color::srgb(0.34, 0.47, 0.56))
            .with_source_size(Vec2::new(320.0, 96.0)),
    ));

    // Canopy — fastest auto-scroll
    commands.spawn((
        Name::new("Canopy"),
        ChildOf(rig),
        Sprite::from_image(textures.canopy.clone()),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::ZERO)
            .with_auto_scroll(Vec2::new(-90.0, 0.0))
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

fn time_control_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut time_scale: ResMut<ParallaxTimeScale>,
    mut memory: ResMut<PauseMemory>,
    mut pane: ResMut<TimePane>,
) {
    let step = 0.1;

    if keys.just_pressed(KeyCode::ArrowUp) {
        time_scale.0 = (time_scale.0 + step).min(3.0);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        time_scale.0 = (time_scale.0 - step).max(0.0);
    }
    if keys.just_pressed(KeyCode::Space) {
        if time_scale.0 > 0.0 {
            memory.saved_scale = time_scale.0;
            time_scale.0 = 0.0;
        } else {
            time_scale.0 = if memory.saved_scale > 0.0 {
                memory.saved_scale
            } else {
                1.0
            };
        }
    }
    if keys.just_pressed(KeyCode::Digit1) {
        time_scale.0 = 0.25;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        time_scale.0 = 1.0;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        time_scale.0 = 2.0;
    }

    // Sync pane from time_scale resource (pane can also drive it)
    if pane.time_scale != time_scale.0 {
        if (pane.time_scale - time_scale.0).abs() > 0.001 {
            // Keyboard changed it
            pane.time_scale = time_scale.0;
        } else {
            // Pane slider changed it
            time_scale.0 = pane.time_scale;
        }
    }
    pane.paused = if time_scale.0 == 0.0 { 1.0 } else { 0.0 };
}
