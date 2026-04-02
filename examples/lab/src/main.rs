#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use saddle_rendering_parallax_scroller_example_common as common;

use std::fmt::Write as _;

use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use common::{
    FollowDot, PixelDrift, add_finite_vista, add_forest_stack, add_pixel_snap_pair,
    add_starfield_stack, demo_textures, install_auto_exit, spawn_demo_rig,
};
use saddle_rendering_parallax_scroller::{
    ParallaxDebugSettings, ParallaxDiagnostics, ParallaxScrollerPlugin,
};

const DEFAULT_BRP_PORT: u16 = 15_742;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabMode {
    Tight,
    Wide,
    PixelDrift,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct LabMotion {
    pub mode: LabMode,
    pub elapsed: f32,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct LabEntities {
    pub camera: Entity,
    pub forest_rig: Entity,
    pub vista_rig: Entity,
    pub pixel_rig: Entity,
    pub vista_layer: Entity,
    pub snapped_layer: Entity,
    pub unsnapped_layer: Entity,
    pub overlay: Entity,
}

#[derive(Component)]
struct LabOverlay;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.09)));
    app.insert_resource(LabMotion {
        mode: LabMode::Tight,
        elapsed: 0.0,
    });
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "saddle-rendering-parallax-scroller crate-local lab".into(),
            resolution: (1440, 900).into(),
            ..default()
        }),
        ..default()
    }));
    #[cfg(feature = "dev")]
    app.add_plugins(BrpExtrasPlugin::with_port(lab_brp_port()));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::ParallaxScrollerLabE2EPlugin);
    app.add_plugins(ParallaxScrollerPlugin::default());
    install_auto_exit(&mut app, "PARALLAX_SCROLLER_EXIT_AFTER_SECONDS");
    app.insert_resource(ParallaxDebugSettings {
        enabled: false,
        ..default()
    });
    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_lab_camera, update_overlay));
    app.run();
}

#[cfg(feature = "dev")]
fn lab_brp_port() -> u16 {
    std::env::var("BRP_EXTRAS_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_BRP_PORT)
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);
    let camera = commands
        .spawn((
            Name::new("Lab Camera"),
            Camera2d,
            PixelDrift,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            Transform::default(),
        ))
        .id();

    let forest_rig = spawn_demo_rig(&mut commands, camera, "Forest Rig", Vec3::ZERO);
    add_forest_stack(&mut commands, forest_rig, &textures);

    let vista_rig = spawn_demo_rig(
        &mut commands,
        camera,
        "Vista Rig",
        Vec3::new(0.0, -240.0, 0.0),
    );
    add_finite_vista(&mut commands, vista_rig, &textures);

    let pixel_rig = spawn_demo_rig(
        &mut commands,
        camera,
        "Pixel Rig",
        Vec3::new(0.0, 220.0, 0.0),
    );
    add_pixel_snap_pair(&mut commands, pixel_rig, &textures);
    add_starfield_stack(&mut commands, pixel_rig, &textures);

    let overlay = commands
        .spawn((
            Name::new("Lab Overlay"),
            LabOverlay,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(18.0),
                left: Val::Px(18.0),
                width: Val::Px(480.0),
                padding: UiRect::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.76)),
            Text::default(),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    // Resolve the two comparison layers by name once the hierarchy exists.
    commands.queue(move |world: &mut World| {
        let mut vista_layer = Entity::PLACEHOLDER;
        let mut snapped_layer = Entity::PLACEHOLDER;
        let mut unsnapped_layer = Entity::PLACEHOLDER;
        let mut names = world.query::<(Entity, &Name)>();
        for (entity, name) in names.iter(world) {
            if name.as_str() == "Finite Vista" {
                vista_layer = entity;
            } else if name.as_str() == "Snapped Clouds" {
                snapped_layer = entity;
            } else if name.as_str() == "Unsnapped Clouds" {
                unsnapped_layer = entity;
            }
        }

        world.insert_resource(LabEntities {
            camera,
            forest_rig,
            vista_rig,
            pixel_rig,
            vista_layer,
            snapped_layer,
            unsnapped_layer,
            overlay,
        });
    });

    commands.spawn((
        Name::new("Follow Marker"),
        FollowDot,
        Sprite::from_color(Color::srgb(0.96, 0.54, 0.22), Vec2::splat(22.0)),
        Transform::from_xyz(0.0, -320.0, 5.0),
    ));
}

