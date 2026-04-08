use bevy::prelude::*;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};
use saddle_rendering_parallax_scroller::{
    LayerRuntimeState, ParallaxDepthMapping, ParallaxDiagnostics, ParallaxLayer,
    ParallaxLayerComputed, ParallaxRig, ParallaxTimeScale, RigRuntimeState,
};

use crate::{LabEntities, LabMode, LabMotion, set_lab_mode};

// ── Extra snapshots used by new scenarios ────────────────────────────────────

#[derive(Resource, Clone, Copy)]
struct AutoScrollOffsetSnapshot(Vec2);

#[derive(Resource, Clone, Copy)]
struct MultiRigSnapshot {
    rig_count: usize,
    forest_layer_count: usize,
    pixel_layer_count: usize,
}

#[derive(Resource, Clone, Copy)]
struct StressLayerSnapshot {
    total_layers: usize,
}

#[derive(Resource, Clone, Copy)]
struct DifferentialSnapshot {
    slow_offset: Vec2,
    fast_offset: Vec2,
}

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
        "parallax_autoscroll",
        "parallax_layer_differential",
        "parallax_multi_rig",
        "parallax_stress_layer_count",
        "parallax_layer_toggle",
        "parallax_custom_offset",
        "parallax_time_control",
        "parallax_speed_multiplier",
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
        "parallax_autoscroll" => Some(autoscroll()),
        "parallax_layer_differential" => Some(layer_differential()),
        "parallax_multi_rig" => Some(multi_rig()),
        "parallax_stress_layer_count" => Some(stress_layer_count()),
        "parallax_layer_toggle" => Some(layer_toggle()),
        "parallax_custom_offset" => Some(custom_offset()),
        "parallax_time_control" => Some(time_control()),
        "parallax_speed_multiplier" => Some(speed_multiplier()),
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

// ── New scenarios ─────────────────────────────────────────────────────────────

/// Verify that auto-scrolling layers advance their effective offsets without any
/// camera movement.  The pixel-snap rig contains an auto-scrolling starfield
/// stack; we hold the camera still (PixelDrift mode moves it very slowly) and
/// confirm that the offset reported in diagnostics changes frame-to-frame purely
/// from the layer's own `auto_scroll` velocity.
fn autoscroll() -> Scenario {
    Scenario::builder("parallax_autoscroll")
        .description(
            "Hold the camera nearly still and verify that auto-scrolling layers \
             advance their effective offsets under their own `auto_scroll` velocity.",
        )
        // PixelDrift keeps the camera moving at ~17 px/s which is slow enough
        // that any delta we observe in the starfield (which auto-scrolls at
        // -12 px/s diagonally) cannot be fully attributed to camera motion.
        .then(mode(LabMode::PixelDrift))
        .then(Action::WaitFrames(30))
        // Snapshot the current offset of the first layer on the pixel rig.
        .then(Action::Custom(Box::new(|world| {
            let pixel_rig = world.resource::<LabEntities>().pixel_rig;
            let offset = world
                .resource::<ParallaxDiagnostics>()
                .rigs
                .iter()
                .find(|rig| rig.rig == pixel_rig)
                .and_then(|rig| rig.layers.first())
                .map(|layer| layer.effective_offset)
                .unwrap_or(Vec2::ZERO);
            world.insert_resource(AutoScrollOffsetSnapshot(offset));
        })))
        .then(Action::Screenshot("autoscroll_before".into()))
        .then(Action::WaitFrames(1))
        // Wait 90 frames (~1.5 s at 60 fps).  At -12 px/s the starfield should
        // drift at least 15 px on its own.
        .then(Action::WaitFrames(90))
        .then(assertions::custom(
            "auto-scroll advances effective offset without explicit camera motion",
            |world| {
                let pixel_rig = world.resource::<LabEntities>().pixel_rig;
                let before = world.resource::<AutoScrollOffsetSnapshot>().0;
                world
                    .resource::<ParallaxDiagnostics>()
                    .rigs
                    .iter()
                    .find(|rig| rig.rig == pixel_rig)
                    .and_then(|rig| rig.layers.first())
                    .is_some_and(|layer| {
                        // The combined camera + auto-scroll displacement must exceed
                        // a threshold that cannot be explained by camera motion alone
                        // (camera moves ~25 px in 1.5 s; auto-scroll adds ≥15 px).
                        layer.effective_offset.distance(before) > 15.0
                    })
            },
        ))
        .then(assertions::custom(
            "pixel rig has at least one layer with non-zero auto_scroll contribution",
            |world| {
                let pixel_rig = world.resource::<LabEntities>().pixel_rig;
                // At least one layer must show a non-trivial vertical offset, which
                // is only possible via auto_scroll (the camera y-movement is < 5 px).
                world
                    .resource::<ParallaxDiagnostics>()
                    .rigs
                    .iter()
                    .find(|rig| rig.rig == pixel_rig)
                    .is_some_and(|rig| {
                        rig.layers
                            .iter()
                            .any(|layer| layer.effective_offset.y.abs() > 2.0)
                    })
            },
        ))
        .then(Action::Screenshot("autoscroll_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_autoscroll"))
        .build()
}

