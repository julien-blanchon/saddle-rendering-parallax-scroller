//! Pixel-art snap comparison — two cloud layers, one snapped to pixel grid and one unsnapped.
//!
//! Demonstrates:
//! - Using `ParallaxSnap::Pixel` to lock layer positions to whole-pixel boundaries
//! - Visual comparison: unsnapped (top, sub-pixel shimmer) vs snapped (bottom, crisp pixels)
//! - Configuring layers with the same texture and factor but different snap modes

use bevy::prelude::*;

use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerBundle, ParallaxRig,
    ParallaxRigBundle, ParallaxSnap,
};
use saddle_rendering_parallax_scroller_example_common as common;
use common::{configure_app, demo_textures};

/// Marker for the slowly-drifting pixel camera.
#[derive(Component)]
struct PixelDrift;

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, drift_pixel_camera);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);

    // --- Camera that drifts slowly at sub-pixel speeds to show the difference ---
    let camera = commands
        .spawn((
            Name::new("Pixel Drift Camera"),
            Camera2d,
            PixelDrift,
            Transform::default(),
        ))
        .id();

    // --- Rig ---
    let rig = commands
        .spawn((
            Name::new("Pixel Snap Rig"),
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

    // Layer 1: Unsnapped clouds (top) — sub-pixel positions cause shimmer in pixel art
    commands.spawn((
        Name::new("Unsnapped Clouds"),
        ChildOf(rig),
        ParallaxLayerBundle {
            layer: ParallaxLayer {
                enabled: true,
                camera_factor: Vec2::new(0.92, 1.0),
                repeat: ParallaxAxes::horizontal(),
                origin: Vec2::new(0.0, 72.0),
                scale: Vec2::splat(4.0),
                tint: Color::srgba(1.0, 1.0, 1.0, 0.92),
                snap: ParallaxSnap::None,
                depth: 0.0,
                ..default()
            },
            sprite: Sprite::from_image(textures.pixel_clouds.clone()),
            ..default()
        },
    ));

    // Layer 2: Snapped clouds (bottom) — locked to pixel grid, no shimmer
    commands.spawn((
        Name::new("Snapped Clouds"),
        ChildOf(rig),
        ParallaxLayerBundle {
            layer: ParallaxLayer {
                enabled: true,
                camera_factor: Vec2::new(0.92, 1.0),
                repeat: ParallaxAxes::horizontal(),
                origin: Vec2::new(0.0, -56.0),
                scale: Vec2::splat(4.0),
                tint: Color::srgba(1.0, 1.0, 1.0, 0.92),
                snap: ParallaxSnap::Pixel,
                depth: 1.0,
                ..default()
            },
            sprite: Sprite::from_image(textures.pixel_clouds.clone()),
            ..default()
        },
    ));
}

/// Drifts the camera slowly to make sub-pixel differences visible.
fn drift_pixel_camera(time: Res<Time>, mut query: Query<&mut Transform, With<PixelDrift>>) {
    for mut transform in &mut query {
        transform.translation.x = time.elapsed_secs() * 17.25;
        transform.translation.y = (time.elapsed_secs() * 0.7).sin() * 2.25;
    }
}
