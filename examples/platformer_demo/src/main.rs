//! Platformer demo — keyboard-controlled character with parallax background.
//!
//! Demonstrates:
//! - Camera-driven parallax responding to player movement (not auto-scroll)
//! - Keyboard input: Arrow keys / WASD to move, Space to jump
//! - Camera follows the player with smooth lerp
//! - Rich forest parallax reacts naturally to camera tracking
//!
//! Controls:
//! - Arrow keys / WASD: Move left/right
//! - Space: Jump
//! - The parallax layers respond to camera movement as the player moves

use bevy::prelude::*;

use common::{add_rich_forest_stack, configure_app, demo_textures};
use saddle_rendering_parallax_scroller::{ParallaxCameraTarget, ParallaxRig};
use saddle_rendering_parallax_scroller_example_common as common;

const MOVE_SPEED: f32 = 250.0;
const JUMP_VELOCITY: f32 = 450.0;
const GRAVITY: f32 = -900.0;
const GROUND_Y: f32 = -250.0;
const CAMERA_LERP_SPEED: f32 = 4.0;

#[derive(Component)]
struct Player {
    velocity: Vec2,
    on_ground: bool,
}

#[derive(Component)]
struct FollowCamera;

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (player_input, player_physics, camera_follow).chain(),
    );
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);

    // Camera
    let camera = commands
        .spawn((
            Name::new("Platformer Camera"),
            Camera2d,
            FollowCamera,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            Transform::default(),
        ))
        .id();

    // Parallax rig
    let rig = commands
        .spawn((
            Name::new("Platformer Parallax Rig"),
            ParallaxRig {
                enabled: true,
                origin: Vec2::ZERO,
                ..default()
            },
            Transform::from_translation(Vec3::ZERO),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    add_rich_forest_stack(&mut commands, rig, &textures);

    // Player character (simple colored rectangle)
    commands.spawn((
        Name::new("Player"),
        Player {
            velocity: Vec2::ZERO,
            on_ground: true,
        },
        Sprite::from_color(Color::srgb(0.2, 0.8, 0.4), Vec2::new(32.0, 48.0)),
        Transform::from_xyz(0.0, GROUND_Y + 24.0, 50.0),
    ));

    // Ground line indicator
    commands.spawn((
        Name::new("Ground Line"),
        Sprite::from_color(Color::srgba(0.3, 0.2, 0.1, 0.5), Vec2::new(10000.0, 4.0)),
        Transform::from_xyz(0.0, GROUND_Y, 45.0),
    ));

    // On-screen instructions
    commands.spawn((
        Name::new("Instructions"),
        Text::new("Platformer Demo — WASD/Arrows to move, Space to jump\nParallax layers respond to camera following the player"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));
}

fn player_input(keyboard: Res<ButtonInput<KeyCode>>, mut players: Query<&mut Player>) {
    for mut player in &mut players {
        // Horizontal movement
        let mut dir = 0.0;
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            dir -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            dir += 1.0;
        }
        player.velocity.x = dir * MOVE_SPEED;

        // Jump
        if (keyboard.just_pressed(KeyCode::Space)
            || keyboard.just_pressed(KeyCode::ArrowUp)
            || keyboard.just_pressed(KeyCode::KeyW))
            && player.on_ground
        {
            player.velocity.y = JUMP_VELOCITY;
            player.on_ground = false;
        }
    }
}

fn player_physics(time: Res<Time>, mut players: Query<(&mut Player, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut player, mut transform) in &mut players {
        // Apply gravity
        if !player.on_ground {
            player.velocity.y += GRAVITY * dt;
        }

        // Move
        transform.translation.x += player.velocity.x * dt;
        transform.translation.y += player.velocity.y * dt;

        // Ground collision
        if transform.translation.y <= GROUND_Y + 24.0 {
            transform.translation.y = GROUND_Y + 24.0;
            player.velocity.y = 0.0;
            player.on_ground = true;
        }
    }
}

fn camera_follow(
    time: Res<Time>,
    players: Query<&Transform, (With<Player>, Without<FollowCamera>)>,
    mut cameras: Query<&mut Transform, (With<FollowCamera>, Without<Player>)>,
) {
    let Ok(player_transform) = players.single() else {
        return;
    };
    for mut camera_transform in &mut cameras {
        let target = Vec3::new(
            player_transform.translation.x,
            player_transform.translation.y + 50.0,
            camera_transform.translation.z,
        );
        let lerp_factor = (CAMERA_LERP_SPEED * time.delta_secs()).min(1.0);
        camera_transform.translation = camera_transform.translation.lerp(target, lerp_factor);
    }
}
