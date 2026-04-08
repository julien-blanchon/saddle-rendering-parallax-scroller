//! Camera-follow parallax — a moving "character dot" with parallax layers that respond to camera
//! tracking the dot.
//!
//! Demonstrates:
//! - Parallax layers responding to a camera that follows a game object
//! - Combining `ParallaxCameraTarget` tracking with a custom follow system
//! - A "follow dot" entity the camera tracks, showing how parallax integrates
//!   with typical game camera follow patterns

use bevy::prelude::*;

use common::{DemoCamera, configure_app, demo_textures, update_demo_camera};
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerStrategy, ParallaxRig,
    ParallaxSegmented,
};
use saddle_rendering_parallax_scroller_example_common as common;

/// Marker for the entity the camera follows.
#[derive(Component)]
struct FollowDot;

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_follow_dot, update_demo_camera));
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);

    // --- Camera that drifts with the follow dot ---
    let camera = commands
        .spawn((
            Name::new("Follow Camera"),
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

    // --- Parallax rig ---
    let rig = commands
        .spawn((
            Name::new("Follow Rig"),
            ParallaxRig {
                enabled: true,
                origin: Vec2::ZERO,
                ..default()
            },
            Transform::from_translation(Vec3::ZERO),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Layer 1: Sky — 1:1 with camera, tiled both axes
    commands.spawn((
        Name::new("Sky Layer"),
        ChildOf(rig),
        ParallaxLayer {
            enabled: true,
            camera_factor: Vec2::ONE,
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

    // Layer 2: Mountains — slower scroll (0.84x), segmented
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

    // Layer 3: Canopy — faster scroll (1.08x) for foreground depth
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

    // --- Follow dot: an orange sprite the camera tracks ---
    commands.spawn((
        Name::new("Follow Dot"),
        FollowDot,
        Sprite::from_color(Color::srgb(0.96, 0.48, 0.22), Vec2::splat(24.0)),
        Transform::from_xyz(0.0, -250.0, 10.0),
    ));
}

/// Moves the follow dot along a sinusoidal path.
fn animate_follow_dot(time: Res<Time>, mut query: Query<&mut Transform, With<FollowDot>>) {
    for mut transform in &mut query {
        transform.translation.x = time.elapsed_secs() * 120.0;
        transform.translation.y = (time.elapsed_secs() * 1.6).sin() * 40.0;
    }
}
