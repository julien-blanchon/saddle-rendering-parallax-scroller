//! Finite-bounds parallax — a single non-repeating layer clamped to a horizontal range.
//!
//! Demonstrates:
//! - Using `ParallaxBounds::horizontal(min, max)` to clamp layer scroll range
//! - Disabling repeat (`ParallaxAxes::none()`) for a one-shot background
//! - A finite vista image that does not tile — it stops at the configured bounds

use bevy::prelude::*;

use common::{DemoCamera, configure_app, demo_textures, update_demo_camera};
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxBounds, ParallaxCameraTarget, ParallaxLayer, ParallaxRig,
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

    // --- Camera ---
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

    // --- Rig ---
    let rig = commands
        .spawn((
            Name::new("Finite Bounds Rig"),
            ParallaxRig {
                enabled: true,
                origin: Vec2::ZERO,
                ..default()
            },
            Transform::from_translation(Vec3::ZERO),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Single layer: Finite vista — no repeat, clamped to [-160, 160] horizontal bounds
    // The layer scrolls at 0.88x camera speed but stops at the edges instead of wrapping.
    commands.spawn((
        Name::new("Finite Vista"),
        ChildOf(rig),
        ParallaxLayer {
            enabled: true,
            repeat: ParallaxAxes::none(),
            bounds: ParallaxBounds::horizontal(-160.0, 160.0),
            camera_factor: Vec2::new(0.88, 1.0),
            origin: Vec2::new(0.0, -40.0),
            scale: Vec2::ONE,
            tint: Color::WHITE,
            source_size: Some(Vec2::new(640.0, 220.0)),
            depth: 0.0,
            ..default()
        },
        Sprite::from_image(textures.vista.clone()),
    ));
}
