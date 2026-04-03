use bevy::{
    asset::AssetPlugin, ecs::schedule::ScheduleLabel, prelude::*, transform::TransformPlugin,
};

use super::*;
use crate::{
    ParallaxCameraTarget, ParallaxDepthMapping, ParallaxLayer, ParallaxLayerStrategy,
    ParallaxScrollerPlugin, ParallaxSegmented,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

fn test_app() -> App {
    let mut app = App::new();
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.init_schedule(Tick);
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin));
    app.init_asset::<Image>();
    app.add_plugins(ParallaxScrollerPlugin::new(Activate, Deactivate, Tick));
    app
}

fn test_image() -> Image {
    Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: 32,
            height: 16,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &[255, 255, 255, 255],
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    )
}

fn segment_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<ManagedSegment>>();
    query.iter(world).count()
}

#[test]
fn segmented_layers_do_not_duplicate_children_on_repeated_updates() {
    let mut app = test_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(test_image())
    };

    let rig = app
        .world_mut()
        .spawn((Name::new("Rig"), ParallaxRig::default()))
        .id();
    app.world_mut().spawn((
        Name::new("Layer"),
        ChildOf(rig),
        Sprite::from_image(image),
        ParallaxLayer {
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented::default()),
            source_size: Some(Vec2::new(64.0, 32.0)),
            ..default()
        },
    ));

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);
    let first_count = segment_count(&mut app);
    app.world_mut().run_schedule(Tick);

    let child_count = segment_count(&mut app);
    assert!(first_count > 0);
    assert_eq!(child_count, first_count);
}

#[test]
fn deactivate_schedule_cleans_generated_segments() {
    let mut app = test_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(test_image())
    };
    let rig = app.world_mut().spawn(ParallaxRig::default()).id();
    app.world_mut().spawn((
        ChildOf(rig),
        Sprite::from_image(image),
        ParallaxLayer {
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented::default()),
            source_size: Some(Vec2::new(64.0, 32.0)),
            ..default()
        },
    ));

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);
    assert!(segment_count(&mut app) > 0);

    app.world_mut().run_schedule(Deactivate);
    assert_eq!(segment_count(&mut app), 0);
}

#[test]
fn perspective_depth_mapping_changes_offset_and_scale_from_camera_depth() {
    let mut app = test_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(test_image())
    };

    let camera = app
        .world_mut()
        .spawn((
            Camera::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            Transform::from_xyz(10.0, 0.0, 12.0),
            GlobalTransform::from(Transform::from_xyz(10.0, 0.0, 12.0)),
        ))
        .id();
    let rig = app
        .world_mut()
        .spawn((ParallaxRig::default(), ParallaxCameraTarget::new(camera)))
        .id();
    let layer = app
        .world_mut()
        .spawn((
            ChildOf(rig),
            Sprite::from_image(image),
            Transform::default(),
            ParallaxLayer {
                camera_factor: Vec2::ZERO,
                depth: -8.0,
                depth_mapping: Some(ParallaxDepthMapping::default()),
                source_size: Some(Vec2::new(100.0, 50.0)),
                ..default()
            },
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let transform = app
        .world()
        .get::<Transform>(layer)
        .expect("layer transform should exist");
    assert!((transform.translation.x - 4.0).abs() < 0.001);
    assert!((transform.scale.x - 0.6).abs() < 0.001);
    assert!((transform.scale.y - 0.6).abs() < 0.001);
}
