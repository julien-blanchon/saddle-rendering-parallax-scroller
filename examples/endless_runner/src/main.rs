//! Endless runner — auto-scrolling city with increasing speed.
//!
//! Demonstrates:
//! - Combining `auto_scroll` on layers with camera-driven parallax
//! - Continuous horizontal scrolling creating an "endless" effect
//! - City skyline with multiple depth layers at different speeds
//! - Speed control via saddle-pane
//!
//! Controls:
//! - The scene auto-scrolls continuously
//! - Up/Down arrows: Adjust scroll speed
//! - Use the saddle-pane UI (top-right) to fine-tune parameters

use bevy::prelude::*;

use common::{configure_app, demo_textures};
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxRig,
};
use saddle_rendering_parallax_scroller_example_common as common;

#[derive(Resource)]
struct RunnerSpeed(f32);

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.insert_resource(RunnerSpeed(150.0));
    app.add_systems(Startup, setup);
    app.add_systems(Update, (speed_control, move_camera));
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);

    // Camera
    let camera = commands
        .spawn((
            Name::new("Runner Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            Transform::default(),
        ))
        .id();

    // Parallax rig
    let rig = commands
        .spawn((
            Name::new("Runner Parallax Rig"),
            ParallaxRig {
                enabled: true,
                origin: Vec2::ZERO,
                ..default()
            },
            Transform::from_translation(Vec3::ZERO),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Layer 0: Sky gradient with auto-scrolling clouds
    common::spawn_tiled_layer(
        &mut commands,
        rig,
        "City Sky",
        textures.city_sky.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::splat(0.05))
            .with_auto_scroll(Vec2::new(-5.0, 0.0))
            .with_repeat(ParallaxAxes::both())
            .with_coverage_margin(Vec2::new(200.0, 100.0))
            .with_scale(Vec2::splat(3.0)),
    );

    // Layer 1: Far buildings
    common::spawn_tiled_layer(
        &mut commands,
        rig,
        "Far Buildings",
        textures.city_far_buildings.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.20, 0.3))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -80.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 256.0)),
    );

    // Layer 2: Mid buildings
    common::spawn_tiled_layer(
        &mut commands,
        rig,
        "Mid Buildings",
        textures.city_mid_buildings.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.50, 0.6))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -100.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 256.0)),
    );

    // Layer 3: Near buildings with windows
    common::spawn_tiled_layer(
        &mut commands,
        rig,
        "Near Buildings",
        textures.city_near_buildings.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.80, 0.9))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -110.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 320.0)),
    );

    // Layer 4: Ground
    common::spawn_tiled_layer(
        &mut commands,
        rig,
        "Ground",
        textures.city_ground.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::new(1.0, 1.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -340.0))
            .with_scale(Vec2::new(2.0, 2.0))
            .with_coverage_margin(Vec2::new(200.0, 0.0)),
    );

    // On-screen instructions
    commands.spawn((
        Name::new("Instructions"),
        Text::new("Endless Runner — Up/Down arrows to change speed\nThe camera scrolls continuously through the city"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));
}

fn speed_control(keyboard: Res<ButtonInput<KeyCode>>, mut speed: ResMut<RunnerSpeed>) {
    if keyboard.pressed(KeyCode::ArrowUp) {
        speed.0 = (speed.0 + 2.0).min(500.0);
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        speed.0 = (speed.0 - 2.0).max(0.0);
    }
}

fn move_camera(
    time: Res<Time>,
    speed: Res<RunnerSpeed>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
) {
    for mut transform in &mut cameras {
        transform.translation.x += speed.0 * time.delta_secs();
    }
}
