//! Full game demo — a platformer scene with real pixel-art assets and all new features.
//!
//! Uses the CC0 "Parallax Background Forest" asset pack by iLkra (OpenGameArt.org).
//! 8 layers: sky, sun, clouds, far mountains, mid mountains, near mountains, rocks, trees.
//!
//! Controls:
//! - **A/D** or **Left/Right**: move player
//! - **Space / W / Up**: jump
//! - **Shift**: run (2x speed)
//! - **1**: toggle wobble effect on trees
//! - **2**: toggle time pause
//! - **3**: cycle speed multiplier (1x → 2x → 0.5x → 1x)
//!
//! Demonstrates:
//! - Real game assets with proper layer ordering and alignment
//! - Camera following the player with directional lead offset
//! - `ComputeOffsets`/`WriteTransforms` hook (wobble on trees layer)
//! - `ParallaxTimeScale` (pause/resume)
//! - `ParallaxRig::speed_multiplier`
//! - Landing screen shake via the custom offset hook

#[cfg(feature = "e2e")]
mod e2e;

use bevy::{
    image::{ImageAddressMode, ImageSamplerDescriptor},
    prelude::*,
};
use saddle_pane::prelude::*;
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxLayer, ParallaxLayerComputed,
    ParallaxLayerStrategy, ParallaxRig, ParallaxScrollerSystems, ParallaxSegmented,
    ParallaxTimeScale,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const LAYER_WIDTH: f32 = 1280.0;
const LAYER_HEIGHT: f32 = 360.0;
/// Vertical offset applied to the entire parallax rig so that the ground
/// (bottom of the images) appears in the lower third of the screen instead of
/// at the very bottom edge.  All layers shift up by this amount.
const LAYER_Y_SHIFT: f32 = 200.0;
/// The Y position of the ground plane in world space.
/// The rocks content sits at pixel rows 340-359 of a 360px image (scale 2x),
/// which places the rock tops at Y = LAYER_Y_SHIFT - 320 = -120.
const GROUND_Y: f32 = -320.0 + LAYER_Y_SHIFT;
const PLAYER_SIZE: f32 = 40.0;
const GRAVITY: f32 = -1100.0;
const JUMP_SPEED: f32 = 450.0;
const MOVE_SPEED: f32 = 280.0;
const RUN_MULT: f32 = 1.8;
/// How fast the camera catches up to the target. Higher = snappier.
const CAM_SMOOTH: f32 = 3.5;
/// Horizontal lead offset: the camera looks ahead in the player's movement direction.
const CAM_LEAD_X: f32 = 250.0;
/// How fast the lead offset transitions when the player changes direction.
const CAM_LEAD_SMOOTH: f32 = 2.5;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Player {
    pub velocity: Vec2,
    pub grounded: bool,
    pub facing: f32, // -1.0, 0.0, or 1.0
}

#[derive(Component)]
struct GameCamera {
    lead_x: f32, // current smoothed lead offset
}

#[derive(Component)]
struct TreeWobble;

#[derive(Component)]
struct CloudDrift;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct ShakeTimer(f32);

#[derive(Resource)]
struct WobbleEnabled(bool);

#[derive(Resource)]
struct SpeedState(usize);

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Game Demo", position = "top-right")]
struct GamePane {
    #[pane(monitor)]
    player_x: f32,
    #[pane(monitor)]
    player_y: f32,
    #[pane(monitor)]
    grounded: f32,
    #[pane(monitor)]
    time_scale: f32,
    #[pane(monitor)]
    speed_mult: f32,
    #[pane(monitor)]
    wobble: f32,
}

