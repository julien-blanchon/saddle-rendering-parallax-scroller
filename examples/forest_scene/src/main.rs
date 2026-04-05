//! Rich forest parallax scene — 7-layer forest with atmospheric perspective.
//!
//! Demonstrates:
//! - Multi-layer parallax with atmospheric depth (far layers are lighter/bluer)
//! - Mountain silhouettes, tree lines at varying distances, and ground layer
//! - Procedurally generated textures that create a convincing forest scene
//! - Different scroll speeds per layer creating depth illusion
//!
//! Controls:
//! - Camera auto-scrolls horizontally with vertical oscillation
//! - Use the saddle-pane UI (top-right) to adjust speed and amplitude

use bevy::prelude::*;

use common::{DemoCamera, add_rich_forest_stack, configure_app, demo_textures, update_demo_camera};
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
            Name::new("Forest Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            DemoCamera {
                horizontal_speed: 80.0,
                vertical_amplitude: 30.0,
                zoom_amplitude: 0.15,
            },
            Transform::default(),
        ))
        .id();

    // Parallax rig
    let rig = commands
        .spawn((
            Name::new("Forest Parallax Rig"),
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

    // Spawn the full 7-layer forest stack
    add_rich_forest_stack(&mut commands, rig, &textures);

    // On-screen instructions
    commands.spawn((
        Name::new("Instructions"),
        Text::new("Forest Parallax — 7 layers with atmospheric perspective\nAdjust speed in the Parallax pane (top-right)"),
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
