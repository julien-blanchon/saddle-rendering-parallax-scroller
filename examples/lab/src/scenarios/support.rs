use bevy::prelude::*;
use saddle_bevy_e2e::action::Action;
use saddle_rendering_parallax_scroller::{
    LayerRuntimeState, ParallaxDiagnostics, RigRuntimeState,
};

use crate::{LabMode, set_lab_mode};

pub fn mode_action(mode: LabMode) -> Action {
    Action::Custom(Box::new(move |world| set_lab_mode(world, mode)))
}

pub fn rig(world: &World, entity: Entity) -> Option<RigRuntimeState> {
    world
        .resource::<ParallaxDiagnostics>()
        .rigs
        .iter()
        .find(|rig| rig.rig == entity)
        .cloned()
}

pub fn layer(world: &World, rig: Entity, layer: Entity) -> Option<LayerRuntimeState> {
    rig(world, rig)?
        .layers
        .into_iter()
        .find(|candidate| candidate.layer == layer)
}

pub fn first_layer_offset(world: &World, rig: Entity) -> Option<Vec2> {
    rig(world, rig)?.layers.first().map(|layer| layer.effective_offset)
}

pub fn first_layer_coverage(world: &World, rig: Entity) -> Option<Vec2> {
    rig(world, rig)?.layers.first().map(|layer| layer.coverage_size)
}

pub fn viewport_size(world: &World, rig: Entity) -> Option<Vec2> {
    rig(world, rig).map(|rig| rig.viewport_size)
}

pub fn layer_count(world: &World, rig: Entity) -> Option<usize> {
    rig(world, rig).map(|rig| rig.layers.len())
}