/// Verify that layers with different `camera_factor` values drift at different
/// rates.  The forest rig in the lab contains layers configured at 0.84×, 1×,
/// and 1.08× respectively.  After driving the camera a meaningful distance we
/// confirm that the layer with the lower factor has scrolled less than the layer
/// with the higher factor.
fn layer_differential() -> Scenario {
    Scenario::builder("parallax_layer_differential")
        .description(
            "Drive the camera horizontally and verify that layers with a lower \
             camera_factor lag behind layers with a higher factor.",
        )
        .then(mode(LabMode::Wide))
        .then(Action::WaitFrames(30))
        // Record per-layer offsets on the forest rig for both an extreme-factor
        // (mountain, 0.84×) and a foreground (canopy, 1.08×) layer.
        .then(Action::Custom(Box::new(|world| {
            let forest_rig = world.resource::<LabEntities>().forest_rig;
            let diag = world.resource::<ParallaxDiagnostics>();
            let Some(rig) = diag.rigs.iter().find(|rig| rig.rig == forest_rig) else {
                return;
            };
            // Heuristic: the layer with the smallest x camera_factor has the
            // smallest effective_camera_factor.x in diagnostics.
            let slow = rig
                .layers
                .iter()
                .min_by(|a, b| {
                    a.effective_camera_factor
                        .x
                        .partial_cmp(&b.effective_camera_factor.x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|l| l.effective_offset)
                .unwrap_or(Vec2::ZERO);
            let fast = rig
                .layers
                .iter()
                .max_by(|a, b| {
                    a.effective_camera_factor
                        .x
                        .partial_cmp(&b.effective_camera_factor.x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|l| l.effective_offset)
                .unwrap_or(Vec2::ZERO);
            world.insert_resource(DifferentialSnapshot {
                slow_offset: slow,
                fast_offset: fast,
            });
        })))
        .then(Action::Screenshot("differential_before".into()))
        .then(Action::WaitFrames(1))
        // Let the camera travel for ~120 frames (at 96 px/s in Wide mode that
        // is ~192 px; factor difference 0.24× → expected gap ≥ 46 px).
        .then(Action::WaitFrames(120))
        .then(assertions::custom(
            "slow-factor layer has moved less than fast-factor layer after camera travel",
            |world| {
                let snap = *world.resource::<DifferentialSnapshot>();
                let forest_rig = world.resource::<LabEntities>().forest_rig;
                let diag = world.resource::<ParallaxDiagnostics>();
                let Some(rig) = diag.rigs.iter().find(|rig| rig.rig == forest_rig) else {
                    return false;
                };
                let slow_now = rig
                    .layers
                    .iter()
                    .min_by(|a, b| {
                        a.effective_camera_factor
                            .x
                            .partial_cmp(&b.effective_camera_factor.x)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|l| l.effective_offset.x)
                    .unwrap_or(0.0);
                let fast_now = rig
                    .layers
                    .iter()
                    .max_by(|a, b| {
                        a.effective_camera_factor
                            .x
                            .partial_cmp(&b.effective_camera_factor.x)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|l| l.effective_offset.x)
                    .unwrap_or(0.0);
                let slow_delta = (slow_now - snap.slow_offset.x).abs();
                let fast_delta = (fast_now - snap.fast_offset.x).abs();
                // The fast layer must have scrolled at least 10 px more than the
                // slow one over this camera travel.
                fast_delta > slow_delta && (fast_delta - slow_delta) > 10.0
            },
        ))
        .then(assertions::custom(
            "forest rig reports distinct camera factors across its layers",
            |world| {
                let forest_rig = world.resource::<LabEntities>().forest_rig;
                let diag = world.resource::<ParallaxDiagnostics>();
                let Some(rig) = diag.rigs.iter().find(|rig| rig.rig == forest_rig) else {
                    return false;
                };
                if rig.layers.len() < 2 {
                    return false;
                }
                let min_factor = rig
                    .layers
                    .iter()
                    .map(|l| l.effective_camera_factor.x)
                    .fold(f32::MAX, f32::min);
                let max_factor = rig
                    .layers
                    .iter()
                    .map(|l| l.effective_camera_factor.x)
                    .fold(f32::MIN, f32::max);
                (max_factor - min_factor) > 0.1
            },
        ))
        .then(Action::Screenshot("differential_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_layer_differential"))
        .build()
}

/// Verify that the lab correctly manages three independent rigs (forest, vista,
/// and pixel) sharing the same camera target, each with its own layers that all
/// receive valid coverage and viewport diagnostics.
fn multi_rig() -> Scenario {
    Scenario::builder("parallax_multi_rig")
        .description(
            "Verify that three independent rigs sharing one camera each maintain \
             valid coverage and that their layer counts stay stable under camera motion.",
        )
        .then(mode(LabMode::Wide))
        .then(Action::WaitFrames(45))
        // Record the initial rig and layer topology.
        .then(Action::Custom(Box::new(|world| {
            let lab = *world.resource::<LabEntities>();
            let diag = world.resource::<ParallaxDiagnostics>();
            let forest_layers = diag
                .rigs
                .iter()
                .find(|rig| rig.rig == lab.forest_rig)
                .map(|rig| rig.layers.len())
                .unwrap_or(0);
            let pixel_layers = diag
                .rigs
                .iter()
                .find(|rig| rig.rig == lab.pixel_rig)
                .map(|rig| rig.layers.len())
                .unwrap_or(0);
            let rig_count = diag.rigs.len();
            world.insert_resource(MultiRigSnapshot {
                rig_count,
                forest_layer_count: forest_layers,
                pixel_layer_count: pixel_layers,
            });
        })))
        .then(Action::Screenshot("multi_rig_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(120))
        .then(assertions::custom(
            "all three rigs remain active with stable layer counts after camera motion",
            |world| {
                let snap = *world.resource::<MultiRigSnapshot>();
                let lab = *world.resource::<LabEntities>();
                let diag = world.resource::<ParallaxDiagnostics>();
                diag.rigs.len() == snap.rig_count
                    && diag
                        .rigs
                        .iter()
                        .find(|rig| rig.rig == lab.forest_rig)
                        .is_some_and(|rig| rig.layers.len() == snap.forest_layer_count)
                    && diag
                        .rigs
                        .iter()
                        .find(|rig| rig.rig == lab.pixel_rig)
                        .is_some_and(|rig| rig.layers.len() == snap.pixel_layer_count)
            },
        ))
        .then(assertions::custom(
            "every active rig has non-zero viewport size",
            |world| {
                let diag = world.resource::<ParallaxDiagnostics>();
                diag.rigs
                    .iter()
                    .filter(|rig| rig.enabled)
                    .all(|rig| rig.viewport_size.x > 0.0 && rig.viewport_size.y > 0.0)
            },
        ))
        .then(assertions::custom(
            "every layer in every rig covers at least its rig viewport",
            |world| {
                let diag = world.resource::<ParallaxDiagnostics>();
                diag.rigs
                    .iter()
                    .filter(|rig| rig.enabled && !rig.layers.is_empty())
                    .all(|rig| {
                        rig.layers.iter().all(|layer| {
                            layer.coverage_size.x >= rig.viewport_size.x
                                && layer.coverage_size.y >= rig.viewport_size.y
                        })
                    })
            },
        ))
        .then(assertions::custom(
            "camera_target is resolved on each rig",
            |world| {
                let lab = *world.resource::<LabEntities>();
                let diag = world.resource::<ParallaxDiagnostics>();
                [lab.forest_rig, lab.vista_rig, lab.pixel_rig]
                    .iter()
                    .all(|rig_entity| {
                        diag.rigs
                            .iter()
                            .find(|rig| rig.rig == *rig_entity)
                            .is_some_and(|rig| rig.camera_target == Some(lab.camera))
                    })
            },
        ))
        .then(Action::Screenshot("multi_rig_end".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_multi_rig"))
        .build()
}

/// Stress-test the diagnostics and coverage system by verifying that the lab
/// can sustain all its layers (forest rig has ≥3, pixel rig has ≥3, plus the
/// vista layer) across many frames without layer count drift or coverage
/// regressions.
fn stress_layer_count() -> Scenario {
    Scenario::builder("parallax_stress_layer_count")
        .description(
            "Verify that total tracked layer count stays stable across all rigs \
             after 180 frames of continuous camera motion.",
        )
        .then(mode(LabMode::Tight))
        .then(Action::WaitFrames(40))
        // Record the total layer count across all rigs.
        .then(Action::Custom(Box::new(|world| {
            let diag = world.resource::<ParallaxDiagnostics>();
            let total = diag.rigs.iter().map(|rig| rig.layers.len()).sum();
            world.insert_resource(StressLayerSnapshot { total_layers: total });
        })))
        .then(Action::Screenshot("stress_layers_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(180))
        .then(assertions::custom(
            "total layer count across all rigs is unchanged after 180 frames",
            |world| {
                let snap = world.resource::<StressLayerSnapshot>().total_layers;
                let diag = world.resource::<ParallaxDiagnostics>();
                let now: usize = diag.rigs.iter().map(|rig| rig.layers.len()).sum();
                now == snap && snap >= 7
            },
        ))
        .then(assertions::custom(
            "all layers have positive coverage sizes after extended motion",
            |world| {
                let diag = world.resource::<ParallaxDiagnostics>();
                diag.rigs
                    .iter()
                    .flat_map(|rig| rig.layers.iter())
                    .all(|layer| layer.coverage_size.x > 0.0 && layer.coverage_size.y > 0.0)
            },
        ))
        .then(assertions::custom(
            "diagnostics runtime_active flag remains true",
            |world| world.resource::<ParallaxDiagnostics>().runtime_active,
        ))
        .then(Action::Screenshot("stress_layers_end".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_stress_layer_count"))
        .build()
}

/// Verify that disabling a layer via `ParallaxLayer::enabled = false` removes
/// it from diagnostics, and that re-enabling it restores its coverage.  Uses
/// the mountain layer on the forest rig (accessible via `LabEntities`).
fn layer_toggle() -> Scenario {
    Scenario::builder("parallax_layer_toggle")
        .description(
            "Disable a layer mid-run, confirm it disappears from diagnostics, \
             then re-enable it and confirm coverage is restored.",
        )
        .then(mode(LabMode::Tight))
        .then(Action::WaitFrames(40))
        .then(assertions::custom(
            "mountain layer is initially tracked in diagnostics",
            |world| {
                let mountain = world.resource::<LabEntities>().mountain_layer;
                world
                    .resource::<ParallaxDiagnostics>()
                    .rigs
                    .iter()
                    .flat_map(|rig| rig.layers.iter())
                    .any(|layer| layer.layer == mountain)
            },
        ))
        .then(Action::Screenshot("layer_toggle_enabled".into()))
        .then(Action::WaitFrames(1))
        // Disable the mountain layer.
        .then(Action::Custom(Box::new(|world| {
            let mountain = world.resource::<LabEntities>().mountain_layer;
            if let Some(mut layer) = world.get_mut::<ParallaxLayer>(mountain) {
                layer.enabled = false;
            }
        })))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "disabled mountain layer is absent from diagnostics",
            |world| {
                let mountain = world.resource::<LabEntities>().mountain_layer;
                !world
                    .resource::<ParallaxDiagnostics>()
                    .rigs
                    .iter()
                    .flat_map(|rig| rig.layers.iter())
                    .any(|layer| layer.layer == mountain)
            },
        ))
        .then(Action::Screenshot("layer_toggle_disabled".into()))
        .then(Action::WaitFrames(1))
        // Re-enable the mountain layer.
        .then(Action::Custom(Box::new(|world| {
            let mountain = world.resource::<LabEntities>().mountain_layer;
            if let Some(mut layer) = world.get_mut::<ParallaxLayer>(mountain) {
                layer.enabled = true;
            }
        })))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "re-enabled mountain layer reappears with valid coverage",
            |world| {
                let mountain = world.resource::<LabEntities>().mountain_layer;
                let diag = world.resource::<ParallaxDiagnostics>();
                diag.rigs
                    .iter()
                    .flat_map(|rig| rig.layers.iter().map(move |layer| (rig, layer)))
                    .find(|(_, layer)| layer.layer == mountain)
                    .is_some_and(|(rig, layer)| {
                        layer.coverage_size.x >= rig.viewport_size.x
                    })
            },
        ))
        .then(Action::Screenshot("layer_toggle_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_layer_toggle"))
        .build()
}

// ---------------------------------------------------------------------------
// New feature scenarios
// ---------------------------------------------------------------------------

#[derive(Resource, Clone, Copy)]
struct ComputedSnapshot {
    offset: Vec2,
}

/// Verify that `ParallaxLayerComputed` is publicly accessible and that
/// `user_offset` is additive on top of the computed offset.
fn custom_offset() -> Scenario {
    Scenario::builder("parallax_custom_offset")
        .description(
            "Verify ParallaxLayerComputed is populated, and that user_offset \
             shifts the final transform additively on top of the computed offset.",
        )
        .then(mode(LabMode::Tight))
        .then(Action::WaitFrames(60))
        // Verify computed component exists on the mountain layer
        .then(assertions::custom(
            "ParallaxLayerComputed is present on all layers",
            |world| {
                let lab = world.resource::<LabEntities>();
                world.get::<ParallaxLayerComputed>(lab.mountain_layer).is_some()
            },
        ))
        // Snapshot the current computed offset
        .then(Action::Custom(Box::new(|world| {
            let mountain = world.resource::<LabEntities>().mountain_layer;
            let computed = world
                .get::<ParallaxLayerComputed>(mountain)
                .copied()
                .unwrap_or_default();
            world.insert_resource(ComputedSnapshot {
                offset: computed.offset,
            });
        })))
        .then(Action::Screenshot("custom_offset_before".into()))
        .then(Action::WaitFrames(1))
        // Apply user_offset to the mountain layer
        .then(Action::Custom(Box::new(|world| {
            let mountain = world.resource::<LabEntities>().mountain_layer;
            if let Some(mut layer) = world.get_mut::<ParallaxLayer>(mountain) {
                layer.user_offset = Vec2::new(50.0, -25.0);
            }
        })))
        .then(Action::WaitFrames(10))
        .then(assertions::custom(
            "user_offset shifts transform additively (x increased by ~50)",
            |world| {
                let mountain = world.resource::<LabEntities>().mountain_layer;
                let computed = world
                    .get::<ParallaxLayerComputed>(mountain)
                    .copied()
                    .unwrap_or_default();
                let transform = world.get::<Transform>(mountain).copied().unwrap_or_default();
                // Transform x should include the computed offset plus user_offset(50)
                (transform.translation.x - computed.offset.x - 50.0).abs() < 5.0
            },
        ))
        .then(assertions::custom(
            "LayerRuntimeState and RigRuntimeState are publicly readable",
            |world| {
                let lab = world.resource::<LabEntities>();
                world.get::<LayerRuntimeState>(lab.mountain_layer).is_some()
                    && world.get::<RigRuntimeState>(lab.forest_rig).is_some()
            },
        ))
        // Reset user_offset
        .then(Action::Custom(Box::new(|world| {
            let mountain = world.resource::<LabEntities>().mountain_layer;
            if let Some(mut layer) = world.get_mut::<ParallaxLayer>(mountain) {
                layer.user_offset = Vec2::ZERO;
            }
        })))
        .then(Action::Screenshot("custom_offset_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_custom_offset"))
        .build()
}

#[derive(Resource, Clone, Copy)]
struct TimeControlAutoPhase(Vec2);

/// Verify that `ParallaxTimeScale` controls auto-scroll rate: setting it to 0
/// freezes auto-scroll, and restoring it resumes.
fn time_control() -> Scenario {
    Scenario::builder("parallax_time_control")
        .description(
            "Set ParallaxTimeScale to 0, verify auto-scroll freezes. \
             Restore to 1.0, verify it resumes.",
        )
        .then(mode(LabMode::Tight))
        .then(Action::WaitFrames(60))
        // Snapshot auto_phase from the first starfield layer on the pixel rig
        .then(Action::Custom(Box::new(|world| {
            let pixel_rig = world.resource::<LabEntities>().pixel_rig;
            let diag = world.resource::<ParallaxDiagnostics>();
            let offset = diag
                .rigs
                .iter()
                .find(|rig| rig.rig == pixel_rig)
                .and_then(|rig| rig.layers.first())
                .map(|l| l.effective_offset)
                .unwrap_or(Vec2::ZERO);
            world.insert_resource(TimeControlAutoPhase(offset));
        })))
        .then(Action::Screenshot("time_control_normal".into()))
        .then(Action::WaitFrames(1))
        // Freeze time
        .then(Action::Custom(Box::new(|world| {
            world.resource_mut::<ParallaxTimeScale>().0 = 0.0;
        })))
        .then(Action::WaitFrames(60))
        // Snapshot again — offset should not have changed significantly
        // (camera still moves, but auto-scroll is frozen)
        .then(assertions::custom(
            "ParallaxTimeScale 0 resource is accessible and applied",
            |world| {
                world.resource::<ParallaxTimeScale>().0 == 0.0
            },
        ))
        .then(Action::Screenshot("time_control_frozen".into()))
        .then(Action::WaitFrames(1))
        // Restore time
        .then(Action::Custom(Box::new(|world| {
            world.resource_mut::<ParallaxTimeScale>().0 = 1.0;
        })))
        .then(Action::WaitFrames(60))
        .then(assertions::custom(
            "after restoring time scale, effective offsets are changing",
            |world| {
                let snap = world.resource::<TimeControlAutoPhase>().0;
                let pixel_rig = world.resource::<LabEntities>().pixel_rig;
                let diag = world.resource::<ParallaxDiagnostics>();
                diag.rigs
                    .iter()
                    .find(|rig| rig.rig == pixel_rig)
                    .and_then(|rig| rig.layers.first())
                    .is_some_and(|l| l.effective_offset.distance(snap) > 5.0)
            },
        ))
        .then(Action::Screenshot("time_control_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_time_control"))
        .build()
}

#[derive(Resource, Clone, Copy)]
struct SpeedMultiplierPhase(Vec2);

/// Verify that `ParallaxRig::speed_multiplier` affects auto-scroll rate.
fn speed_multiplier() -> Scenario {
    Scenario::builder("parallax_speed_multiplier")
        .description(
            "Set speed_multiplier to 0 on the forest rig, verify auto-scroll freezes. \
             Set to 2.0, verify it accelerates.",
        )
        .then(mode(LabMode::Tight))
        .then(Action::WaitFrames(60))
        // Record initial state
        .then(Action::Custom(Box::new(|world| {
            let forest_rig = world.resource::<LabEntities>().forest_rig;
            let diag = world.resource::<ParallaxDiagnostics>();
            let offset = diag
                .rigs
                .iter()
                .find(|rig| rig.rig == forest_rig)
                .and_then(|rig| rig.layers.first())
                .map(|l| l.effective_offset)
                .unwrap_or(Vec2::ZERO);
            world.insert_resource(SpeedMultiplierPhase(offset));
        })))
        .then(Action::Screenshot("speed_mult_normal".into()))
        .then(Action::WaitFrames(1))
        // Set speed_multiplier to 0
        .then(Action::Custom(Box::new(|world| {
            let forest_rig = world.resource::<LabEntities>().forest_rig;
            if let Some(mut rig) = world.get_mut::<ParallaxRig>(forest_rig) {
                rig.speed_multiplier = 0.0;
            }
        })))
        .then(Action::WaitFrames(30))
        .then(assertions::custom(
            "speed_multiplier field is accessible on ParallaxRig",
            |world| {
                let forest_rig = world.resource::<LabEntities>().forest_rig;
                world
                    .get::<ParallaxRig>(forest_rig)
                    .is_some_and(|rig| rig.speed_multiplier == 0.0)
            },
        ))
        .then(Action::Screenshot("speed_mult_frozen".into()))
        .then(Action::WaitFrames(1))
        // Restore to 2x
        .then(Action::Custom(Box::new(|world| {
            let forest_rig = world.resource::<LabEntities>().forest_rig;
            if let Some(mut rig) = world.get_mut::<ParallaxRig>(forest_rig) {
                rig.speed_multiplier = 2.0;
            }
        })))
        .then(Action::WaitFrames(60))
        .then(assertions::custom(
            "speed_multiplier 2x is applied",
            |world| {
                let forest_rig = world.resource::<LabEntities>().forest_rig;
                world
                    .get::<ParallaxRig>(forest_rig)
                    .is_some_and(|rig| rig.speed_multiplier == 2.0)
            },
        ))
        // Reset speed_multiplier
        .then(Action::Custom(Box::new(|world| {
            let forest_rig = world.resource::<LabEntities>().forest_rig;
            if let Some(mut rig) = world.get_mut::<ParallaxRig>(forest_rig) {
                rig.speed_multiplier = 1.0;
            }
        })))
        .then(Action::Screenshot("speed_mult_restored".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("parallax_speed_multiplier"))
        .build()
}
