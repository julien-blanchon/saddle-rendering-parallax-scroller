//! City skyline parallax scene — urban dusk panorama with lit windows.
//!
//! Demonstrates:
//! - Multi-layer city parallax with atmospheric depth at dusk
//! - Building silhouettes at different distances with varying detail
//! - Lit windows on near buildings, pure silhouettes on far buildings
//! - Warm dusk sky gradient with procedural cloud wisps
//!
//! Controls:
//! - Camera auto-scrolls horizontally with subtle vertical drift
//! - Use the saddle-pane UI (top-right) to adjust speed and amplitude

use bevy::prelude::*;

use common::{DemoCamera, add_city_stack, configure_app, demo_textures, update_demo_camera};
use saddle_rendering_parallax_scroller::{ParallaxCameraTarget, ParallaxRig, ParallaxRigBundle};
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

    // Camera with slow drift
    let camera = commands
        .spawn((
            Name::new("City Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            DemoCamera {
                horizontal_speed: 60.0,
                vertical_amplitude: 15.0,
                zoom_amplitude: 0.10,
            },
            Transform::default(),
        ))
        .id();

    // Parallax rig
    let rig = commands
        .spawn((
            Name::new("City Parallax Rig"),
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

    // Spawn the full city stack
    add_city_stack(&mut commands, rig, &textures);

    // On-screen instructions
    commands.spawn((
        Name::new("Instructions"),
        Text::new("City Skyline — dusk panorama with lit windows\nAdjust speed in the Parallax pane (top-right)"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));
}
