use bevy::{
    asset::AssetPlugin, camera::visibility::RenderLayers, ecs::schedule::ScheduleLabel, prelude::*,
    transform::TransformPlugin,
};

use saddle_rendering_parallax_scroller::{
    LayerRuntimeState, ParallaxCameraTarget, ParallaxDiagnostics, ParallaxLayer,
    ParallaxLayerComputed, ParallaxLayerStrategy, ParallaxRig, ParallaxScrollerPlugin,
    ParallaxScrollerSystems, ParallaxSegmented, ParallaxTimeScale, RigRuntimeState,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

fn app() -> App {
    let mut app = App::new();
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.init_schedule(Tick);
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin));
    app.init_asset::<Image>();
    app.configure_sets(
        Tick,
        ParallaxScrollerSystems::TrackCamera.before(ParallaxScrollerSystems::ComputeOffsets),
    );
    app.add_plugins(ParallaxScrollerPlugin::new(Activate, Deactivate, Tick));
    app
}

fn white_image() -> Image {
    Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &[255, 255, 255, 255],
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    )
}

fn child_entities(world: &mut World, parent: Entity) -> Vec<Entity> {
    let mut query = world.query::<(Entity, &ChildOf)>();
    query
        .iter(world)
        .filter_map(|(entity, child_of)| (child_of.parent() == parent).then_some(entity))
        .collect()
}

#[test]
fn plugin_supports_custom_schedules_and_publishes_diagnostics() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let camera = app
        .world_mut()
        .spawn((Name::new("Camera"), Camera2d, Camera::default()))
        .id();
    let rig = app
        .world_mut()
        .spawn((ParallaxRig::default(), ParallaxCameraTarget::new(camera)))
        .id();
    app.world_mut().spawn((
        ChildOf(rig),
        Sprite::from_image(image),
        ParallaxLayer::default().with_source_size(Vec2::new(16.0, 16.0)),
    ));

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let diagnostics = app.world().resource::<ParallaxDiagnostics>();
    assert!(diagnostics.runtime_active);
    assert_eq!(diagnostics.rigs.len(), 1);
    assert_eq!(diagnostics.rigs[0].layers.len(), 1);
}

#[test]
fn runtime_config_changes_propagate_without_respawning_the_layer() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let camera = app
        .world_mut()
        .spawn((Name::new("Camera"), Camera2d, Camera::default()))
        .id();
    let rig = app
        .world_mut()
        .spawn((ParallaxRig::default(), ParallaxCameraTarget::new(camera)))
        .id();
    let layer = app
        .world_mut()
        .spawn((
            Name::new("Layer"),
            ChildOf(rig),
            Sprite::from_image(image),
            ParallaxLayer {
                strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented::default()),
                source_size: Some(Vec2::new(32.0, 16.0)),
                ..default()
            },
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let initial_offset =
        app.world().resource::<ParallaxDiagnostics>().rigs[0].layers[0].effective_offset;

    app.world_mut().entity_mut(layer).insert(ParallaxLayer {
        phase: Vec2::new(48.0, 0.0),
        strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented::default()),
        source_size: Some(Vec2::new(32.0, 16.0)),
        ..default()
    });
    app.world_mut().run_schedule(Tick);

    let next_offset =
        app.world().resource::<ParallaxDiagnostics>().rigs[0].layers[0].effective_offset;

    assert_ne!(initial_offset, next_offset);
}