impl Default for GamePane {
    fn default() -> Self {
        Self {
            player_x: 0.0,
            player_y: 0.0,
            grounded: 1.0,
            time_scale: 1.0,
            speed_mult: 1.0,
            wobble: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Parallax Game Demo — Forest Platformer".into(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mag_filter: bevy::image::ImageFilterMode::Nearest,
                    min_filter: bevy::image::ImageFilterMode::Nearest,
                    ..default()
                },
            }),
    );
    app.add_plugins(saddle_rendering_parallax_scroller::ParallaxScrollerPlugin::default());
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::GameDemoE2EPlugin);
    app.add_plugins(PanePlugin);
    app.insert_resource(ClearColor(Color::srgb(0.33, 0.50, 0.55)));
    app.insert_resource(ShakeTimer(0.0));
    app.insert_resource(WobbleEnabled(false));
    app.insert_resource(SpeedState(0));
    app.insert_resource(GamePane::default());

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            player_input,
            player_physics,
            camera_follow,
            feature_toggles,
            update_pane,
        )
            .chain(),
    );
    // Custom offset hook: wobble + shake
    app.add_systems(
        Update,
        (wobble_system, shake_system)
            .after(ParallaxScrollerSystems::ComputeOffsets)
            .before(ParallaxScrollerSystems::WriteTransforms),
    );
    app.run();
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera — starts at the scene center
    let camera = commands
        .spawn((
            Name::new("Camera"),
            Camera2d,
            GameCamera { lead_x: 0.0 },
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // Parallax rig — origin.y shifts all layers up so the ground is visible
    let rig = commands
        .spawn((
            Name::new("Forest Rig"),
            ParallaxRig {
                origin: Vec2::new(0.0, LAYER_Y_SHIFT),
                ..default()
            },
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    // Load textures
    let sky: Handle<Image> = asset_server.load("parallax/sky.png");
    let sun: Handle<Image> = asset_server.load("parallax/sun.png");
    let clouds: Handle<Image> = asset_server.load("parallax/clouds.png");
    let mountains_far: Handle<Image> = asset_server.load("parallax/mountains_far.png");
    let mountains_mid: Handle<Image> = asset_server.load("parallax/mountains_mid.png");
    let mountains_near: Handle<Image> = asset_server.load("parallax/mountains_near.png");
    let rocks: Handle<Image> = asset_server.load("parallax/rocks.png");
    let trees: Handle<Image> = asset_server.load("parallax/trees.png");

    // -----------------------------------------------------------------------
    // Parallax layers (back to front)
    //
    // camera_factor semantics:
    //   1.0 = locked to camera (stays fixed on screen)
    //   0.0 = world-fixed (scrolls past camera fully)
    //   <1.0 = drifts slowly → appears distant
    //   >1.0 = moves faster than camera → appears close (foreground)
    //
    // For a platformer with camera-follow:
    //   - Sky: 1.0 (locked, never scrolls)
    //   - Far background: ~0.85-0.95 (very slow drift)
    //   - Mid background: ~0.6-0.8
    //   - Near background: ~0.3-0.5
    //   - Foreground: ~1.05-1.15 (moves faster, feels close)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Vertical camera_factor is 0 for ALL layers.
    // This keeps them world-fixed vertically so they never drift relative
    // to the player when the camera tracks jumps.  Only the horizontal
    // factor varies (creating the classic side-scroller parallax).
    // -----------------------------------------------------------------------

    // 0) Sky — tiled, fills viewport
    commands.spawn((
        Name::new("Sky"),
        ChildOf(rig),
        Sprite::from_image(sky),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::new(1.0, 0.0))
            .with_repeat(ParallaxAxes::both())
            .with_source_size(Vec2::new(LAYER_WIDTH, LAYER_HEIGHT))
            .with_scale(Vec2::splat(2.0))
            .with_coverage_margin(Vec2::new(200.0, 200.0))
            .with_depth(0.0),
    ));

    // 1) Sun — almost locked horizontally
    commands.spawn((
        Name::new("Sun"),
        ChildOf(rig),
        Sprite::from_image(sun),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.97, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_source_size(Vec2::new(LAYER_WIDTH, LAYER_HEIGHT))
            .with_scale(Vec2::splat(2.0))
            .with_depth(0.5),
    ));

    // 2) Clouds — slow drift + gentle auto-scroll
    commands.spawn((
        Name::new("Clouds"),
        ChildOf(rig),
        Sprite::from_image(clouds),
        CloudDrift,
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.92, 0.0))
            .with_auto_scroll(Vec2::new(-6.0, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_source_size(Vec2::new(LAYER_WIDTH, LAYER_HEIGHT))
            .with_scale(Vec2::splat(2.0))
            .with_depth(1.0),
    ));

    // 3) Far mountains — slow parallax, appears very distant
    commands.spawn((
        Name::new("Far Mountains"),
        ChildOf(rig),
        Sprite::from_image(mountains_far),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.85, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_source_size(Vec2::new(LAYER_WIDTH, LAYER_HEIGHT))
            .with_scale(Vec2::splat(2.0))
            .with_depth(2.0),
    ));

    // 4) Mid mountains — medium parallax
    commands.spawn((
        Name::new("Mid Mountains"),
        ChildOf(rig),
        Sprite::from_image(mountains_mid),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.72, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_source_size(Vec2::new(LAYER_WIDTH, LAYER_HEIGHT))
            .with_scale(Vec2::splat(2.0))
            .with_depth(3.0),
    ));

    // 5) Near mountains — noticeable scroll
    commands.spawn((
        Name::new("Near Mountains"),
        ChildOf(rig),
        Sprite::from_image(mountains_near),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.55, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_source_size(Vec2::new(LAYER_WIDTH, LAYER_HEIGHT))
            .with_scale(Vec2::splat(2.0))
            .with_depth(4.0),
    ));

    // 6) Rocks — ground-level, scrolls past noticeably
    commands.spawn((
        Name::new("Rocks"),
        ChildOf(rig),
        Sprite::from_image(rocks),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.35, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_source_size(Vec2::new(LAYER_WIDTH, LAYER_HEIGHT))
            .with_scale(Vec2::splat(2.0))
            .with_depth(5.0),
    ));

    // 7) Trees — foreground, moves faster than camera horizontally
    commands.spawn((
        Name::new("Trees"),
        ChildOf(rig),
        Sprite::from_image(trees),
        TreeWobble,
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(1.18, 0.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_source_size(Vec2::new(LAYER_WIDTH, LAYER_HEIGHT))
            .with_scale(Vec2::splat(2.0))
            .with_depth(6.0)
            .with_strategy(ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(3, 0),
            })),
    ));

    // Underground fill — solid brown below the parallax layers.
    // With LAYER_Y_SHIFT, layer bottoms sit at Y = LAYER_Y_SHIFT - 360 = -160.
    // A tall strip covers everything below that.
    commands.spawn((
        Name::new("Underground"),
        Sprite::from_color(Color::srgb(0.28, 0.20, 0.13), Vec2::new(100_000.0, 800.0)),
        Transform::from_xyz(0.0, -160.0 - 400.0, 7.0),
    ));

    // Player — colored square
    commands.spawn((
        Name::new("Player"),
        Player {
            velocity: Vec2::ZERO,
            grounded: true,
            facing: 1.0,
        },
        Sprite::from_color(Color::srgb(0.95, 0.30, 0.20), Vec2::splat(PLAYER_SIZE)),
        Transform::from_xyz(0.0, GROUND_Y + PLAYER_SIZE * 0.5, 10.0),
    ));
}

// ---------------------------------------------------------------------------
// Player systems
// ---------------------------------------------------------------------------

fn player_input(keys: Res<ButtonInput<KeyCode>>, mut players: Query<&mut Player>) {
    for mut player in &mut players {
        let mut dir = 0.0;
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            dir -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            dir += 1.0;
        }
        let run = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            RUN_MULT
        } else {
            1.0
        };
        player.velocity.x = dir * MOVE_SPEED * run;

        // Track facing direction for camera lead
        if dir != 0.0 {
            player.facing = dir;
        }

        if player.grounded
            && (keys.just_pressed(KeyCode::Space)
                || keys.just_pressed(KeyCode::KeyW)
                || keys.just_pressed(KeyCode::ArrowUp))
        {
            player.velocity.y = JUMP_SPEED;
            player.grounded = false;
        }
    }
}

