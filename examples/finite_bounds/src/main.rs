use saddle_rendering_parallax_scroller_example_common as common;

use bevy::prelude::*;

use common::{
    add_finite_vista, configure_app, demo_textures, spawn_demo_rig, spawn_follow_camera,
    update_demo_camera,
};

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, update_demo_camera);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);
    let camera = spawn_follow_camera(&mut commands);
    let rig = spawn_demo_rig(&mut commands, camera, "Finite Bounds Rig", Vec3::ZERO);
    add_finite_vista(&mut commands, rig, &textures);
}
