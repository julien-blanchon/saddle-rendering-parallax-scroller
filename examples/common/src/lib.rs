use bevy::{
    app::AppExit,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
};
use saddle_pane::prelude::*;

use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxDiagnostics, ParallaxLayer, ParallaxLayerBundle,
    ParallaxLayerStrategy, ParallaxRig, ParallaxRigBundle, ParallaxScrollerPlugin,
    ParallaxSegmented, ParallaxSnap,
};

pub const WINDOW_SIZE: (u32, u32) = (1280, 720);

#[derive(Component)]
pub struct DemoCamera {
    pub horizontal_speed: f32,
    pub vertical_amplitude: f32,
    pub zoom_amplitude: f32,
}

#[derive(Component)]
pub struct FollowDot;

#[derive(Component)]
pub struct PixelDrift;

pub struct DemoTextures {
    pub sky: Handle<Image>,
    pub mountains: Handle<Image>,
    pub canopy: Handle<Image>,
    pub stars: Handle<Image>,
    pub pixel_clouds: Handle<Image>,
    pub vista: Handle<Image>,
}

#[derive(Resource, Debug, Clone, PartialEq, Pane)]
#[pane(title = "Parallax", position = "top-right")]
pub struct ExampleParallaxPane {
    #[pane(slider, min = 0.0, max = 220.0, step = 1.0)]
    pub camera_speed: f32,
    #[pane(slider, min = 0.0, max = 60.0, step = 0.5)]
    pub vertical_amplitude: f32,
    #[pane(slider, min = 0.0, max = 0.5, step = 0.01)]
    pub zoom_amplitude: f32,
    #[pane(slider, min = 0.0, max = 1.4, step = 0.01)]
    pub mountain_factor: f32,
    #[pane(slider, min = 0.0, max = 1.4, step = 0.01)]
    pub canopy_factor: f32,
    #[pane(slider, min = -120.0, max = 120.0, step = 1.0)]
    pub starfield_scroll_y: f32,
    #[pane(slider, min = 0.0, max = 1.5, step = 0.05)]
    pub depth_translation_response: f32,
    #[pane(slider, min = 0.0, max = 2.0, step = 0.05)]
    pub depth_scale_response: f32,
    #[pane(monitor)]
    pub rig_count: f32,
    #[pane(monitor)]
    pub layer_count: f32,
    #[pane(monitor)]
    pub first_depth_ratio: f32,
}

impl Default for ExampleParallaxPane {
    fn default() -> Self {
        Self {
            camera_speed: 120.0,
            vertical_amplitude: 24.0,
            zoom_amplitude: 0.22,
            mountain_factor: 0.84,
            canopy_factor: 1.08,
            starfield_scroll_y: -48.0,
            depth_translation_response: 1.0,
            depth_scale_response: 1.0,
            rig_count: 0.0,
            layer_count: 0.0,
            first_depth_ratio: 0.0,
        }
    }
}

pub fn configure_app(app: &mut App) {
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "parallax_scroller examples".into(),
            resolution: WINDOW_SIZE.into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(ParallaxScrollerPlugin::default());
    install_auto_exit(app, "PARALLAX_SCROLLER_EXIT_AFTER_SECONDS");
}

pub fn install_pane(app: &mut App) {
    if !app.is_plugin_added::<PanePlugin>() {
        app.add_plugins((
            bevy_flair::FlairPlugin,
            bevy_input_focus::InputDispatchPlugin,
            bevy_ui_widgets::UiWidgetsPlugins,
            bevy_input_focus::tab_navigation::TabNavigationPlugin,
            PanePlugin,
        ));
    }

    app.register_pane::<ExampleParallaxPane>()
        .add_systems(Update, (sync_example_pane, update_example_pane_monitors));
}

#[derive(Resource)]
struct AutoExitAfter(Timer);

pub fn install_auto_exit(app: &mut App, env_var: &str) {
    let Some(seconds) = std::env::var(env_var)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|seconds| *seconds > 0.0)
    else {
        return;
    };

    app.insert_resource(AutoExitAfter(Timer::from_seconds(seconds, TimerMode::Once)));
    app.add_systems(Update, auto_exit_after);
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        app_exit.write(AppExit::Success);
    }
}

