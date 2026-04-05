//! Auto-scrolling starfield — layers that scroll automatically without camera movement.
//!
//! Demonstrates:
//! - Using `auto_scroll` on `ParallaxLayer` to create continuously moving backgrounds
//! - Layering multiple auto-scrolling layers at different speeds for depth
//! - Using `phase` to offset initial layer positions
//! - Combining auto-scroll with camera tracking

use bevy::prelude::*;

use common::{DemoCamera, configure_app, demo_textures, update_demo_camera};
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerBundle, ParallaxRig,
    ParallaxRigBundle,
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
            Name::new("Starfield Rig"),
            ParallaxRigBundle {
                rig: ParallaxRig {
                    enabled: true,
                    origin: Vec2::ZERO,
                },
                transform: Transform::from_translation(Vec3::ZERO),
                ..default()
            },
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Layer 1: Starfield — auto-scrolls diagonally, tiled both axes
    commands.spawn((
        Name::new("Starfield"),
        ChildOf(rig),
        ParallaxLayerBundle {
            layer: ParallaxLayer {
                enabled: true,
                camera_factor: Vec2::ONE,
                auto_scroll: Vec2::new(-12.0, -48.0),
                repeat: ParallaxAxes::both(),
                scale: Vec2::splat(2.0),
                coverage_margin: Vec2::new(80.0, 80.0),
                tint: Color::srgba(1.0, 1.0, 1.0, 0.95),
                depth: 0.0,
                ..default()
            },
            sprite: Sprite::from_image(textures.stars.clone()),
            ..default()
        },
    ));

    // Layer 2: Cloud bands — slower auto-scroll in different direction, with initial phase offset
    commands.spawn((
        Name::new("Cloud Bands"),
        ChildOf(rig),
        ParallaxLayerBundle {
            layer: ParallaxLayer {
                enabled: true,
                camera_factor: Vec2::ONE,
                auto_scroll: Vec2::new(18.0, -12.0),
                repeat: ParallaxAxes::both(),
                scale: Vec2::splat(3.0),
                tint: Color::srgba(0.68, 0.86, 1.0, 0.22),
                phase: Vec2::new(60.0, 40.0),
                depth: 1.0,
                ..default()
            },
            sprite: Sprite::from_image(textures.pixel_clouds.clone()),
            ..default()
        },
    ));
}
