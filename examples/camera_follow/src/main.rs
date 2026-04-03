use saddle_rendering_parallax_scroller_example_common as common;

use bevy::prelude::*;

use common::{
    FollowDot, add_forest_stack, animate_follow_dot, configure_app, demo_textures, spawn_demo_rig,
    spawn_follow_camera, update_demo_camera,
};

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_follow_dot, update_demo_camera));
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);
    let camera = spawn_follow_camera(&mut commands);
    let rig = spawn_demo_rig(&mut commands, camera, "Follow Rig", Vec3::ZERO);
    add_forest_stack(&mut commands, rig, &textures);

    commands.spawn((
        Name::new("Follow Dot"),
        FollowDot,
        Sprite::from_color(Color::srgb(0.96, 0.48, 0.22), Vec2::splat(24.0)),
        Transform::from_xyz(0.0, -250.0, 10.0),
    ));
}