fn sync_example_pane(
    pane: Res<ExampleParallaxPane>,
    mut demo_cameras: Query<&mut DemoCamera>,
    mut layers: Query<(&Name, &mut ParallaxLayer)>,
) {
    for mut camera in &mut demo_cameras {
        camera.horizontal_speed = pane.camera_speed.max(0.0);
        camera.vertical_amplitude = pane.vertical_amplitude.max(0.0);
        camera.zoom_amplitude = pane.zoom_amplitude.max(0.0);
    }

    for (name, mut layer) in &mut layers {
        let label = name.as_str();
        if label.contains("Mountain") || label.contains("Cliffs") {
            layer.camera_factor.x = pane.mountain_factor.max(0.0);
        }
        if label.contains("Canopy") || label.contains("Boughs") {
            layer.camera_factor.x = pane.canopy_factor.max(0.0);
        }
        if label.contains("Starfield") {
            layer.auto_scroll.y = pane.starfield_scroll_y;
        }
        if let Some(depth_mapping) = layer.depth_mapping.as_mut() {
            depth_mapping.translation_response.x = pane.depth_translation_response.max(0.0);
            depth_mapping.scale_response = pane.depth_scale_response.max(0.0);
        }
    }
}

fn update_example_pane_monitors(
    diagnostics: Option<Res<ParallaxDiagnostics>>,
    mut pane: ResMut<ExampleParallaxPane>,
) {
    let Some(diagnostics) = diagnostics else {
        return;
    };

    pane.rig_count = diagnostics.rigs.len() as f32;
    pane.layer_count = diagnostics.rigs.iter().map(|rig| rig.layers.len()).sum::<usize>() as f32;
    pane.first_depth_ratio = diagnostics
        .rigs
        .iter()
        .flat_map(|rig| rig.layers.iter())
        .find_map(|layer| layer.depth_ratio)
        .unwrap_or(0.0);
}

pub fn demo_textures(images: &mut Assets<Image>) -> DemoTextures {
    DemoTextures {
        sky: images.add(pattern_image(
            UVec2::new(256, 256),
            Color::srgb(0.72, 0.88, 0.98),
            Color::srgb(0.96, 0.98, 1.0),
            6.0,
        )),
        mountains: images.add(mountain_strip(UVec2::new(320, 96))),
        canopy: images.add(stripe_strip(
            UVec2::new(256, 64),
            Color::srgba(0.10, 0.24, 0.12, 1.0),
            Color::srgba(0.18, 0.34, 0.16, 1.0),
            10,
        )),
        stars: images.add(starfield(UVec2::new(256, 256))),
        pixel_clouds: images.add(pixel_clouds(UVec2::new(96, 64))),
        vista: images.add(vista_image(UVec2::new(640, 220))),
    }
}

pub fn spawn_camera(commands: &mut Commands, translation: Vec3) -> Entity {
    commands
        .spawn((
            Name::new("Demo Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            Transform::from_translation(translation),
        ))
        .id()
}

pub fn spawn_follow_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Name::new("Follow Camera"),
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                ..OrthographicProjection::default_2d()
            }),
            DemoCamera {
                horizontal_speed: 120.0,
                vertical_amplitude: 24.0,
                zoom_amplitude: 0.22,
            },
            Transform::default(),
        ))
        .id()
}

pub fn spawn_demo_rig(commands: &mut Commands, camera: Entity, name: &str, origin: Vec3) -> Entity {
    commands
        .spawn((
            Name::new(name.to_string()),
            ParallaxRigBundle {
                rig: ParallaxRig::default(),
                transform: Transform::from_translation(origin),
                ..default()
            },
            ParallaxCameraTarget::new(camera),
        ))
        .id()
}

pub fn spawn_tiled_layer(
    commands: &mut Commands,
    rig: Entity,
    name: &str,
    image: Handle<Image>,
    layer: ParallaxLayer,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_string()),
            ChildOf(rig),
            ParallaxLayerBundle {
                layer,
                sprite: Sprite::from_image(image),
                ..default()
            },
        ))
        .id()
}

pub fn update_demo_camera(
    time: Res<Time>,
    mut cameras: Query<(&DemoCamera, &mut Transform, &mut Projection)>,
) {
    let seconds = time.elapsed_secs();
    for (controller, mut transform, mut projection) in &mut cameras {
        transform.translation.x = seconds * controller.horizontal_speed;
        transform.translation.y = controller.vertical_amplitude * (seconds * 0.55).sin();
        if let Projection::Orthographic(projection) = projection.as_mut() {
            projection.scale = 1.0 + controller.zoom_amplitude * (seconds * 0.35).sin().abs();
        }
    }
}

