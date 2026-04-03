use std::collections::{HashMap, HashSet};

use bevy::{
    camera::visibility::RenderLayers,
    ecs::hierarchy::ChildOf,
    gizmos::{config::DefaultGizmoConfigGroup, prelude::Gizmos},
    math::Rot2,
    prelude::*,
    transform::helper::TransformHelper,
};

use crate::{
    ParallaxCameraTarget, ParallaxDebugSettings, ParallaxDiagnostics, ParallaxLayer,
    ParallaxLayerDiagnostics, ParallaxLayerStrategy, ParallaxRig, ParallaxRigDiagnostics,
    ParallaxTiledSprite,
    math::{
        advance_phase, apply_repeat_and_bounds, centered_indices, compute_unbounded_offset,
        coverage_size, perspective_depth_ratio, required_segment_grid,
        resolve_depth_mapped_camera_factor, resolve_depth_mapped_scale, safe_abs_scale,
        snap_offset,
    },
    resources::ParallaxRuntimeState,
};

#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct RigRuntimeState {
    pub camera_target: Option<Entity>,
    pub camera_position: Vec2,
    pub camera_depth: f32,
    pub camera_is_perspective: bool,
    pub viewport_size: Vec2,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct LayerRuntimeState {
    pub effective_camera_factor: Vec2,
    pub effective_scale: Vec2,
    pub auto_phase: Vec2,
    pub depth_ratio: Option<f32>,
    pub effective_offset: Vec2,
    pub wrap_span: Vec2,
    pub coverage_size: Vec2,
    pub segment_grid: UVec2,
}

