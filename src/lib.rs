use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    gizmos::{config::DefaultGizmoConfigGroup, gizmos::GizmoStorage},
    prelude::*,
};

mod bundle;
mod components;
mod config;
mod math;
mod resources;
mod systems;

pub use bundle::{ParallaxLayerBundle, ParallaxRigBundle};
pub use components::{ParallaxCameraTarget, ParallaxLayer, ParallaxRig};
pub use config::{
    AxisRange, ParallaxAxes, ParallaxBounds, ParallaxLayerStrategy, ParallaxSegmented,
    ParallaxSnap, ParallaxStrategyKind, ParallaxTiledSprite,
};
pub use resources::{
    ParallaxDebugSettings, ParallaxDiagnostics, ParallaxLayerDiagnostics, ParallaxRigDiagnostics,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ParallaxScrollerSystems {
    TrackCamera,
    UpdateOffsets,
    ApplyLayout,
    Diagnostics,
    Debug,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct ParallaxScrollerPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl ParallaxScrollerPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for ParallaxScrollerPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for ParallaxScrollerPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<resources::ParallaxRuntimeState>()
            .init_resource::<ParallaxDebugSettings>()
            .init_resource::<ParallaxDiagnostics>()
            .register_type::<AxisRange>()
            .register_type::<ParallaxAxes>()
            .register_type::<ParallaxBounds>()
            .register_type::<ParallaxCameraTarget>()
            .register_type::<ParallaxDebugSettings>()
            .register_type::<ParallaxDiagnostics>()
            .register_type::<ParallaxLayer>()
            .register_type::<ParallaxLayerDiagnostics>()
            .register_type::<ParallaxLayerStrategy>()
            .register_type::<ParallaxRig>()
            .register_type::<ParallaxRigDiagnostics>()
            .register_type::<ParallaxSegmented>()
            .register_type::<ParallaxSnap>()
            .register_type::<ParallaxStrategyKind>()
            .register_type::<ParallaxTiledSprite>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    ParallaxScrollerSystems::TrackCamera,
                    ParallaxScrollerSystems::UpdateOffsets,
                    ParallaxScrollerSystems::ApplyLayout,
                    ParallaxScrollerSystems::Diagnostics,
                    ParallaxScrollerSystems::Debug,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (systems::ensure_rig_state, systems::track_camera_targets)
                    .chain()
                    .in_set(ParallaxScrollerSystems::TrackCamera)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                (systems::ensure_layer_state, systems::advance_layer_phase)
                    .chain()
                    .in_set(ParallaxScrollerSystems::UpdateOffsets)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                (systems::apply_layout, systems::sync_segment_children)
                    .chain()
                    .in_set(ParallaxScrollerSystems::ApplyLayout)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::publish_diagnostics.in_set(ParallaxScrollerSystems::Diagnostics),
            )
            .add_systems(
                self.update_schedule,
                systems::draw_debug_gizmos
                    .in_set(ParallaxScrollerSystems::Debug)
                    .run_if(systems::runtime_is_active)
                    .run_if(resource_exists::<GizmoStorage<DefaultGizmoConfigGroup, ()>>),
            );
    }
}