pub fn animate_follow_dot(time: Res<Time>, mut query: Query<&mut Transform, With<FollowDot>>) {
    for mut transform in &mut query {
        transform.translation.x = time.elapsed_secs() * 120.0;
        transform.translation.y = (time.elapsed_secs() * 1.6).sin() * 40.0;
    }
}

pub fn drift_pixel_camera(time: Res<Time>, mut query: Query<&mut Transform, With<PixelDrift>>) {
    for mut transform in &mut query {
        transform.translation.x = time.elapsed_secs() * 17.25;
        transform.translation.y = (time.elapsed_secs() * 0.7).sin() * 2.25;
    }
}

pub fn add_forest_stack(commands: &mut Commands, rig: Entity, textures: &DemoTextures) {
    spawn_tiled_layer(
        commands,
        rig,
        "Sky Layer",
        textures.sky.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::ONE)
            .with_repeat(ParallaxAxes::both())
            .with_coverage_margin(Vec2::new(96.0, 48.0))
            .with_tint(Color::srgba(0.95, 0.98, 1.0, 0.92))
            .with_scale(Vec2::splat(2.0))
            .with_origin(Vec2::new(0.0, 24.0)),
    );

    spawn_tiled_layer(
        commands,
        rig,
        "Mountain Layer",
        textures.mountains.clone(),
        ParallaxLayer {
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented::default()),
            camera_factor: Vec2::new(0.84, 1.0),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -96.0),
            depth: 1.0,
            scale: Vec2::splat(1.4),
            tint: Color::srgb(0.34, 0.47, 0.56),
            source_size: Some(Vec2::new(320.0, 96.0)),
            ..default()
        },
    );

    spawn_tiled_layer(
        commands,
        rig,
        "Canopy Layer",
        textures.canopy.clone(),
        ParallaxLayer {
            strategy: ParallaxLayerStrategy::Segmented(ParallaxSegmented {
                extra_rings: UVec2::new(2, 0),
            }),
            camera_factor: Vec2::new(1.08, 1.0),
            repeat: ParallaxAxes::horizontal(),
            origin: Vec2::new(0.0, -200.0),
            depth: 2.0,
            scale: Vec2::new(1.5, 2.0),
            tint: Color::srgb(0.14, 0.28, 0.14),
            source_size: Some(Vec2::new(256.0, 64.0)),
            ..default()
        },
    );
}

pub fn add_starfield_stack(commands: &mut Commands, rig: Entity, textures: &DemoTextures) {
    spawn_tiled_layer(
        commands,
        rig,
        "Starfield",
        textures.stars.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::ONE)
            .with_auto_scroll(Vec2::new(-12.0, -48.0))
            .with_repeat(ParallaxAxes::both())
            .with_scale(Vec2::splat(2.0))
            .with_coverage_margin(Vec2::new(80.0, 80.0))
            .with_tint(Color::srgba(1.0, 1.0, 1.0, 0.95)),
    );

    spawn_tiled_layer(
        commands,
        rig,
        "Cloud Bands",
        textures.pixel_clouds.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::ONE)
            .with_auto_scroll(Vec2::new(18.0, -12.0))
            .with_repeat(ParallaxAxes::both())
            .with_scale(Vec2::splat(3.0))
            .with_tint(Color::srgba(0.68, 0.86, 1.0, 0.22))
            .with_phase(Vec2::new(60.0, 40.0)),
    );
}

pub fn add_pixel_snap_pair(commands: &mut Commands, rig: Entity, textures: &DemoTextures) {
    spawn_tiled_layer(
        commands,
        rig,
        "Unsnapped Clouds",
        textures.pixel_clouds.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::new(0.92, 1.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, 72.0))
            .with_scale(Vec2::splat(4.0))
            .with_tint(Color::srgba(1.0, 1.0, 1.0, 0.92)),
    );

    spawn_tiled_layer(
        commands,
        rig,
        "Snapped Clouds",
        textures.pixel_clouds.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::new(0.92, 1.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -56.0))
            .with_scale(Vec2::splat(4.0))
            .with_snap(ParallaxSnap::Pixel)
            .with_tint(Color::srgba(1.0, 1.0, 1.0, 0.92)),
    );
}

