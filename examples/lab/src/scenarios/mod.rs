use bevy::prelude::*;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_rendering_parallax_scroller::{
    ParallaxDepthMapping, ParallaxDiagnostics, ParallaxLayer,
};

use crate::{LabEntities, LabMode, LabMotion, set_lab_mode};

#[derive(Resource, Clone, Copy)]
struct OffsetSnapshot(Vec2);

#[derive(Resource, Clone, Copy)]
struct CoverageSnapshot(Vec2);

#[derive(Resource, Clone, Copy)]
struct ViewportSnapshot(Vec2);

#[derive(Resource, Clone, Copy)]
struct LayerCountSnapshot(usize);

#[derive(Resource, Clone, Copy)]
struct DepthMappingSnapshot {
    effective_camera_factor: Vec2,
    effective_scale: Vec2,
    depth_ratio: f32,
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "parallax_scroller_smoke",
        "parallax_camera_motion",
        "parallax_finite_bounds",
        "parallax_zoom",
        "parallax_pixel_snap",
        "parallax_depth_mapping",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "parallax_scroller_smoke" => Some(smoke()),
        "parallax_camera_motion" => Some(camera_motion()),
        "parallax_finite_bounds" => Some(finite_bounds()),
        "parallax_zoom" => Some(zoom()),
        "parallax_pixel_snap" => Some(pixel_snap()),
        "parallax_depth_mapping" => Some(depth_mapping()),
        _ => None,
    }
}

fn mode(mode: LabMode) -> Action {
    Action::Custom(Box::new(move |world| set_lab_mode(world, mode)))
}

fn smoke() -> Scenario {
    Scenario::builder("parallax_scroller_smoke")
        .description("Launch the lab, verify all showcase rigs are present, and capture the default composition.")
        .then(Action::WaitFrames(60))
        .then(assertions::resource_exists::<ParallaxDiagnostics>(
            "diagnostics resource exists",
        ))
        .then(assertions::custom("three showcase rigs are active", |world| {
            world.resource::<ParallaxDiagnostics>().rigs.len() == 3
        }))
        .then(assertions::custom("forest rig has multiple layers", |world| {
            let lab = world.resource::<LabEntities>();
            world.resource::<ParallaxDiagnostics>().rigs.iter().any(|rig| {
                rig.rig == lab.forest_rig && rig.layers.len() >= 3
            })
        }))
        .then(assertions::custom("pixel snap rig exists", |world| {
            let lab = world.resource::<LabEntities>();
            world.resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .any(|rig| rig.rig == lab.pixel_rig)
        }))
        .then(assertions::custom("named showcase layers resolved", |world| {
            let lab = world.resource::<LabEntities>();
            lab.vista_layer != Entity::PLACEHOLDER
                && lab.snapped_layer != Entity::PLACEHOLDER
                && lab.unsnapped_layer != Entity::PLACEHOLDER
        }))
        .then(assertions::custom("forest coverage is at least the visible viewport", |world| {
            let lab = world.resource::<LabEntities>();
            let Some(rig) = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .find(|rig| rig.rig == lab.forest_rig)
            else {
                return false;
            };
            rig.layers.first().is_some_and(|layer| {
                layer.coverage_size.x >= rig.viewport_size.x
                    && layer.coverage_size.y >= rig.viewport_size.y
            })
        }))
        .then(Action::Screenshot("parallax_smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_scroller_smoke"))
        .build()
}