fn player_physics(
    time: Res<Time>,
    mut players: Query<(&mut Player, &mut Transform)>,
    mut shake: ResMut<ShakeTimer>,
) {
    let dt = time.delta_secs();
    for (mut player, mut transform) in &mut players {
        player.velocity.y += GRAVITY * dt;
        transform.translation.x += player.velocity.x * dt;
        transform.translation.y += player.velocity.y * dt;

        let floor = GROUND_Y + PLAYER_SIZE * 0.5;
        if transform.translation.y <= floor {
            if !player.grounded && player.velocity.y < -200.0 {
                shake.0 = 0.15;
            }
            transform.translation.y = floor;
            player.velocity.y = 0.0;
            player.grounded = true;
        }
    }
}

/// Camera follows player with a directional lead offset.
/// The camera looks ahead in the direction the player is facing,
/// so the player is offset from screen center and you can see what's coming.
/// The camera Y is mostly fixed (slight tracking for jumps) so the parallax
/// layers stay properly composed.
fn camera_follow(
    time: Res<Time>,
    players: Query<(&Player, &Transform), Without<GameCamera>>,
    mut cameras: Query<(&mut GameCamera, &mut Transform), Without<Player>>,
) {
    let Ok((player, player_transform)) = players.single() else {
        return;
    };
    let dt = time.delta_secs();

    for (mut cam_state, mut cam_transform) in &mut cameras {
        // Smooth the lead offset so direction changes don't snap instantly
        let target_lead = player.facing * CAM_LEAD_X;
        cam_state.lead_x += (target_lead - cam_state.lead_x) * CAM_LEAD_SMOOTH * dt;

        let target_x = player_transform.translation.x + cam_state.lead_x;

        // Camera Y: mostly fixed at scene center, only follows large jumps slightly
        // This keeps the parallax layers properly composed
        let base_y = 0.0; // scene center
        let player_offset_y = (player_transform.translation.y - GROUND_Y).max(0.0);
        let target_y = base_y + player_offset_y * 0.3; // only 30% of jump height

        cam_transform.translation.x += (target_x - cam_transform.translation.x) * CAM_SMOOTH * dt;
        cam_transform.translation.y += (target_y - cam_transform.translation.y) * CAM_SMOOTH * dt;
    }
}