pub fn add_finite_vista(commands: &mut Commands, rig: Entity, textures: &DemoTextures) {
    spawn_tiled_layer(
        commands,
        rig,
        "Finite Vista",
        textures.vista.clone(),
        ParallaxLayer {
            repeat: ParallaxAxes::none(),
            bounds: saddle_rendering_parallax_scroller::ParallaxBounds::horizontal(-160.0, 160.0),
            camera_factor: Vec2::new(0.88, 1.0),
            origin: Vec2::new(0.0, -40.0),
            scale: Vec2::ONE,
            tint: Color::WHITE,
            source_size: Some(Vec2::new(640.0, 220.0)),
            ..default()
        },
    );
}

fn pattern_image(size: UVec2, a: Color, b: Color, band_height: f32) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    let a = a.to_srgba();
    let b = b.to_srgba();
    for y in 0..size.y {
        let t = ((y as f32 / band_height).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let r = a.red + (b.red - a.red) * t;
        let g = a.green + (b.green - a.green) * t;
        let bl = a.blue + (b.blue - a.blue) * t;
        for _ in 0..size.x {
            bytes.extend_from_slice(&[
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (bl * 255.0) as u8,
                255,
            ]);
        }
    }
    image(size, bytes, false)
}

fn mountain_strip(size: UVec2) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let normalized_x = x as f32 / size.x as f32;
            let ridge = 0.34 + (normalized_x * std::f32::consts::TAU * 2.0).sin() * 0.14;
            let ridge = ridge + (normalized_x * std::f32::consts::TAU * 6.0).sin() * 0.05;
            let y_normalized = y as f32 / size.y as f32;
            let alpha = if y_normalized > ridge { 255 } else { 0 };
            let shade = if y_normalized > ridge + 0.12 { 120 } else { 88 };
            bytes.extend_from_slice(&[shade, shade + 20, shade + 30, alpha]);
        }
    }
    image(size, bytes, false)
}

fn stripe_strip(size: UVec2, dark: Color, light: Color, stripe_width: u32) -> Image {
    let dark = dark.to_srgba();
    let light = light.to_srgba();
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let use_light = ((x / stripe_width) + (y / stripe_width.max(1))).is_multiple_of(2);
            let color = if use_light { light } else { dark };
            bytes.extend_from_slice(&[
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
                255,
            ]);
        }
    }
    image(size, bytes, false)
}

fn starfield(size: UVec2) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let seed = (x * 31 + y * 17 + x * y * 3) % 97;
            let bright = if seed == 0 || seed == 11 || seed == 37 {
                255
            } else if seed % 29 == 0 {
                180
            } else {
                8
            };
            bytes.extend_from_slice(&[bright, bright, bright, 255]);
        }
    }
    image(size, bytes, false)
}

fn pixel_clouds(size: UVec2) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let band = ((x / 8) + (y / 8)) % 3;
            let alpha = if band == 0 {
                40
            } else if band == 1 {
                120
            } else {
                200
            };
            bytes.extend_from_slice(&[255, 255, 255, alpha]);
        }
    }
    image(size, bytes, true)
}

fn vista_image(size: UVec2) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let horizon =
                0.44 + (x as f32 / size.x as f32 * std::f32::consts::TAU * 1.8).sin() * 0.08;
            let normalized_y = y as f32 / size.y as f32;
            let (r, g, b, a) = if normalized_y > horizon {
                (96, 132, 160, 255)
            } else if normalized_y > horizon - 0.06 {
                (180, 196, 206, 255)
            } else {
                (214, 224, 232, 255)
            };
            bytes.extend_from_slice(&[r, g, b, a]);
        }
    }
    image(size, bytes, false)
}

fn image(size: UVec2, bytes: Vec<u8>, nearest: bool) -> Image {
    let mut image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        bytes,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    image.sampler = if nearest {
        ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: bevy::image::ImageFilterMode::Nearest,
            min_filter: bevy::image::ImageFilterMode::Nearest,
            mipmap_filter: bevy::image::ImageFilterMode::Nearest,
            ..default()
        })
    } else {
        ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..default()
        })
    };
    image
}
