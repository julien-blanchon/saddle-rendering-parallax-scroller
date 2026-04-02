use bevy::prelude::*;
use bevy::reflect::Reflect;

use crate::config::{ParallaxAxes, ParallaxBounds, ParallaxLayerStrategy, ParallaxSnap};

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
}

impl Default for ParallaxRig {
    fn default() -> Self {
        Self {
            enabled: true,
            origin: Vec2::ZERO,
        }
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
    pub repeat: ParallaxAxes,
    pub bounds: ParallaxBounds,
    pub snap: ParallaxSnap,
    pub coverage_margin: Vec2,
    pub source_size: Option<Vec2>,
    pub scale: Vec2,
    pub tint: Color,
    pub strategy: ParallaxLayerStrategy,
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
            repeat: ParallaxAxes::both(),
            bounds: ParallaxBounds::default(),
            snap: ParallaxSnap::default(),
            coverage_margin: Vec2::ZERO,
            source_size: None,
            scale: Vec2::ONE,
            tint: Color::WHITE,
            strategy: ParallaxLayerStrategy::default(),
        }
    }
}