impl Default for LayerRuntimeState {
    fn default() -> Self {
        Self {
            effective_camera_factor: Vec2::ZERO,
            effective_scale: Vec2::ONE,
            auto_phase: Vec2::ZERO,
            depth_ratio: None,
            effective_offset: Vec2::ZERO,
            wrap_span: Vec2::ZERO,
            coverage_size: Vec2::ZERO,
            segment_grid: UVec2::ONE,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ManagedSegment {
    pub grid: IVec2,
}

pub(crate) fn activate_runtime(mut runtime: ResMut<ParallaxRuntimeState>) {
    runtime.active = true;
}

pub(crate) fn deactivate_runtime(
    mut commands: Commands,
    mut runtime: ResMut<ParallaxRuntimeState>,
    mut diagnostics: ResMut<ParallaxDiagnostics>,
    rigs: Query<Entity, With<ParallaxRig>>,
    layers: Query<Entity, With<ParallaxLayer>>,
    segments: Query<Entity, With<ManagedSegment>>,
) {
    runtime.active = false;
    diagnostics.runtime_active = false;
    diagnostics.rigs.clear();

    for rig in &rigs {
        commands.entity(rig).remove::<RigRuntimeState>();
    }

    for layer in &layers {
        commands.entity(layer).remove::<LayerRuntimeState>();
    }

    for segment in &segments {
        commands.entity(segment).despawn();
    }
}

pub(crate) fn runtime_is_active(runtime: Res<ParallaxRuntimeState>) -> bool {
    runtime.active
}

pub(crate) fn ensure_rig_state(
    mut commands: Commands,
    rigs: Query<Entity, (With<ParallaxRig>, Without<RigRuntimeState>)>,
) {
    for rig in &rigs {
        commands.entity(rig).insert(RigRuntimeState::default());
    }
}

pub(crate) fn ensure_layer_state(
    mut commands: Commands,
    layers: Query<Entity, (With<ParallaxLayer>, Without<LayerRuntimeState>)>,
) {
    for layer in &layers {
        commands.entity(layer).insert(LayerRuntimeState::default());
    }
}

pub(crate) fn track_camera_targets(
    transform_helper: TransformHelper,
    cameras: Query<(&Camera, Option<&Projection>)>,
    mut rigs: Query<
        (
            &ParallaxRig,
            Option<&ParallaxCameraTarget>,
            &mut RigRuntimeState,
        ),
        With<ParallaxRig>,
    >,
) {
    for (rig, camera_target, mut runtime) in &mut rigs {
        runtime.camera_target = camera_target.map(|target| target.camera);

        if !rig.enabled {
            continue;
        }

        runtime.camera_position = Vec2::ZERO;
        runtime.camera_depth = 0.0;
        runtime.camera_is_perspective = false;
        runtime.viewport_size = Vec2::ZERO;

        let Some(target) = camera_target else {
            continue;
        };
        let Ok((camera, projection)) = cameras.get(target.camera) else {
            continue;
        };
        let Ok(camera_transform) = transform_helper.compute_global_transform(target.camera) else {
            continue;
        };

        runtime.camera_position = camera_transform.translation().truncate();
        runtime.camera_depth = camera_transform.translation().z;
        runtime.camera_is_perspective =
            projection.is_some_and(|projection| matches!(projection, Projection::Perspective(_)));
        runtime.viewport_size = if runtime.camera_is_perspective {
            camera.logical_viewport_size().unwrap_or(Vec2::ZERO)
        } else {
            camera
                .logical_viewport_rect()
                .and_then(|viewport| {
                    let min = camera
                        .viewport_to_world_2d(&camera_transform, viewport.min)
                        .ok()?;
                    let max = camera
                        .viewport_to_world_2d(&camera_transform, viewport.max)
                        .ok()?;
                    Some((max - min).abs())
                })
                .or_else(|| camera.logical_viewport_size())
                .unwrap_or(Vec2::ZERO)
        };
    }
}

pub(crate) fn advance_layer_phase(
    time: Res<Time>,
    rigs: Query<&ParallaxRig>,
    mut layers: Query<(&ChildOf, &ParallaxLayer, &mut LayerRuntimeState), With<ParallaxLayer>>,
) {
    let dt = time.delta_secs();
    for (child_of, layer, mut runtime) in &mut layers {
        if !layer.enabled {
            continue;
        }
        if let Ok(rig) = rigs.get(child_of.parent()) {
            if !rig.enabled {
                continue;
            }
        }

        runtime.auto_phase = advance_phase(runtime.auto_phase, layer.auto_scroll, dt);
    }
}

pub(crate) fn apply_layout(
    images: Res<Assets<Image>>,
    rigs: Query<(&ParallaxRig, &RigRuntimeState, &GlobalTransform)>,
    mut layers: Query<
        (
            Entity,
            &ChildOf,
            &ParallaxLayer,
            &mut LayerRuntimeState,
            &mut Transform,
            &mut Sprite,
        ),
        (With<ParallaxLayer>, Without<ManagedSegment>),
    >,
) {
    for (layer_entity, child_of, layer, mut runtime, mut transform, mut sprite) in &mut layers {
        let Ok((rig, rig_runtime, rig_global_transform)) = rigs.get(child_of.parent()) else {
            continue;
        };
        if !rig.enabled {
            continue;
        }

        let source_local_size = resolve_source_local_size(layer, &sprite, &images);
        let depth_ratio = layer.depth_mapping.as_ref().and_then(|depth_mapping| {
            perspective_depth_ratio(
                rig_runtime.camera_is_perspective,
                rig_runtime.camera_depth,
                rig_global_transform.translation().z + layer.depth,
                depth_mapping.reference_plane_z,
            )
        });
        let effective_camera_factor = resolve_depth_mapped_camera_factor(
            layer.camera_factor,
            layer.depth_mapping.as_ref(),
            depth_ratio,
        );
        let effective_scale =
            resolve_depth_mapped_scale(layer.scale, layer.depth_mapping.as_ref(), depth_ratio);
        let scale_abs = safe_abs_scale(effective_scale);
        let source_world_size = source_local_size * scale_abs;
        let resolved_viewport_size =
            if rig_runtime.viewport_size.x > 0.0 && rig_runtime.viewport_size.y > 0.0 {
                rig_runtime.viewport_size
            } else {
                source_world_size.max(Vec2::ONE)
            };

        let unbounded_offset = compute_unbounded_offset(
            rig_runtime.camera_position,
            effective_camera_factor,
            layer.phase,
            runtime.auto_phase,
        );
        let wrap_span = match &layer.strategy {
            ParallaxLayerStrategy::TiledSprite(tiled) => {
                source_world_size * tiled.stretch_value.max(0.0001)
            }
            ParallaxLayerStrategy::Segmented(_) => source_world_size,
        };
        let constrained_offset =
            apply_repeat_and_bounds(unbounded_offset, layer.repeat, wrap_span, layer.bounds);
        let snapped_offset = snap_offset(constrained_offset, &layer.snap);

        runtime.effective_camera_factor = effective_camera_factor;
        runtime.effective_scale = effective_scale;
        runtime.depth_ratio = depth_ratio;
        runtime.effective_offset = snapped_offset;
        runtime.wrap_span = wrap_span;

        transform.translation = Vec3::new(
            rig.origin.x + layer.origin.x + snapped_offset.x,
            rig.origin.y + layer.origin.y + snapped_offset.y,
            layer.depth,
        );
        transform.scale = effective_scale.extend(1.0);

        sprite.color = layer.tint;

        match &layer.strategy {
            ParallaxLayerStrategy::TiledSprite(tiled) => {
                let _ = layer_entity;
                apply_tiled_layer(
                    layer,
                    tiled,
                    &mut runtime,
                    &mut sprite,
                    source_world_size,
                    scale_abs,
                    resolved_viewport_size,
                );
            }
            ParallaxLayerStrategy::Segmented(segmented) => {
                let _ = layer_entity;
                apply_segmented_layer(
                    layer,
                    segmented,
                    &mut runtime,
                    &mut sprite,
                    source_world_size,
                    resolved_viewport_size,
                );
                sprite.custom_size = Some(source_local_size);
            }
        }
    }
}

fn apply_tiled_layer(
    layer: &ParallaxLayer,
    tiled: &ParallaxTiledSprite,
    runtime: &mut Mut<LayerRuntimeState>,
    sprite: &mut Mut<Sprite>,
    source_world_size: Vec2,
    scale_abs: Vec2,
    viewport_size: Vec2,
) {
    sprite.image_mode = SpriteImageMode::Tiled {
        tile_x: layer.repeat.x,
        tile_y: layer.repeat.y,
        stretch_value: tiled.stretch_value.max(0.0001),
    };

    let coverage_world = coverage_size(
        viewport_size,
        layer.coverage_margin,
        tiled.minimum_coverage,
        layer.repeat,
        source_world_size,
    );
    runtime.coverage_size = coverage_world;
    runtime.segment_grid = UVec2::ONE;
    sprite.custom_size = Some(coverage_world / scale_abs);
}

fn apply_segmented_layer(
    layer: &ParallaxLayer,
    segmented: &crate::ParallaxSegmented,
    runtime: &mut Mut<LayerRuntimeState>,
    sprite: &mut Mut<Sprite>,
    source_world_size: Vec2,
    viewport_size: Vec2,
) {
    sprite.image_mode = SpriteImageMode::Auto;

    runtime.coverage_size = source_world_size;
    runtime.segment_grid = required_segment_grid(
        viewport_size + layer.coverage_margin.max(Vec2::ZERO) * 2.0,
        source_world_size,
        layer.repeat,
        segmented.extra_rings,
    );
}

pub(crate) fn sync_segment_children(
    mut commands: Commands,
    layers: Query<
        (
            Entity,
            &ParallaxLayer,
            &LayerRuntimeState,
            &Sprite,
            Option<&RenderLayers>,
        ),
        (With<ParallaxLayer>, Without<ManagedSegment>),
    >,
    mut segments: Query<
        (
            Entity,
            &ManagedSegment,
            &ChildOf,
            &mut Transform,
            &mut Sprite,
        ),
        (With<ManagedSegment>, Without<ParallaxLayer>),
    >,
) {
    let mut existing = HashMap::<Entity, HashMap<IVec2, Entity>>::new();
    for (entity, managed, child_of, _, _) in &segments {
        existing
            .entry(child_of.parent())
            .or_default()
            .insert(managed.grid, entity);
    }

    let mut desired_by_layer = HashMap::<Entity, HashSet<IVec2>>::new();
    for (layer_entity, layer, runtime, sprite, render_layers) in &layers {
        match &layer.strategy {
            ParallaxLayerStrategy::Segmented(_) => {
                let desired = desired_segment_positions(runtime.segment_grid);
                let desired_set: HashSet<IVec2> = desired.iter().copied().collect();
                desired_by_layer.insert(layer_entity, desired_set.clone());

                let source_local_size = sprite.custom_size.unwrap_or(Vec2::ONE);
                let template_sprite = sprite.clone();
                let existing_for_layer = existing.remove(&layer_entity).unwrap_or_default();

                for (grid, entity) in &existing_for_layer {
                    if !desired_set.contains(grid) {
                        commands.entity(*entity).despawn();
                    }
                }

                for grid in desired {
                    if grid == IVec2::ZERO {
                        continue;
                    }

                    if let Some(entity) = existing_for_layer.get(&grid).copied() {
                        if let Ok((_, _, _, mut segment_transform, mut segment_sprite)) =
                            segments.get_mut(entity)
                        {
                            segment_transform.translation = Vec3::new(
                                grid.x as f32 * source_local_size.x,
                                grid.y as f32 * source_local_size.y,
                                0.0,
                            );
                            *segment_sprite = template_sprite.clone();
                        }
                        if let Some(render_layers) = render_layers.cloned() {
                            commands.entity(entity).insert(render_layers);
                        } else {
                            commands.entity(entity).remove::<RenderLayers>();
                        }
                    } else {
                        let mut child = commands.spawn((
                            Name::new("Parallax Segment"),
                            ManagedSegment { grid },
                            ChildOf(layer_entity),
                            Transform::from_translation(Vec3::new(
                                grid.x as f32 * source_local_size.x,
                                grid.y as f32 * source_local_size.y,
                                0.0,
                            )),
                            GlobalTransform::default(),
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            template_sprite.clone(),
                        ));

                        if let Some(render_layers) = render_layers.cloned() {
                            child.insert(render_layers);
                        }
                    }
                }
            }
            ParallaxLayerStrategy::TiledSprite(_) => {
                if let Some(existing_for_layer) = existing.remove(&layer_entity) {
                    for (_, entity) in existing_for_layer {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }

    for (layer_entity, existing_for_layer) in existing {
        if desired_by_layer.contains_key(&layer_entity) {
            continue;
        }
        for (_, entity) in existing_for_layer {
            commands.entity(entity).despawn();
        }
    }
}

fn desired_segment_positions(segment_grid: UVec2) -> Vec<IVec2> {
    let mut result = Vec::with_capacity((segment_grid.x * segment_grid.y) as usize);
    for y in centered_indices(segment_grid.y) {
        for x in centered_indices(segment_grid.x) {
            result.push(IVec2::new(x, y));
        }
    }
    result
}

fn resolve_source_local_size(
    layer: &ParallaxLayer,
    sprite: &Sprite,
    images: &Assets<Image>,
) -> Vec2 {
    if let Some(source_size) = layer.source_size {
        return source_size.max(Vec2::splat(0.0001));
    }

    if let Some(custom_size) = sprite.custom_size {
        return custom_size.max(Vec2::splat(0.0001));
    }

    if let Some(rect) = sprite.rect {
        return rect.size().max(Vec2::splat(0.0001));
    }

    images
        .get(&sprite.image)
        .map(Image::size)
        .map(|size| size.as_vec2().max(Vec2::splat(0.0001)))
        .unwrap_or(Vec2::ONE)
}

pub(crate) fn publish_diagnostics(
    runtime: Res<ParallaxRuntimeState>,
    mut diagnostics: ResMut<ParallaxDiagnostics>,
    rigs: Query<(Entity, &ParallaxRig, &RigRuntimeState)>,
    layers: Query<(Entity, &ChildOf, &ParallaxLayer, &LayerRuntimeState)>,
) {
    diagnostics.runtime_active = runtime.active;
    diagnostics.rigs.clear();

    let mut by_rig = HashMap::<Entity, ParallaxRigDiagnostics>::new();
    for (entity, rig, rig_runtime) in &rigs {
        by_rig.insert(
            entity,
            ParallaxRigDiagnostics {
                rig: entity,
                camera_target: rig_runtime.camera_target,
                camera_position: rig_runtime.camera_position,
                viewport_size: rig_runtime.viewport_size,
                enabled: rig.enabled,
                layers: Vec::new(),
            },
        );
    }

    for (entity, child_of, layer, layer_runtime) in &layers {
        let Some(rig_entry) = by_rig.get_mut(&child_of.parent()) else {
            continue;
        };
        rig_entry.layers.push(ParallaxLayerDiagnostics {
            layer: entity,
            strategy: layer.strategy.kind(),
            repeat: layer.repeat,
            effective_camera_factor: layer_runtime.effective_camera_factor,
            effective_scale: layer_runtime.effective_scale,
            effective_offset: layer_runtime.effective_offset,
            depth_ratio: layer_runtime.depth_ratio,
            wrap_span: layer_runtime.wrap_span,
            coverage_size: layer_runtime.coverage_size,
            segment_grid: layer_runtime.segment_grid,
        });
    }

    diagnostics.rigs = by_rig.into_values().collect();
    diagnostics.rigs.sort_by_key(|entry| entry.rig.index());
    for rig in &mut diagnostics.rigs {
        rig.layers.sort_by_key(|entry| entry.layer.index());
    }
}

pub(crate) fn draw_debug_gizmos(
    debug: Res<ParallaxDebugSettings>,
    rigs: Query<(Entity, &ParallaxRig, &GlobalTransform, &RigRuntimeState)>,
    layers: Query<(&ChildOf, &ParallaxLayer, &LayerRuntimeState)>,
    mut gizmos: Gizmos<DefaultGizmoConfigGroup>,
) {
    if !debug.enabled {
        return;
    }

    let mut rig_lookup = HashMap::new();
    for (entity, rig, global, runtime) in &rigs {
        let rig_origin = global.translation().truncate() + rig.origin;
        rig_lookup.insert(
            entity,
            (rig_origin, runtime.camera_position, runtime.viewport_size),
        );

        if debug.draw_viewport_bounds
            && runtime.viewport_size.x > 0.0
            && runtime.viewport_size.y > 0.0
        {
            gizmos.rect_2d(
                Isometry2d::new(runtime.camera_position, Rot2::default()),
                runtime.viewport_size,
                debug.viewport_color,
            );
        }
    }

    for (child_of, layer, runtime) in &layers {
        let Some((rig_origin, _, _)) = rig_lookup.get(&child_of.parent()) else {
            continue;
        };
        let layer_center = *rig_origin + layer.origin + runtime.effective_offset;

        if debug.draw_layer_bounds && runtime.coverage_size.x > 0.0 && runtime.coverage_size.y > 0.0
        {
            gizmos.rect_2d(
                Isometry2d::new(layer_center, Rot2::default()),
                runtime.coverage_size,
                debug.coverage_color,
            );
        }

        if debug.draw_layer_bounds && runtime.wrap_span.x > 0.0 && runtime.wrap_span.y > 0.0 {
            gizmos.rect_2d(
                Isometry2d::new(layer_center, Rot2::default()),
                runtime.wrap_span,
                debug.wrap_color,
            );
        }

        if debug.draw_offsets {
            gizmos.line_2d(*rig_origin + layer.origin, layer_center, debug.offset_color);
        }
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
