use super::*;
use crate::{AxisRange, ParallaxAxes, ParallaxBounds, ParallaxSnap};

#[test]
fn wrap_centered_handles_positive_and_negative_offsets() {
    assert!((wrap_centered(17.0, 10.0) + 3.0).abs() < 0.001);
    assert!((wrap_centered(-17.0, 10.0) - 3.0).abs() < 0.001);
}

#[test]
fn advance_phase_ignores_non_positive_delta_time() {
    let phase = Vec2::new(4.0, -6.0);
    assert_eq!(advance_phase(phase, Vec2::ZERO, 1.0), phase);
    assert_eq!(advance_phase(phase, Vec2::new(12.0, 18.0), 0.0), phase);
    assert_eq!(advance_phase(phase, Vec2::new(12.0, 18.0), -1.0), phase);
}

#[test]
fn compute_unbounded_offset_combines_camera_phase_and_auto_scroll() {
    let offset = compute_unbounded_offset(
        Vec2::new(12.0, -6.0),
        Vec2::new(0.75, 0.5),
        Vec2::new(3.0, 4.0),
        Vec2::new(-2.0, 1.0),
    );
    assert_eq!(offset, Vec2::new(10.0, 2.0));
}

#[test]
fn compute_unbounded_offset_ignores_camera_when_factor_is_zero() {
    let offset = compute_unbounded_offset(
        Vec2::new(240.0, -120.0),
        Vec2::ZERO,
        Vec2::new(6.0, 8.0),
        Vec2::new(-1.0, 3.0),
    );
    assert_eq!(offset, Vec2::new(5.0, 11.0));
}

#[test]
fn repeat_and_bounds_use_repeat_on_enabled_axes_and_clamp_elsewhere() {
    let offset = apply_repeat_and_bounds(
        Vec2::new(18.0, 18.0),
        ParallaxAxes::horizontal(),
        Vec2::new(10.0, 12.0),
        ParallaxBounds {
            x: None,
            y: Some(AxisRange::new(-4.0, 4.0)),
        },
    );
    assert!((offset.x + 2.0).abs() < 0.001);
    assert_eq!(offset.y, 4.0);
}

#[test]
fn snap_offset_supports_pixel_and_grid_modes() {
    assert_eq!(
        snap_offset(Vec2::new(10.4, -2.6), &ParallaxSnap::Pixel),
        Vec2::new(10.0, -3.0)
    );
    assert_eq!(
        snap_offset(
            Vec2::new(10.4, -2.6),
            &ParallaxSnap::Grid(Vec2::new(4.0, 0.5))
        ),
        Vec2::new(12.0, -2.5)
    );
}

#[test]
fn required_segment_count_expands_when_viewport_exceeds_segment_span() {
    assert_eq!(required_segment_count(0.0, 64.0, 1), 1);
    assert_eq!(required_segment_count(128.0, 64.0, 1), 5);
    assert_eq!(required_segment_count(300.0, 64.0, 1), 9);
}

#[test]
fn coverage_size_respects_repeat_axes_and_minimums() {
    let coverage = coverage_size(
        Vec2::new(320.0, 180.0),
        Vec2::new(16.0, 8.0),
        Vec2::new(400.0, 90.0),
        ParallaxAxes::horizontal(),
        Vec2::new(64.0, 48.0),
    );
    assert_eq!(coverage, Vec2::new(400.0, 90.0));
}

#[test]
fn required_segment_grid_only_expands_repeating_axes() {
    let grid = required_segment_grid(
        Vec2::new(512.0, 256.0),
        Vec2::new(128.0, 64.0),
        ParallaxAxes::horizontal(),
        UVec2::new(2, 2),
    );
    assert_eq!(grid, UVec2::new(9, 1));
}