fn camera_motion() -> Scenario {
    Scenario::builder("parallax_camera_motion")
        .description("Drive the camera across wrap boundaries, capture multiple checkpoints, and verify effective offsets change without changing layer counts.")
        .then(mode(LabMode::Wide))
        .then(Action::WaitFrames(30))
        .then(Action::Screenshot("camera_motion_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            let forest_rig = world.resource::<LabEntities>().forest_rig;
            let offset = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .find(|rig| rig.rig == forest_rig)
                .and_then(|rig| rig.layers.first())
                .map(|layer| layer.effective_offset)
                .unwrap_or(Vec2::ZERO);
            world.insert_resource(OffsetSnapshot(offset));
            let layer_count = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .find(|rig| rig.rig == forest_rig)
                .map(|rig| rig.layers.len())
                .unwrap_or_default();
            world.insert_resource(LayerCountSnapshot(layer_count));
        })))
        .then(Action::WaitFrames(80))
        .then(Action::Screenshot("camera_motion_mid".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(90))
        .then(assertions::custom("forest offsets changed under camera motion", |world| {
            let lab = world.resource::<LabEntities>();
            let before = world.resource::<OffsetSnapshot>().0;
            let Some(rig) = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .find(|rig| rig.rig == lab.forest_rig)
            else {
                return false;
            };
            rig.layers
                .first()
                .is_some_and(|layer| layer.effective_offset.distance(before) > 8.0)
                && rig.layers.len() == world.resource::<LayerCountSnapshot>().0
                && rig.layers.iter().any(|layer| {
                    layer.segment_grid.x >= 3
                        && layer.segment_grid.x % 2 == 1
                        && layer.wrap_span.x > 0.0
                })
        }))
        .then(Action::Screenshot("camera_motion_end".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_camera_motion"))
        .build()
}

fn finite_bounds() -> Scenario {
    Scenario::builder("parallax_finite_bounds")
        .description("Drive the lab camera far enough to hit the finite vista clamp and verify the layer stays inside its authored horizontal bounds.")
        .then(mode(LabMode::Wide))
        .then(Action::WaitFrames(40))
        .then(Action::Screenshot("finite_bounds_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(220))
        .then(assertions::custom("finite vista layer stays clamped to +160", |world| {
            let lab = world.resource::<LabEntities>();
            let Some(rig) = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .find(|rig| rig.rig == lab.vista_rig)
            else {
                return false;
            };
            rig.layers
                .iter()
                .find(|layer| layer.layer == lab.vista_layer)
                .is_some_and(|layer| layer.effective_offset.x >= 159.0 && layer.effective_offset.x <= 160.0)
        }))
        .then(Action::Screenshot("finite_bounds_end".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_finite_bounds"))
        .build()
}

fn zoom() -> Scenario {
    Scenario::builder("parallax_zoom")
        .description("Widen the orthographic view, capture before and after, and verify the tiled coverage grows with the viewport.")
        .then(mode(LabMode::Tight))
        .then(Action::WaitFrames(45))
        .then(Action::Screenshot("zoom_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            let forest_rig = world.resource::<LabEntities>().forest_rig;
            let before = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .find(|rig| rig.rig == forest_rig)
                .and_then(|rig| rig.layers.first())
                .map(|layer| layer.coverage_size)
                .unwrap_or(Vec2::ZERO);
            world.insert_resource(CoverageSnapshot(before));
            let viewport = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .find(|rig| rig.rig == forest_rig)
                .map(|rig| rig.viewport_size)
                .unwrap_or(Vec2::ZERO);
            world.insert_resource(ViewportSnapshot(viewport));
            set_lab_mode(world, LabMode::Wide);
        })))
        .then(Action::WaitFrames(70))
        .then(assertions::custom("coverage expands after zooming out", |world| {
            let lab = world.resource::<LabEntities>();
            let before = world.resource::<CoverageSnapshot>().0;
            let viewport_before = world.resource::<ViewportSnapshot>().0;
            world.resource::<ParallaxDiagnostics>().rigs.iter().find(|rig| rig.rig == lab.forest_rig).is_some_and(|rig| {
                rig.viewport_size.x > viewport_before.x
                    && rig.layers.first().is_some_and(|layer| {
                        layer.coverage_size.x > before.x
                            && layer.coverage_size.x >= rig.viewport_size.x
                            && layer.coverage_size.y >= rig.viewport_size.y
                    })
            })
        }))
        .then(Action::Screenshot("zoom_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_zoom"))
        .build()
}

fn pixel_snap() -> Scenario {
    Scenario::builder("parallax_pixel_snap")
        .description("Enable slow drift, capture two frames, and verify the snapped rig keeps whole-pixel offsets while the unsnapped rig does not.")
        .then(mode(LabMode::PixelDrift))
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("pixel_snap_a".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(40))
        .then(assertions::custom("snapped and unsnapped offsets diverge", |world| {
            let lab = world.resource::<LabEntities>();
            let Some(rig) = world.resource::<ParallaxDiagnostics>().rigs.iter().find(|rig| rig.rig == lab.pixel_rig) else {
                return false;
            };
            let snapped = rig.layers.iter().find(|layer| layer.layer == lab.snapped_layer);
            let unsnapped = rig.layers.iter().find(|layer| layer.layer == lab.unsnapped_layer);
            match (snapped, unsnapped) {
                (Some(snapped), Some(unsnapped)) => {
                    snapped.effective_offset.x.fract().abs() < 0.001
                        && unsnapped.effective_offset.x.fract().abs() > 0.05
                }
                _ => false,
            }
        }))
        .then(assertions::resource_satisfies::<LabMotion>(
            "pixel drift mode active",
            |motion| motion.mode == LabMode::PixelDrift,
        ))
        .then(Action::Screenshot("pixel_snap_b".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_pixel_snap"))
        .build()
}

fn depth_mapping() -> Scenario {
    Scenario::builder("parallax_depth_mapping")
        .description(
            "Convert the mountain layer to perspective depth mapping, vary the camera depth, and verify the effective parallax factor and scale respond.",
        )
        .then(mode(LabMode::Tight))
        .then(Action::WaitFrames(20))
        .then(Action::Custom(Box::new(|world| {
            let lab = *world.resource::<LabEntities>();
            if let Some(mut layer) = world.get_mut::<ParallaxLayer>(lab.mountain_layer) {
                layer.camera_factor = Vec2::ZERO;
                layer.depth = -4.0;
                layer.depth_mapping = Some(ParallaxDepthMapping {
                    reference_plane_z: 0.0,
                    translation_response: Vec2::new(1.0, 0.0),
                    scale_response: 1.0,
                });
            }
            *world
                .get_mut::<Projection>(lab.camera)
                .expect("lab camera should expose a projection") =
                Projection::Perspective(PerspectiveProjection {
                    fov: std::f32::consts::FRAC_PI_4,
                    near: 0.1,
                    far: 2000.0,
                    ..default()
                });
            world
                .get_mut::<Transform>(lab.camera)
                .expect("lab camera should expose a transform")
                .translation
                .z = 8.0;
        })))
        .then(Action::WaitFrames(24))
        .then(assertions::custom(
            "depth mapping produces non-zero diagnostics under perspective projection",
            |world| {
                let mountain_layer = world.resource::<LabEntities>().mountain_layer;
                world.resource::<ParallaxDiagnostics>().rigs.iter().any(|rig| {
                    rig.layers.iter().any(|layer| {
                        layer.layer == mountain_layer
                            && layer.depth_ratio.is_some()
                            && layer.effective_camera_factor.length() > 0.01
                    })
                })
            },
        ))
        .then(Action::Screenshot("depth_mapping_before".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            let lab = *world.resource::<LabEntities>();
            let layer = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .flat_map(|rig| rig.layers.iter())
                .find(|layer| layer.layer == lab.mountain_layer)
                .expect("mountain diagnostics should exist after settle");
            world.insert_resource(DepthMappingSnapshot {
                effective_camera_factor: layer.effective_camera_factor,
                effective_scale: layer.effective_scale,
                depth_ratio: layer.depth_ratio.unwrap_or(1.0),
            });
            world
                .get_mut::<Transform>(lab.camera)
                .expect("lab camera should expose a transform")
                .translation
                .z = 15.0;
        })))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "effective factor and scale change when camera depth changes",
            |world| {
                let before = world.resource::<DepthMappingSnapshot>();
                let mountain_layer = world.resource::<LabEntities>().mountain_layer;
                world.resource::<ParallaxDiagnostics>().rigs.iter().any(|rig| {
                    rig.layers.iter().any(|layer| {
                        layer.layer == mountain_layer
                            && layer.depth_ratio.is_some_and(|ratio| {
                                (ratio - before.depth_ratio).abs() > 0.05
                            })
                            && layer
                                .effective_camera_factor
                                .distance(before.effective_camera_factor)
                                > 0.05
                            && layer.effective_scale.distance(before.effective_scale) > 0.05
                    })
                })
            },
        ))
        .then(Action::Screenshot("depth_mapping_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_depth_mapping"))
        .build()
}
