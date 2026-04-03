use bevy::prelude::*;
use bevy::reflect::Reflect;

use crate::{ParallaxAxes, ParallaxStrategyKind};

#[derive(Resource, Default)]
pub(crate) struct ParallaxRuntimeState {
    pub active: bool,
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource, Debug, Default)]
pub struct ParallaxDebugSettings {
    pub enabled: bool,
    pub draw_viewport_bounds: bool,
    pub draw_layer_bounds: bool,
    pub draw_offsets: bool,
    pub viewport_color: Color,
    pub coverage_color: Color,
    pub wrap_color: Color,
    pub offset_color: Color,
}

impl Default for ParallaxDebugSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            draw_viewport_bounds: true,
            draw_layer_bounds: true,
            draw_offsets: false,
            viewport_color: Color::srgba(0.18, 0.90, 0.95, 0.80),
            coverage_color: Color::srgba(0.28, 0.95, 0.56, 0.80),
            wrap_color: Color::srgba(0.98, 0.68, 0.22, 0.80),
            offset_color: Color::srgba(0.98, 0.32, 0.32, 0.90),
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Reflect)]
#[reflect(Resource, Debug, Default)]
pub struct ParallaxDiagnostics {
    pub runtime_active: bool,
    pub rigs: Vec<ParallaxRigDiagnostics>,
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Default)]
pub struct ParallaxRigDiagnostics {
    pub rig: Entity,
    pub camera_target: Option<Entity>,
    pub camera_position: Vec2,
    pub viewport_size: Vec2,
    pub enabled: bool,
    pub layers: Vec<ParallaxLayerDiagnostics>,
}

impl Default for ParallaxRigDiagnostics {
    fn default() -> Self {
        Self {
            rig: Entity::PLACEHOLDER,
            camera_target: None,
            camera_position: Vec2::ZERO,
            viewport_size: Vec2::ZERO,
            enabled: false,
            layers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Reflect)]
#[reflect(Debug, Default)]
pub struct ParallaxLayerDiagnostics {
    pub layer: Entity,
    pub strategy: ParallaxStrategyKind,
    pub repeat: ParallaxAxes,
    pub effective_camera_factor: Vec2,
    pub effective_scale: Vec2,
    pub effective_offset: Vec2,
    pub depth_ratio: Option<f32>,
    pub wrap_span: Vec2,
    pub coverage_size: Vec2,
    pub segment_grid: UVec2,
}

impl Default for ParallaxLayerDiagnostics {
    fn default() -> Self {
        Self {
            layer: Entity::PLACEHOLDER,
            strategy: Default::default(),
            repeat: Default::default(),
            effective_camera_factor: Vec2::ZERO,
            effective_scale: Vec2::ONE,
            effective_offset: Vec2::ZERO,
            depth_ratio: None,
            wrap_span: Vec2::ZERO,
            coverage_size: Vec2::ZERO,
            segment_grid: UVec2::ONE,
        }
    }
}
