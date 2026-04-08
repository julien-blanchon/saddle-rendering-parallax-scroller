use bevy::prelude::*;

/// Fired when a layer's offset wraps around on a repeating axis.
#[derive(Message, Debug, Clone)]
pub struct ParallaxLayerWrapped {
    pub layer: Entity,
    pub rig: Entity,
    /// The unbounded offset before wrapping.
    pub offset_before_wrap: Vec2,
    /// The offset after wrapping.
    pub offset_after_wrap: Vec2,
}

/// Fired when a managed segment child is spawned for a segmented layer.
#[derive(Message, Debug, Clone)]
pub struct ParallaxSegmentSpawned {
    pub segment: Entity,
    pub layer: Entity,
    pub grid: IVec2,
}

/// Fired when a managed segment child is despawned.
#[derive(Message, Debug, Clone)]
pub struct ParallaxSegmentDespawned {
    pub layer: Entity,
    pub grid: IVec2,
}

/// Fired when the parallax runtime is activated.
#[derive(Message, Debug, Clone)]
pub struct ParallaxActivated;

/// Fired when the parallax runtime is deactivated.
#[derive(Message, Debug, Clone)]
pub struct ParallaxDeactivated;
