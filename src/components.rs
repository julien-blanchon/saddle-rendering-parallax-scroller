use bevy::prelude::*;
use bevy::reflect::Reflect;

use crate::config::{
    ParallaxAxes, ParallaxBounds, ParallaxDepthMapping, ParallaxLayerStrategy, ParallaxSnap,
};

// ---------------------------------------------------------------------------
// Runtime state components (readable by user systems)
// ---------------------------------------------------------------------------

/// Per-rig runtime state. Populated by `TrackCamera`. Read-only for consumers.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct RigRuntimeState {
    pub camera_target: Option<Entity>,
    pub camera_position: Vec2,
    pub camera_depth: f32,
    pub camera_is_perspective: bool,
    pub viewport_size: Vec2,
}

/// Per-layer runtime state. Populated by `ComputeOffsets`. Read-only for consumers.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct LayerRuntimeState {
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

/// Intermediate computed values written by `ComputeOffsets`, read by `WriteTransforms`.
///
/// **Custom offset hook**: schedule your system
/// `after(ParallaxScrollerSystems::ComputeOffsets).before(ParallaxScrollerSystems::WriteTransforms)`
/// and mutate this component to inject wobble, shake, event bursts, etc.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct ParallaxLayerComputed {
    /// Computed parallax offset (before user_offset is applied).
    pub offset: Vec2,
    /// Computed effective scale (before user_scale is applied).
    pub scale: Vec2,
    /// The layer's depth value, passed through for convenience.
    pub depth: f32,
}

/// Internal marker for managed segment children. Not public API.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ManagedSegment {
    pub grid: IVec2,
}

#[derive(Component, Debug, Clone, Reflect)]
#[require(
    Transform,
    GlobalTransform,
    Visibility,
    InheritedVisibility,
    ViewVisibility
)]
#[reflect(Component, Debug, Default)]
pub struct ParallaxRig {
    pub enabled: bool,
    pub origin: Vec2,
    /// Multiplier applied to auto-scroll dt for all child layers.
    /// `1.0` = normal speed, `2.0` = double, `0.0` = frozen auto-scroll.
    /// Does not affect camera-factor (spatial parallax ratio).
    pub speed_multiplier: f32,
}

impl Default for ParallaxRig {
    fn default() -> Self {
        Self {
            enabled: true,
            origin: Vec2::ZERO,
            speed_multiplier: 1.0,
        }
    }
}

impl ParallaxRig {
    pub fn with_speed_multiplier(mut self, speed_multiplier: f32) -> Self {
        self.speed_multiplier = speed_multiplier;
        self
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect, PartialEq, Eq)]
#[reflect(Component, Debug, PartialEq)]
pub struct ParallaxCameraTarget {
    pub camera: Entity,
}

impl ParallaxCameraTarget {
    pub const fn new(camera: Entity) -> Self {
        Self { camera }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[require(
    Sprite,
    Transform,
    GlobalTransform,
    Visibility,
    InheritedVisibility,
    ViewVisibility
)]
#[reflect(Component, Debug, Default)]
pub struct ParallaxLayer {
    pub enabled: bool,
    pub camera_factor: Vec2,
    pub auto_scroll: Vec2,
    pub origin: Vec2,
    pub phase: Vec2,
    pub depth: f32,
    pub depth_mapping: Option<ParallaxDepthMapping>,
    pub repeat: ParallaxAxes,
    pub bounds: ParallaxBounds,
    pub snap: ParallaxSnap,
    pub coverage_margin: Vec2,
    pub source_size: Option<Vec2>,
    pub scale: Vec2,
    pub tint: Color,
    pub strategy: ParallaxLayerStrategy,
    /// User-controlled offset added on top of the computed parallax offset.
    /// Write this from your own systems to add wobble, shake, etc.
    pub user_offset: Vec2,
    /// User-controlled scale multiplied on top of the computed parallax scale.
    /// Defaults to `Vec2::ONE` (no effect).
    pub user_scale: Vec2,
    /// Rotation in radians applied to the layer transform (around Z axis).
    pub rotation: f32,
}

impl ParallaxLayer {
    pub fn tiled() -> Self {
        Self::default()
    }

    pub fn segmented() -> Self {
        Self {
            strategy: ParallaxLayerStrategy::Segmented(Default::default()),
            ..default()
        }
    }

    pub fn with_camera_factor(mut self, camera_factor: Vec2) -> Self {
        self.camera_factor = camera_factor;
        self
    }

    pub fn with_auto_scroll(mut self, auto_scroll: Vec2) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    pub fn with_repeat(mut self, repeat: ParallaxAxes) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn with_depth_mapping(mut self, depth_mapping: ParallaxDepthMapping) -> Self {
        self.depth_mapping = Some(depth_mapping);
        self
    }

    pub fn with_bounds(mut self, bounds: ParallaxBounds) -> Self {
        self.bounds = bounds;
        self
    }

    pub fn with_snap(mut self, snap: ParallaxSnap) -> Self {
        self.snap = snap;
        self
    }

    pub fn with_source_size(mut self, source_size: Vec2) -> Self {
        self.source_size = Some(source_size);
        self
    }

    pub fn with_origin(mut self, origin: Vec2) -> Self {
        self.origin = origin;
        self
    }

    pub fn with_phase(mut self, phase: Vec2) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    pub fn with_coverage_margin(mut self, coverage_margin: Vec2) -> Self {
        self.coverage_margin = coverage_margin.max(Vec2::ZERO);
        self
    }

    pub fn with_depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_strategy(mut self, strategy: ParallaxLayerStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_user_offset(mut self, user_offset: Vec2) -> Self {
        self.user_offset = user_offset;
        self
    }

    pub fn with_user_scale(mut self, user_scale: Vec2) -> Self {
        self.user_scale = user_scale;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

impl Default for ParallaxLayer {
    fn default() -> Self {
        Self {
            enabled: true,
            camera_factor: Vec2::ONE,
            auto_scroll: Vec2::ZERO,
            origin: Vec2::ZERO,
            phase: Vec2::ZERO,
            depth: 0.0,
            depth_mapping: None,
            repeat: ParallaxAxes::both(),
            bounds: ParallaxBounds::default(),
            snap: ParallaxSnap::default(),
            coverage_margin: Vec2::ZERO,
            source_size: None,
            scale: Vec2::ONE,
            tint: Color::WHITE,
            strategy: ParallaxLayerStrategy::default(),
            user_offset: Vec2::ZERO,
            user_scale: Vec2::ONE,
            rotation: 0.0,
        }
    }
}
