use saddle_rendering_parallax_scroller_example_common as common;

use bevy::prelude::*;

use common::{
    PixelDrift, add_pixel_snap_pair, configure_app, demo_textures, drift_pixel_camera,
    spawn_demo_rig,
};

fn main() {
    let mut app = App::new();
    configure_app(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, drift_pixel_camera);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let textures = demo_textures(&mut images);
    let camera = commands
        .spawn((
            Name::new("Pixel Drift Camera"),
            Camera2d,
            PixelDrift,
            Transform::default(),
        ))
        .id();
    let rig = spawn_demo_rig(&mut commands, camera, "Pixel Snap Rig", Vec3::ZERO);
    add_pixel_snap_pair(&mut commands, rig, &textures);
}