pub fn set_lab_mode(world: &mut World, mode: LabMode) {
    let mut motion = world.resource_mut::<LabMotion>();
    motion.mode = mode;
}

fn animate_lab_camera(
    time: Res<Time>,
    mut motion: ResMut<LabMotion>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<PixelDrift>>,
    mut follow_markers: Query<&mut Transform, (With<FollowDot>, Without<PixelDrift>)>,
) {
    motion.elapsed += time.delta_secs();
    let seconds = motion.elapsed;

    for (mut transform, mut projection) in &mut cameras {
        match motion.mode {
            LabMode::Tight => {
                transform.translation.x = seconds * 72.0;
                transform.translation.y = (seconds * 0.7).sin() * 18.0;
                if let Projection::Orthographic(projection) = projection.as_mut() {
                    projection.scale = 1.0;
                }
            }
            LabMode::Wide => {
                transform.translation.x = seconds * 96.0;
                transform.translation.y = (seconds * 0.7).sin() * 26.0;
                if let Projection::Orthographic(projection) = projection.as_mut() {
                    projection.scale = 1.65;
                }
            }
            LabMode::PixelDrift => {
                transform.translation.x = seconds * 17.25;
                transform.translation.y = (seconds * 0.55).sin() * 2.25;
                if let Projection::Orthographic(projection) = projection.as_mut() {
                    projection.scale = 1.0;
                }
            }
        }
    }

    for mut transform in &mut follow_markers {
        transform.translation.x = seconds * 96.0;
        transform.translation.y = -320.0 + (seconds * 1.3).sin() * 22.0;
    }
}

fn update_overlay(
    diagnostics: Res<ParallaxDiagnostics>,
    motion: Res<LabMotion>,
    lab: Option<Res<LabEntities>>,
    mut query: Query<&mut Text, With<LabOverlay>>,
) {
    let Some(lab) = lab else {
        return;
    };

    for mut text in &mut query {
        let mut output = String::new();
        let _ = writeln!(output, "Parallax Scroller Lab");
        let _ = writeln!(output, "mode: {:?}", motion.mode);
        let _ = writeln!(output, "rigs: {}", diagnostics.rigs.len());

        if let Some(forest) = diagnostics
            .rigs
            .iter()
            .find(|rig| rig.rig == lab.forest_rig)
        {
            let _ = writeln!(
                output,
                "forest viewport: {:.0} x {:.0}",
                forest.viewport_size.x, forest.viewport_size.y
            );
            if let Some(layer) = forest.layers.first() {
                let _ = writeln!(
                    output,
                    "forest coverage: {:.0} x {:.0}",
                    layer.coverage_size.x, layer.coverage_size.y
                );
            }
        }

        if let Some(pixel) = diagnostics.rigs.iter().find(|rig| rig.rig == lab.pixel_rig) {
            let snapped = pixel
                .layers
                .iter()
                .find(|layer| layer.layer == lab.snapped_layer);
            let unsnapped = pixel
                .layers
                .iter()
                .find(|layer| layer.layer == lab.unsnapped_layer);
            if let (Some(snapped), Some(unsnapped)) = (snapped, unsnapped) {
                let _ = writeln!(
                    output,
                    "snap compare: {:.2} / {:.2}",
                    snapped.effective_offset.x, unsnapped.effective_offset.x
                );
            }
        }

        if let Some(vista) = diagnostics.rigs.iter().find(|rig| rig.rig == lab.vista_rig) {
            if let Some(layer) = vista
                .layers
                .iter()
                .find(|layer| layer.layer == lab.vista_layer)
            {
                let _ = writeln!(output, "vista x: {:.2}", layer.effective_offset.x);
            }
        }

        text.0 = output;
    }
}
