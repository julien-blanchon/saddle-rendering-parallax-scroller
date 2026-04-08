#![allow(deprecated)]

use bevy::prelude::*;

use crate::{ParallaxLayer, ParallaxRig};

#[deprecated(
    note = "Use ParallaxRig directly; required components are auto-inserted via #[require(...)]"
)]
#[derive(Bundle, Default)]
pub struct ParallaxRigBundle {
    pub rig: ParallaxRig,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[deprecated(
    note = "Use ParallaxLayer directly; required components are auto-inserted via #[require(...)]"
)]
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
