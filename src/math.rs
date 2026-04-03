use bevy::prelude::*;

use crate::config::{
    AxisRange, ParallaxAxes, ParallaxBounds, ParallaxDepthMapping, ParallaxSnap,
};

pub(crate) fn advance_phase(current: Vec2, velocity: Vec2, dt: f32) -> Vec2 {
    if dt <= 0.0 {
        current
    } else {
        current + velocity * dt
    }
}

pub(crate) fn compute_unbounded_offset(
    camera_position: Vec2,
    camera_factor: Vec2,
    phase: Vec2,
    auto_phase: Vec2,
) -> Vec2 {
    camera_position * camera_factor + phase + auto_phase
}

pub(crate) fn perspective_depth_ratio(
    is_perspective_camera: bool,
    camera_plane_z: f32,
    layer_plane_z: f32,
    reference_plane_z: f32,
) -> Option<f32> {
    if !is_perspective_camera {
        return None;
    }

    let reference_distance = (camera_plane_z - reference_plane_z).abs();
    let layer_distance = (camera_plane_z - layer_plane_z).abs();
    if reference_distance <= f32::EPSILON || layer_distance <= f32::EPSILON {
        return None;
    }

    Some(reference_distance / layer_distance)
}

pub(crate) fn resolve_depth_mapped_camera_factor(
    base_camera_factor: Vec2,
    depth_mapping: Option<&ParallaxDepthMapping>,
    depth_ratio: Option<f32>,
) -> Vec2 {
    let Some(depth_mapping) = depth_mapping else {
        return base_camera_factor;
    };
    let Some(depth_ratio) = depth_ratio else {
        return base_camera_factor;
    };

    base_camera_factor + Vec2::splat(1.0 - depth_ratio) * depth_mapping.translation_response
}

pub(crate) fn resolve_depth_mapped_scale(
    base_scale: Vec2,
    depth_mapping: Option<&ParallaxDepthMapping>,
    depth_ratio: Option<f32>,
) -> Vec2 {
    let Some(depth_mapping) = depth_mapping else {
        return base_scale;
    };
    let Some(depth_ratio) = depth_ratio else {
        return base_scale;
    };

    let scale_factor = (1.0 + (depth_ratio - 1.0) * depth_mapping.scale_response).max(0.0001);
    base_scale * scale_factor
}

pub(crate) fn wrap_centered(value: f32, span: f32) -> f32 {
    if !span.is_finite() || span <= f32::EPSILON {
        return value;
    }

    let wrapped = value.rem_euclid(span);
    if wrapped >= span * 0.5 {
        wrapped - span
    } else {
        wrapped
    }
}

pub(crate) fn apply_axis_bounds(value: f32, bounds: Option<AxisRange>) -> f32 {
    bounds.map_or(value, |bounds| bounds.clamp(value))
}

pub(crate) fn apply_repeat_and_bounds(
    offset: Vec2,
    repeat: ParallaxAxes,
    wrap_span: Vec2,
    bounds: ParallaxBounds,
) -> Vec2 {
    Vec2::new(
        if repeat.x {
            wrap_centered(offset.x, wrap_span.x)
        } else {
            apply_axis_bounds(offset.x, bounds.x)
        },
        if repeat.y {
            wrap_centered(offset.y, wrap_span.y)
        } else {
            apply_axis_bounds(offset.y, bounds.y)
        },
    )
}

pub(crate) fn snap_offset(offset: Vec2, snap: &ParallaxSnap) -> Vec2 {
    match snap {
        ParallaxSnap::None => offset,
        ParallaxSnap::Pixel => offset.round(),
        ParallaxSnap::Grid(step) => {
            Vec2::new(snap_axis(offset.x, step.x), snap_axis(offset.y, step.y))
        }
    }
}

fn snap_axis(value: f32, step: f32) -> f32 {
    if !step.is_finite() || step <= f32::EPSILON {
        value
    } else {
        (value / step).round() * step
    }
}

pub(crate) fn safe_abs_scale(scale: Vec2) -> Vec2 {
    Vec2::new(scale.x.abs().max(0.0001), scale.y.abs().max(0.0001))
}

pub(crate) fn coverage_size(
    viewport_size: Vec2,
    margin: Vec2,
    minimum_coverage: Vec2,
    repeat: ParallaxAxes,
    source_world_size: Vec2,
) -> Vec2 {
    let repeated_target = (viewport_size + margin.max(Vec2::ZERO) * 2.0).max(source_world_size);
    Vec2::new(
        if repeat.x {
            repeated_target.x.max(minimum_coverage.x)
        } else {
            source_world_size.x.max(minimum_coverage.x)
        },
        if repeat.y {
            repeated_target.y.max(minimum_coverage.y)
        } else {
            source_world_size.y.max(minimum_coverage.y)
        },
    )
}

pub(crate) fn required_segment_count(
    viewport_span: f32,
    segment_world_span: f32,
    extra_rings: u32,
) -> u32 {
    if viewport_span <= 0.0 || segment_world_span <= f32::EPSILON {
        return 1;
    }

    let visible_half_count = (viewport_span * 0.5 / segment_world_span).ceil() as u32;
    let half_count = visible_half_count + extra_rings;
    half_count * 2 + 1
}

pub(crate) fn required_segment_grid(
    viewport_size: Vec2,
    segment_world_size: Vec2,
    repeat: ParallaxAxes,
    extra_rings: UVec2,
) -> UVec2 {
    UVec2::new(
        if repeat.x {
            required_segment_count(viewport_size.x, segment_world_size.x, extra_rings.x)
        } else {
            1
        },
        if repeat.y {
            required_segment_count(viewport_size.y, segment_world_size.y, extra_rings.y)
        } else {
            1
        },
    )
}

pub(crate) fn centered_indices(count: u32) -> impl Iterator<Item = i32> {
    let half = (count as i32) / 2;
    (-half)..=half
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;
