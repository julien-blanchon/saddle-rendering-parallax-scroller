//! Custom offset hook — demonstrates injecting user logic between `ComputeOffsets` and `WriteTransforms`.
//!
//! Three effects are shown:
//! - **Sine-wave wobble**: vertical oscillation applied to marked layers
//! - **Shake**: random jitter on keypress (Space)
//! - **Scroll burst**: instant horizontal offset spike on keypress (Enter)
//!
//! Press **Space** for shake, **Enter** for a scroll burst. All effects layer
//! on top of the normal parallax pipeline via `ParallaxLayerComputed` mutation.

use bevy::prelude::*;

use common::{DemoCamera, configure_app, demo_textures, update_demo_camera};
use saddle_pane::prelude::*;
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerComputed,
    ParallaxLayerStrategy, ParallaxRig, ParallaxScrollerSystems, ParallaxSegmented,
};
use saddle_rendering_parallax_scroller_example_common as common;

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.init_resource::<ShakeState>();
    app.init_resource::<BurstState>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, (update_demo_camera, input_handler));
    // The custom offset hook — runs between ComputeOffsets and WriteTransforms
    app.add_systems(
        Update,
        (wobble_system, shake_system, burst_system)
            .after(ParallaxScrollerSystems::ComputeOffsets)
            .before(ParallaxScrollerSystems::WriteTransforms),
    );
    app.run();
}

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Wobble {
    frequency: f32,
    amplitude: f32,
}

// ---------------------------------------------------------------------------
// Shake & burst resources
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ShakeState {
    remaining: f32,
    intensity: f32,
}

#[derive(Resource, Default)]
struct BurstState {
    remaining: f32,
    magnitude: f32,
}

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Custom Offset", position = "top-right")]
struct OffsetPane {
    #[pane(slider, min = 0.0, max = 30.0, step = 0.5)]
    wobble_amplitude: f32,
    #[pane(slider, min = 0.0, max = 5.0, step = 0.1)]
    wobble_frequency: f32,
    #[pane(slider, min = 0.0, max = 20.0, step = 0.5)]
    shake_intensity: f32,
    #[pane(slider, min = 0.0, max = 200.0, step = 5.0)]
    burst_magnitude: f32,
    #[pane(monitor)]
    shake_active: f32,
    #[pane(monitor)]
    burst_active: f32,
}

impl Default for OffsetPane {
    fn default() -> Self {
        Self {
            wobble_amplitude: 8.0,
            wobble_frequency: 2.0,
            shake_intensity: 6.0,
            burst_magnitude: 80.0,
            shake_active: 0.0,
            burst_active: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);

    commands.insert_resource(OffsetPane::default());

    let camera = commands
        .spawn((
            Name::new("Demo Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            DemoCamera {
                horizontal_speed: 80.0,
                vertical_amplitude: 16.0,
                zoom_amplitude: 0.15,
            },
            Transform::default(),
        ))
        .id();

    let rig = commands
        .spawn((
            Name::new("Parallax Rig"),
            ParallaxRig::default(),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Sky — locked to camera, no wobble
    commands.spawn((
        Name::new("Sky"),
        ChildOf(rig),
        Sprite::from_image(textures.sky.clone()),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::ONE)
            .with_repeat(ParallaxAxes::both())
            .with_coverage_margin(Vec2::new(96.0, 48.0))
            .with_tint(Color::srgba(0.95, 0.98, 1.0, 0.92))
            .with_scale(Vec2::splat(2.0))
            .with_origin(Vec2::new(0.0, 24.0))
            .with_depth(0.0),
    ));

    // Mountains — segmented, receives wobble
    commands.spawn((
        Name::new("Mountains (wobble)"),
        ChildOf(rig),
        Sprite::from_image(textures.mountains.clone()),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.84, 1.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -96.0))
            .with_depth(1.0)
            .with_scale(Vec2::splat(1.4))
            .with_tint(Color::srgb(0.34, 0.47, 0.56))
            .with_source_size(Vec2::new(320.0, 96.0)),
        Wobble {
            frequency: 2.0,
            amplitude: 8.0,
        },
    ));

    // Canopy — segmented, receives wobble with different params
    commands.spawn((
        Name::new("Canopy (wobble)"),
        ChildOf(rig),
        Sprite::from_image(textures.canopy.clone()),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(1.08, 1.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -200.0))
            .with_depth(2.0)
            .with_scale(Vec2::new(1.5, 2.0))
            .with_tint(Color::srgb(0.14, 0.28, 0.14))
            .with_source_size(Vec2::new(256.0, 64.0))
            .with_strategy(ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(2, 0),
            })),
        Wobble {
            frequency: 3.0,
            amplitude: 5.0,
        },
    ));
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

fn input_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut shake: ResMut<ShakeState>,
    mut burst: ResMut<BurstState>,
    pane: Res<OffsetPane>,
) {
    if keys.just_pressed(KeyCode::Space) {
        shake.remaining = 0.4;
        shake.intensity = pane.shake_intensity;
    }
    if keys.just_pressed(KeyCode::Enter) {
        burst.remaining = 0.3;
        burst.magnitude = pane.burst_magnitude;
    }
}

// ---------------------------------------------------------------------------
// Custom offset systems (the hook point!)
// ---------------------------------------------------------------------------

fn wobble_system(
    time: Res<Time>,
    pane: Res<OffsetPane>,
    mut layers: Query<(&mut ParallaxLayerComputed, &mut Wobble)>,
) {
    let t = time.elapsed_secs();
    for (mut computed, mut wobble) in &mut layers {
        wobble.frequency = pane.wobble_frequency;
        wobble.amplitude = pane.wobble_amplitude;
        computed.offset.y += (t * wobble.frequency * std::f32::consts::TAU).sin() * wobble.amplitude;
    }
}

fn shake_system(
    time: Res<Time>,
    mut shake: ResMut<ShakeState>,
    mut pane: ResMut<OffsetPane>,
    mut layers: Query<&mut ParallaxLayerComputed>,
) {
    if shake.remaining <= 0.0 {
        pane.shake_active = 0.0;
        return;
    }
    shake.remaining -= time.delta_secs();
    pane.shake_active = 1.0;

    let t = time.elapsed_secs();
    let jitter_x = ((t * 47.0).sin() + (t * 113.0).cos()) * shake.intensity;
    let jitter_y = ((t * 73.0).cos() + (t * 97.0).sin()) * shake.intensity;

    for mut computed in &mut layers {
        computed.offset.x += jitter_x;
        computed.offset.y += jitter_y;
    }
}

fn burst_system(
    time: Res<Time>,
    mut burst: ResMut<BurstState>,
    mut pane: ResMut<OffsetPane>,
    mut layers: Query<&mut ParallaxLayerComputed>,
) {
    if burst.remaining <= 0.0 {
        pane.burst_active = 0.0;
        return;
    }
    let progress = burst.remaining / 0.3;
    burst.remaining -= time.delta_secs();
    pane.burst_active = 1.0;

    let offset_x = burst.magnitude * progress;
    for mut computed in &mut layers {
        computed.offset.x += offset_x;
    }
}
