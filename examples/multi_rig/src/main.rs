//! Multi-rig parallax — two independent rigs stacked vertically, sharing one camera.
//!
//! Demonstrates:
//! - Spawning multiple `ParallaxRig` entities that share the same camera target
//! - Placing rigs at different world origins to partition the screen
//! - Different visual themes (forest vs. starfield) on separate rigs
//! - Each rig's layers scroll independently based on their own `camera_factor`

use bevy::prelude::*;

use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerBundle, ParallaxLayerStrategy,
    ParallaxRig, ParallaxRigBundle, ParallaxSegmented,
};
use saddle_rendering_parallax_scroller_example_common as common;
use common::{DemoCamera, configure_app, demo_textures, update_demo_camera};

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

    // -----------------------------------------------------------------------
    // Rig 1: Forest — positioned at y=0 (bottom half of screen)
    // -----------------------------------------------------------------------
    let forest = commands
        .spawn((
            Name::new("Forest Rig"),
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

    // Forest sky
    commands.spawn((
        Name::new("Forest Sky"),
        ChildOf(forest),
        ParallaxLayerBundle {
            layer: ParallaxLayer {
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
            sprite: Sprite::from_image(textures.sky.clone()),
            ..default()
        },
    ));

    // Forest mountains
    commands.spawn((
        Name::new("Forest Mountains"),
        ChildOf(forest),
        ParallaxLayerBundle {
            layer: ParallaxLayer {
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
            sprite: Sprite::from_image(textures.mountains.clone()),
            ..default()
        },
    ));

    // Forest canopy
    commands.spawn((
        Name::new("Forest Canopy"),
        ChildOf(forest),
        ParallaxLayerBundle {
            layer: ParallaxLayer {
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
            sprite: Sprite::from_image(textures.canopy.clone()),
            ..default()
        },
    ));

    // -----------------------------------------------------------------------
    // Rig 2: Space — positioned at y=260 (top half of screen)
    // -----------------------------------------------------------------------
    let space = commands
        .spawn((
            Name::new("Space Rig"),
            ParallaxRigBundle {
                rig: ParallaxRig {
                    enabled: true,
                    origin: Vec2::ZERO,
                },
                transform: Transform::from_translation(Vec3::new(0.0, 260.0, 0.0)),
                ..default()
            },
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Space starfield
    commands.spawn((
        Name::new("Starfield"),
        ChildOf(space),
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

    // Space cloud bands
    commands.spawn((
        Name::new("Cloud Bands"),
        ChildOf(space),
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