// ---------------------------------------------------------------------------
// Feature toggle system
// ---------------------------------------------------------------------------

fn feature_toggles(
    keys: Res<ButtonInput<KeyCode>>,
    mut wobble_enabled: ResMut<WobbleEnabled>,
    mut time_scale: ResMut<ParallaxTimeScale>,
    mut speed_state: ResMut<SpeedState>,
    mut rigs: Query<&mut ParallaxRig>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        wobble_enabled.0 = !wobble_enabled.0;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        time_scale.0 = if time_scale.0 > 0.0 { 0.0 } else { 1.0 };
    }
    if keys.just_pressed(KeyCode::Digit3) {
        let speeds = [1.0, 2.0, 0.5];
        speed_state.0 = (speed_state.0 + 1) % speeds.len();
        let new_speed = speeds[speed_state.0];
        for mut rig in &mut rigs {
            rig.speed_multiplier = new_speed;
        }
    }
}

// ---------------------------------------------------------------------------
// Custom offset hooks
// ---------------------------------------------------------------------------

fn wobble_system(
    time: Res<Time>,
    wobble_enabled: Res<WobbleEnabled>,
    mut layers: Query<&mut ParallaxLayerComputed, With<TreeWobble>>,
) {
    if !wobble_enabled.0 {
        return;
    }
    let t = time.elapsed_secs();
    for mut computed in &mut layers {
        computed.offset.x += (t * 1.5).sin() * 3.0;
        computed.offset.y += (t * 2.0).cos() * 2.0;
    }
}

fn shake_system(
    time: Res<Time>,
    mut shake: ResMut<ShakeTimer>,
    mut layers: Query<&mut ParallaxLayerComputed>,
) {
    if shake.0 <= 0.0 {
        return;
    }
    shake.0 -= time.delta_secs();
    let intensity = shake.0.max(0.0) * 40.0;
    let t = time.elapsed_secs();
    let jx = ((t * 73.0).sin() + (t * 137.0).cos()) * intensity;
    let jy = ((t * 97.0).cos() + (t * 53.0).sin()) * intensity;
    for mut computed in &mut layers {
        computed.offset.x += jx;
        computed.offset.y += jy;
    }
}

// ---------------------------------------------------------------------------
// Pane update
// ---------------------------------------------------------------------------

fn update_pane(
    mut pane: ResMut<GamePane>,
    players: Query<(&Player, &Transform)>,
    time_scale: Res<ParallaxTimeScale>,
    rigs: Query<&ParallaxRig>,
    wobble_enabled: Res<WobbleEnabled>,
) {
    if let Ok((player, transform)) = players.single() {
        pane.player_x = transform.translation.x;
        pane.player_y = transform.translation.y;
        pane.grounded = if player.grounded { 1.0 } else { 0.0 };
    }
    pane.time_scale = time_scale.0;
    pane.speed_mult = rigs.iter().next().map_or(1.0, |r| r.speed_multiplier);
    pane.wobble = if wobble_enabled.0 { 1.0 } else { 0.0 };
}
