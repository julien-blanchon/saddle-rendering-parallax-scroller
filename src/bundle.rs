use bevy::prelude::*;

use crate::{ParallaxLayer, ParallaxRig};

#[derive(Bundle, Default)]
pub struct ParallaxRigBundle {
    pub rig: ParallaxRig,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
pub struct ParallaxLayerBundle {
    pub layer: ParallaxLayer,
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
