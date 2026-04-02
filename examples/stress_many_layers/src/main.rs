use saddle_rendering_parallax_scroller_example_common as common;

use bevy::prelude::*;

use common::{
    configure_app, demo_textures, spawn_demo_rig, spawn_follow_camera, spawn_tiled_layer,
    update_demo_camera,
};
use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxLayer, ParallaxLayerStrategy, ParallaxSegmented,
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

    for rig_index in 0..3 {
        let origin = Vec3::new(0.0, (rig_index as f32 - 1.0) * 260.0, 0.0);
        let rig = spawn_demo_rig(
            &mut commands,
            camera,
            &format!("Stress Rig {}", rig_index + 1),
            origin,
        );

        for layer_index in 0..12 {
            let depth = layer_index as f32 * 0.05;
            let band = -220.0 + layer_index as f32 * 38.0;
            let factor = 0.62 + layer_index as f32 * 0.045;
            let auto_scroll = if layer_index % 3 == 0 {
                Vec2::new(
                    -10.0 + rig_index as f32 * 4.0,
                    (layer_index as f32 - 4.0) * 2.0,
                )
            } else {
                Vec2::ZERO
            };

            let (image, source_size, strategy, repeat, scale, tint) = match layer_index % 4 {
                0 => (
                    textures.sky.clone(),
                    None,
                    ParallaxLayerStrategy::default(),
                    ParallaxAxes::both(),
                    Vec2::splat(1.5 + rig_index as f32 * 0.2),
                    Color::srgba(1.0, 1.0, 1.0, 0.55),
                ),
                1 => (
                    textures.stars.clone(),
                    None,
                    ParallaxLayerStrategy::default(),
                    ParallaxAxes::both(),
                    Vec2::splat(1.8),
                    Color::srgba(0.92, 0.96, 1.0, 0.65),
                ),
                2 => (
                    textures.mountains.clone(),
                    Some(Vec2::new(320.0, 96.0)),
                    ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                        extra_rings: UVec2::new(2, 0),
                    }),
                    ParallaxAxes::horizontal(),
                    Vec2::new(1.1 + rig_index as f32 * 0.15, 1.35),
                    Color::srgba(0.35, 0.46, 0.58, 0.95),
                ),
                _ => (
                    textures.canopy.clone(),
                    Some(Vec2::new(256.0, 64.0)),
                    ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                        extra_rings: UVec2::new(3, 0),
                    }),
                    ParallaxAxes::horizontal(),
                    Vec2::new(1.3, 1.8),
                    Color::srgba(0.14, 0.28, 0.16, 0.92),
                ),
            };

            let mut layer = ParallaxLayer::default()
                .with_camera_factor(Vec2::new(factor, 1.0))
                .with_auto_scroll(auto_scroll)
                .with_repeat(repeat)
                .with_origin(Vec2::new(0.0, band))
                .with_scale(scale)
                .with_tint(tint)
                .with_coverage_margin(Vec2::new(96.0, 48.0));
            layer.depth = depth;
            layer.strategy = strategy;
            layer.source_size = source_size;

            spawn_tiled_layer(
                &mut commands,
                rig,
                &format!("Stress Layer {}-{}", rig_index + 1, layer_index + 1),
                image,
                layer,
            );
        }
    }
}