#[test]
fn unbound_rig_uses_zero_camera_input_instead_of_reusing_parent_translation() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let rig = app
        .world_mut()
        .spawn((
            ParallaxRig::default(),
            Transform::from_xyz(48.0, -20.0, 0.0),
        ))
        .id();
    let layer = app
        .world_mut()
        .spawn((
            Name::new("Layer"),
            ChildOf(rig),
            Sprite::from_image(image),
            ParallaxLayer::default().with_source_size(Vec2::new(16.0, 16.0)),
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let transform = app.world().entity(layer).get::<Transform>().unwrap();
    assert_eq!(transform.translation, Vec3::ZERO);
}

#[test]
fn disabled_rig_freezes_layer_layout_after_initial_update() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let camera = app
        .world_mut()
        .spawn((Name::new("Camera"), Camera2d, Transform::default()))
        .id();
    let rig = app
        .world_mut()
        .spawn((ParallaxRig::default(), ParallaxCameraTarget::new(camera)))
        .id();
    let layer = app
        .world_mut()
        .spawn((
            Name::new("Layer"),
            ChildOf(rig),
            Sprite::from_image(image),
            ParallaxLayer {
                auto_scroll: Vec2::new(32.0, 0.0),
                source_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);
    let before = app
        .world()
        .entity(layer)
        .get::<Transform>()
        .unwrap()
        .translation;

    app.world_mut()
        .entity_mut(camera)
        .insert(Transform::from_xyz(160.0, 0.0, 0.0));
    app.world_mut().entity_mut(rig).insert(ParallaxRig {
        enabled: false,
        ..default()
    });
    app.world_mut().run_schedule(Tick);

    let after = app
        .world()
        .entity(layer)
        .get::<Transform>()
        .unwrap()
        .translation;
    assert_eq!(after, before);
}

#[test]
fn render_layer_changes_propagate_to_segment_children() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let camera = app
        .world_mut()
        .spawn((Name::new("Camera"), Camera2d, Camera::default()))
        .id();
    let rig = app
        .world_mut()
        .spawn((ParallaxRig::default(), ParallaxCameraTarget::new(camera)))
        .id();
    let layer = app
        .world_mut()
        .spawn((
            Name::new("Layer"),
            ChildOf(rig),
            RenderLayers::layer(3),
            Sprite::from_image(image),
            ParallaxLayer {
                strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented::default()),
                source_size: Some(Vec2::new(32.0, 16.0)),
                ..default()
            },
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let children = child_entities(app.world_mut(), layer);
    assert!(!children.is_empty());
    for child in &children {
        let layers = app.world().entity(*child).get::<RenderLayers>().unwrap();
        assert_eq!(*layers, RenderLayers::layer(3));
    }

    app.world_mut().entity_mut(layer).remove::<RenderLayers>();
    app.world_mut().run_schedule(Tick);

    for child in child_entities(app.world_mut(), layer) {
        assert!(app.world().entity(child).get::<RenderLayers>().is_none());
    }
}

#[test]
fn custom_offset_hook_can_modify_computed_before_write() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let rig = app.world_mut().spawn(ParallaxRig::default()).id();
    let layer = app
        .world_mut()
        .spawn((
            Name::new("Layer"),
            ChildOf(rig),
            Sprite::from_image(image),
            ParallaxLayer::default()
                .with_source_size(Vec2::new(16.0, 16.0))
                .with_phase(Vec2::new(7.0, 0.0)),
        ))
        .id();

    // Inject a system between ComputeOffsets and WriteTransforms
    app.add_systems(
        Tick,
        (|mut query: Query<&mut ParallaxLayerComputed>| {
            for mut computed in &mut query {
                computed.offset.x += 50.0;
            }
        })
        .after(ParallaxScrollerSystems::ComputeOffsets)
        .before(ParallaxScrollerSystems::WriteTransforms),
    );

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let transform = app.world().entity(layer).get::<Transform>().unwrap();
    // phase(7) + hook(50) = 57
    assert!((transform.translation.x - 57.0).abs() < 0.001);
}

#[test]
fn user_offset_and_user_scale_are_additive() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let rig = app.world_mut().spawn(ParallaxRig::default()).id();
    let layer = app
        .world_mut()
        .spawn((
            Name::new("Layer"),
            ChildOf(rig),
            Sprite::from_image(image),
            ParallaxLayer {
                phase: Vec2::new(3.0, 0.0),
                user_offset: Vec2::new(5.0, -3.0),
                user_scale: Vec2::new(2.0, 3.0),
                source_size: Some(Vec2::new(16.0, 16.0)),
                repeat: saddle_rendering_parallax_scroller::ParallaxAxes::none(),
                ..default()
            },
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let t = app.world().entity(layer).get::<Transform>().unwrap();
    assert!((t.translation.x - 8.0).abs() < 0.001); // phase(3) + user_offset(5) = 8
    assert!((t.translation.y + 3.0).abs() < 0.001); // 0 + (-3)
    assert!((t.scale.x - 2.0).abs() < 0.001); // 1 * 2
    assert!((t.scale.y - 3.0).abs() < 0.001); // 1 * 3
}

#[test]
fn time_scale_zero_freezes_auto_scroll() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let rig = app.world_mut().spawn(ParallaxRig::default()).id();
    let layer = app
        .world_mut()
        .spawn((
            Name::new("Layer"),
            ChildOf(rig),
            Sprite::from_image(image),
            ParallaxLayer {
                auto_scroll: Vec2::new(999.0, 0.0),
                source_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
        ))
        .id();

    app.world_mut().resource_mut::<ParallaxTimeScale>().0 = 0.0;

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let runtime = app
        .world()
        .entity(layer)
        .get::<LayerRuntimeState>()
        .unwrap();
    assert_eq!(runtime.auto_phase, Vec2::ZERO);
}

#[test]
fn runtime_state_is_publicly_accessible() {
    let mut app = app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(white_image())
    };

    let rig = app.world_mut().spawn(ParallaxRig::default()).id();
    let layer = app
        .world_mut()
        .spawn((
            Name::new("Layer"),
            ChildOf(rig),
            Sprite::from_image(image),
            ParallaxLayer::default().with_source_size(Vec2::new(16.0, 16.0)),
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    // Public API: can query RigRuntimeState and LayerRuntimeState
    let _rig_state: &RigRuntimeState = app.world().entity(rig).get().unwrap();
    let _layer_state: &LayerRuntimeState = app.world().entity(layer).get().unwrap();
    let _computed: &ParallaxLayerComputed = app.world().entity(layer).get().unwrap();
}
