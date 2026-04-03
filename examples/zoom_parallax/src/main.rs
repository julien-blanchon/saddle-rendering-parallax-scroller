use saddle_rendering_parallax_scroller_example_common as common;

use bevy::prelude::*;

use common::{configure_app, demo_textures, spawn_demo_rig, spawn_tiled_layer};
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxDepthMapping, ParallaxLayer, ParallaxLayerStrategy, ParallaxSegmented,
};

#[derive(Component)]
struct DollyZoomCamera {
    horizontal_speed: f32,
    base_depth: f32,
    zoom_amplitude: f32,
    vertical_bob: f32,
}

#[derive(Component)]
struct RunnerMarker;

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.insert_resource(ClearColor(Color::srgb(0.07, 0.08, 0.12)));
    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_camera, animate_runner_marker));
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);
    let camera = commands
        .spawn((
            Name::new("Dolly Zoom Camera"),
            Camera2d,
            Projection::Perspective(PerspectiveProjection {
                fov: std::f32::consts::FRAC_PI_4,
                near: 0.1,
                far: 2000.0,
                ..default()
            }),
            DollyZoomCamera {
                horizontal_speed: 96.0,
                base_depth: 12.0,
                zoom_amplitude: 3.0,
                vertical_bob: 18.0,
            },
            Transform::from_xyz(0.0, 0.0, 12.0),
        ))
        .id();

    let rig = spawn_demo_rig(&mut commands, camera, "Depth Mapped Rig", Vec3::ZERO);
    let depth_mapping = ParallaxDepthMapping {
        reference_plane_z: 0.0,
        translation_response: Vec2::new(1.0, 0.12),
        scale_response: 1.0,
    };

    spawn_tiled_layer(
        &mut commands,
        rig,
        "Locked Horizon",
        textures.sky.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::ONE)
            .with_repeat(ParallaxAxes::both())
            .with_scale(Vec2::splat(3.0))
            .with_coverage_margin(Vec2::new(140.0, 120.0))
            .with_origin(Vec2::new(0.0, 80.0))
            .with_tint(Color::srgba(1.0, 0.93, 0.84, 0.92)),
    );

    spawn_tiled_layer(
        &mut commands,
        rig,
        "Far Canyon Cards",
        textures.vista.clone(),
        ParallaxLayer {
            camera_factor: Vec2::ZERO,
            depth: -10.0,
            depth_mapping: Some(depth_mapping.clone()),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -30.0),
            scale: Vec2::new(1.6, 1.3),
            tint: Color::srgb(0.42, 0.40, 0.52),
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(2, 0),
            }),
            source_size: Some(Vec2::new(640.0, 220.0)),
            ..default()
        },
    );

    spawn_tiled_layer(
        &mut commands,
        rig,
        "Mid Cliffs",
        textures.mountains.clone(),
        ParallaxLayer {
            camera_factor: Vec2::ZERO,
            depth: -4.0,
            depth_mapping: Some(depth_mapping.clone()),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -132.0),
            scale: Vec2::new(1.7, 2.6),
            tint: Color::srgb(0.24, 0.28, 0.35),
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(2, 0),
            }),
            source_size: Some(Vec2::new(320.0, 96.0)),
            ..default()
        },
    );

    spawn_tiled_layer(
        &mut commands,
        rig,
        "Foreground Boughs",
        textures.canopy.clone(),
        ParallaxLayer {
            camera_factor: Vec2::ZERO,
            depth: 4.0,
            depth_mapping: Some(ParallaxDepthMapping {
                translation_response: Vec2::new(1.0, 0.0),
                ..depth_mapping
            }),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -188.0),
            scale: Vec2::new(1.4, 2.4),
            tint: Color::srgba(0.12, 0.16, 0.12, 0.94),
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(2, 0),
            }),
            source_size: Some(Vec2::new(256.0, 64.0)),
            ..default()
        },
    );

    commands.spawn((
        Name::new("Runner Ground"),
        Sprite::from_color(Color::srgb(0.15, 0.11, 0.12), Vec2::new(2400.0, 28.0)),
        Transform::from_xyz(0.0, -220.0, 0.05),
    ));
    commands.spawn((
        Name::new("Runner Marker"),
        RunnerMarker,
        Sprite::from_color(Color::srgb(0.95, 0.58, 0.28), Vec2::new(36.0, 64.0)),
        Transform::from_xyz(-120.0, -182.0, 0.08),
    ));
    commands.spawn((
        Name::new("Guide Lights"),
        Sprite::from_color(Color::srgba(1.0, 0.84, 0.42, 0.7), Vec2::new(520.0, 8.0)),
        Transform::from_xyz(40.0, -154.0, 0.04),
    ));
}

fn animate_camera(time: Res<Time>, mut cameras: Query<(&DollyZoomCamera, &mut Transform)>) {
    let seconds = time.elapsed_secs();
    for (controller, mut transform) in &mut cameras {
        transform.translation.x = seconds * controller.horizontal_speed;
        transform.translation.y = (seconds * 0.55).sin() * controller.vertical_bob;
        transform.translation.z =
            controller.base_depth + (seconds * 0.35).sin() * controller.zoom_amplitude;
    }
}

fn animate_runner_marker(time: Res<Time>, mut markers: Query<&mut Transform, With<RunnerMarker>>) {
    let seconds = time.elapsed_secs();
    for mut transform in &mut markers {
        transform.translation.y = -182.0 + (seconds * 6.0).sin().abs() * 8.0;
    }
}
