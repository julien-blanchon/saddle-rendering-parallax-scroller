//! Basic parallax scrolling — a three-layer forest scene with sky, mountains, and canopy.
//!
//! Demonstrates:
//! - Creating a `ParallaxRig` and attaching it to a camera via `ParallaxCameraTarget`
//! - Adding multiple `ParallaxLayer`s with different `camera_factor` scroll speeds
//! - Using both tiled and segmented layer strategies
//! - Controlling layer depth ordering, scale, tint, and origin offsets

use bevy::prelude::*;

use common::{DemoCamera, configure_app, demo_textures, update_demo_camera};
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerStrategy, ParallaxRig,
    ParallaxSegmented,
};
use saddle_rendering_parallax_scroller_example_common as common;

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, update_demo_camera);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);

    // --- Camera with slow horizontal drift + vertical oscillation ---
    let camera = commands
        .spawn((
            Name::new("Demo Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            DemoCamera {
                horizontal_speed: 120.0,
                vertical_amplitude: 24.0,
                zoom_amplitude: 0.22,
            },
            Transform::default(),
        ))
        .id();

    // --- Parallax rig at world origin, tracking the camera ---
    let rig = commands
        .spawn((
            Name::new("Forest Parallax Rig"),
            ParallaxRig {
                enabled: true,
                origin: Vec2::ZERO,
                ..default()
            },
            Transform::from_translation(Vec3::ZERO),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Layer 1: Sky — tiled both axes, scroll factor 1:1, pastel gradient texture
    commands.spawn((
        Name::new("Sky Layer"),
        ChildOf(rig),
        ParallaxLayer {
            enabled: true,
            camera_factor: Vec2::ONE,
            auto_scroll: Vec2::ZERO,
            repeat: ParallaxAxes::both(),
            coverage_margin: Vec2::new(96.0, 48.0),
            tint: Color::srgba(0.95, 0.98, 1.0, 0.92),
            scale: Vec2::splat(2.0),
            origin: Vec2::new(0.0, 24.0),
            depth: 0.0,
            ..default()
        },
        Sprite::from_image(textures.sky.clone()),
    ));

    // Layer 2: Mountains — segmented strategy, scrolls slower than camera (factor 0.84)
    commands.spawn((
        Name::new("Mountain Layer"),
        ChildOf(rig),
        ParallaxLayer {
            enabled: true,
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented::default()),
            camera_factor: Vec2::new(0.84, 1.0),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -96.0),
            depth: 1.0,
            scale: Vec2::splat(1.4),
            tint: Color::srgb(0.34, 0.47, 0.56),
            source_size: Some(Vec2::new(320.0, 96.0)),
            ..default()
        },
        Sprite::from_image(textures.mountains.clone()),
    ));

    // Layer 3: Canopy — segmented with extra rings, scrolls faster than camera (factor 1.08)
    commands.spawn((
        Name::new("Canopy Layer"),
        ChildOf(rig),
        ParallaxLayer {
            enabled: true,
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(2, 0),
            }),
            camera_factor: Vec2::new(1.08, 1.0),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -200.0),
            depth: 2.0,
            scale: Vec2::new(1.5, 2.0),
            tint: Color::srgb(0.14, 0.28, 0.14),
            source_size: Some(Vec2::new(256.0, 64.0)),
            ..default()
        },
        Sprite::from_image(textures.canopy.clone()),
    ));
}
