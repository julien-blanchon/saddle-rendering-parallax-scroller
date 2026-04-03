use bevy::prelude::*;
use bevy::reflect::Reflect;

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq)]
#[reflect(Debug, PartialEq, Default)]
pub struct ParallaxAxes {
    pub x: bool,
    pub y: bool,
}

impl ParallaxAxes {
    pub const fn none() -> Self {
        Self { x: false, y: false }
    }

    pub const fn horizontal() -> Self {
        Self { x: true, y: false }
    }

    pub const fn vertical() -> Self {
        Self { x: false, y: true }
    }

    pub const fn both() -> Self {
        Self { x: true, y: true }
    }
}

impl Default for ParallaxAxes {
    fn default() -> Self {
        Self::both()
    }
}

#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
#[reflect(Debug, PartialEq)]
pub struct AxisRange {
    pub min: f32,
    pub max: f32,
}

impl AxisRange {
    pub fn new(min: f32, max: f32) -> Self {
        if min <= max {
            Self { min, max }
        } else {
            Self { min: max, max: min }
        }
    }

    pub(crate) fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq)]
#[reflect(Debug, PartialEq, Default)]
pub struct ParallaxBounds {
    pub x: Option<AxisRange>,
    pub y: Option<AxisRange>,
}

impl ParallaxBounds {
    pub fn horizontal(min: f32, max: f32) -> Self {
        Self {
            x: Some(AxisRange::new(min, max)),
            y: None,
        }
    }

    pub fn vertical(min: f32, max: f32) -> Self {
        Self {
            x: None,
            y: Some(AxisRange::new(min, max)),
        }
    }

    pub fn xy(x: AxisRange, y: AxisRange) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq)]
#[reflect(Debug, PartialEq, Default)]
pub struct ParallaxDepthMapping {
    pub reference_plane_z: f32,
    pub translation_response: Vec2,
    pub scale_response: f32,
}

impl Default for ParallaxDepthMapping {
    fn default() -> Self {
        Self {
            reference_plane_z: 0.0,
            translation_response: Vec2::ONE,
            scale_response: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default, Reflect, PartialEq)]
#[reflect(Debug, PartialEq, Default)]
pub enum ParallaxSnap {
    #[default]
    None,
    Pixel,
    Grid(Vec2),
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
#[reflect(Debug, PartialEq, Default)]
pub enum ParallaxStrategyKind {
    #[default]
    TiledSprite,
    Segmented,
}

#[derive(Clone, Debug, Reflect, PartialEq)]
#[reflect(Debug, PartialEq, Default)]
pub struct ParallaxTiledSprite {
    pub stretch_value: f32,
    pub minimum_coverage: Vec2,
}

impl Default for ParallaxTiledSprite {
    fn default() -> Self {
        Self {
            stretch_value: 1.0,
            minimum_coverage: Vec2::ZERO,
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq)]
#[reflect(Debug, PartialEq, Default)]
pub struct ParallaxSegmented {
    pub extra_rings: UVec2,
}

impl Default for ParallaxSegmented {
    fn default() -> Self {
        Self {
            extra_rings: UVec2::ONE,
        }
    }
}

#[derive(Clone, Debug, Reflect, PartialEq)]
#[reflect(Debug, PartialEq, Default)]
pub enum ParallaxLayerStrategy {
    TiledSprite(ParallaxTiledSprite),
    Segmented(ParallaxSegmented),
}

impl ParallaxLayerStrategy {
    pub fn kind(&self) -> ParallaxStrategyKind {
        match self {
            Self::TiledSprite(_) => ParallaxStrategyKind::TiledSprite,
            Self::Segmented(_) => ParallaxStrategyKind::Segmented,
        }
    }
}

impl Default for ParallaxLayerStrategy {
    fn default() -> Self {
        Self::TiledSprite(ParallaxTiledSprite::default())
    }
}
