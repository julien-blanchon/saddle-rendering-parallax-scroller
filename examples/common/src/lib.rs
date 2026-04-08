use bevy::{
    app::AppExit,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
};
use saddle_pane::prelude::*;

use saddle_rendering_parallax_scroller::{
    ParallaxAxes, ParallaxCameraTarget, ParallaxDiagnostics, ParallaxLayer, ParallaxLayerStrategy,
    ParallaxRig, ParallaxSegmented, ParallaxSnap,
};

pub const WINDOW_SIZE: (u32, u32) = (1280, 720);

/// Marker for the animated demo camera that drifts horizontally and oscillates vertically.
#[derive(Component)]
pub struct DemoCamera {
    pub horizontal_speed: f32,
    pub vertical_amplitude: f32,
    pub zoom_amplitude: f32,
}

/// Pre-generated procedural textures used across examples.
pub struct DemoTextures {
    pub sky: Handle<Image>,
    pub mountains: Handle<Image>,
    pub canopy: Handle<Image>,
    pub stars: Handle<Image>,
    pub pixel_clouds: Handle<Image>,
    pub vista: Handle<Image>,
    // Rich forest layers (back-to-front, each with atmospheric perspective)
    pub forest_sky: Handle<Image>,
    pub forest_far_mountains: Handle<Image>,
    pub forest_near_mountains: Handle<Image>,
    pub forest_far_trees: Handle<Image>,
    pub forest_mid_trees: Handle<Image>,
    pub forest_near_trees: Handle<Image>,
    pub forest_ground: Handle<Image>,
    // City layers
    pub city_sky: Handle<Image>,
    pub city_far_buildings: Handle<Image>,
    pub city_mid_buildings: Handle<Image>,
    pub city_near_buildings: Handle<Image>,
    pub city_ground: Handle<Image>,
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
    #[pane(monitor)]
    pub rig_count: f32,
    #[pane(monitor)]
    pub layer_count: f32,
}

impl Default for ExampleParallaxPane {
    fn default() -> Self {
        Self {
            camera_speed: 120.0,
            vertical_amplitude: 24.0,
            zoom_amplitude: 0.22,
            rig_count: 0.0,
            layer_count: 0.0,
        }
    }
}

/// Adds DefaultPlugins (with window), the parallax scroller plugin, and auto-exit.
pub fn configure_app(app: &mut App) {
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "parallax_scroller examples".into(),
            resolution: WINDOW_SIZE.into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(saddle_rendering_parallax_scroller::ParallaxScrollerPlugin::default());
    install_auto_exit(app, "PARALLAX_SCROLLER_EXIT_AFTER_SECONDS");
}

/// Installs the saddle-pane debug UI with the parallax pane.
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
        .add_systems(Update, (sync_pane_to_camera, update_pane_monitors));
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

fn sync_pane_to_camera(pane: Res<ExampleParallaxPane>, mut cameras: Query<&mut DemoCamera>) {
    for mut camera in &mut cameras {
        camera.horizontal_speed = pane.camera_speed.max(0.0);
        camera.vertical_amplitude = pane.vertical_amplitude.max(0.0);
        camera.zoom_amplitude = pane.zoom_amplitude.max(0.0);
    }
}

fn update_pane_monitors(
    diagnostics: Option<Res<ParallaxDiagnostics>>,
    mut pane: ResMut<ExampleParallaxPane>,
) {
    let Some(diagnostics) = diagnostics else {
        return;
    };

    pane.rig_count = diagnostics.rigs.len() as f32;
    pane.layer_count = diagnostics
        .rigs
        .iter()
        .map(|rig| rig.layers.len())
        .sum::<usize>() as f32;
}

// ---------------------------------------------------------------------------
// Procedural texture generation
// ---------------------------------------------------------------------------

pub fn demo_textures(images: &mut Assets<Image>) -> DemoTextures {
    DemoTextures {
        // Legacy textures (kept for backward compat with existing examples)
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
        // Rich forest scene layers
        forest_sky: images.add(forest_sky_gradient(UVec2::new(256, 384))),
        forest_far_mountains: images.add(forest_mountain_silhouette(
            UVec2::new(512, 192),
            0.85,
            &[1.2, 3.5, 7.0],
            &[0.22, 0.10, 0.04],
            [0.55, 0.62, 0.75],
        )),
        forest_near_mountains: images.add(forest_mountain_silhouette(
            UVec2::new(512, 192),
            0.65,
            &[0.8, 2.2, 5.5, 9.0],
            &[0.18, 0.12, 0.06, 0.03],
            [0.30, 0.38, 0.48],
        )),
        forest_far_trees: images.add(tree_silhouette_strip(
            UVec2::new(512, 224),
            0.55,
            14,
            [0.18, 0.30, 0.22],
            0.65,
        )),
        forest_mid_trees: images.add(tree_silhouette_strip(
            UVec2::new(512, 256),
            0.42,
            10,
            [0.10, 0.22, 0.14],
            0.80,
        )),
        forest_near_trees: images.add(tree_silhouette_strip(
            UVec2::new(512, 288),
            0.28,
            8,
            [0.05, 0.14, 0.08],
            1.0,
        )),
        forest_ground: images.add(ground_strip(
            UVec2::new(256, 64),
            [0.08, 0.16, 0.06],
            [0.12, 0.22, 0.10],
        )),
        // City scene layers
        city_sky: images.add(city_sky_gradient(UVec2::new(256, 384))),
        city_far_buildings: images.add(building_silhouette_strip(
            UVec2::new(512, 256),
            0.55,
            &[28, 42, 36, 50, 32, 44, 38, 46, 34, 40],
            [0.30, 0.32, 0.42],
            0.0,
        )),
        city_mid_buildings: images.add(building_silhouette_strip(
            UVec2::new(512, 256),
            0.40,
            &[55, 70, 45, 85, 60, 75, 50, 80, 65, 55, 90, 48],
            [0.18, 0.20, 0.28],
            0.15,
        )),
        city_near_buildings: images.add(building_silhouette_strip(
            UVec2::new(512, 320),
            0.25,
            &[70, 95, 55, 110, 80, 65, 100, 75, 90, 60, 85],
            [0.10, 0.12, 0.18],
            0.35,
        )),
        city_ground: images.add(ground_strip(
            UVec2::new(256, 48),
            [0.14, 0.14, 0.16],
            [0.18, 0.18, 0.22],
        )),
    }
}

/// System that drifts the demo camera horizontally and oscillates vertically + zoom.
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

// ===========================================================================
// Rich procedural texture generators
// ===========================================================================

/// Deterministic hash for procedural noise (no rand dependency).
fn hash_u32(mut x: u32) -> u32 {
    x = x.wrapping_mul(0x9E3779B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EBCA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2AE35);
    x ^= x >> 16;
    x
}

/// Value noise in [0, 1] from integer coords.
fn noise2d(x: i32, y: i32) -> f32 {
    let h = hash_u32(
        (x as u32)
            .wrapping_mul(1597)
            .wrapping_add((y as u32).wrapping_mul(51749)),
    );
    (h & 0xFFFF) as f32 / 65535.0
}

/// Smooth value noise with bilinear interpolation.
fn smooth_noise(x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();
    // Smoothstep
    let fx = fx * fx * (3.0 - 2.0 * fx);
    let fy = fy * fy * (3.0 - 2.0 * fy);

    let n00 = noise2d(ix, iy);
    let n10 = noise2d(ix + 1, iy);
    let n01 = noise2d(ix, iy + 1);
    let n11 = noise2d(ix + 1, iy + 1);

    let nx0 = n00 + (n10 - n00) * fx;
    let nx1 = n01 + (n11 - n01) * fx;
    nx0 + (nx1 - nx0) * fy
}

/// Fractal Brownian motion (layered noise) for organic shapes.
fn fbm(x: f32, y: f32, octaves: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    for _ in 0..octaves {
        value += smooth_noise(x * frequency, y * frequency) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    value
}

// --- Forest scene textures ---

fn forest_sky_gradient(size: UVec2) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        let t = y as f32 / size.y as f32;
        // Top: deep blue, bottom: pale orange/pink horizon
        let r = lerp(0.15, 0.95, t);
        let g = lerp(0.22, 0.78, t);
        let b = lerp(0.55, 0.72, t);
        for x in 0..size.x {
            // Subtle cloud wisps
            let cloud = fbm(x as f32 * 0.008, y as f32 * 0.015, 3);
            let cloud_amount = ((cloud - 0.35) * 3.0).clamp(0.0, 0.15);
            bytes.extend_from_slice(&[
                to_u8(r + cloud_amount),
                to_u8(g + cloud_amount),
                to_u8(b + cloud_amount * 0.5),
                255,
            ]);
        }
    }
    make_image(size, bytes, false)
}

fn forest_mountain_silhouette(
    size: UVec2,
    horizon: f32,
    frequencies: &[f32],
    amplitudes: &[f32],
    color: [f32; 3],
) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let nx = x as f32 / size.x as f32;
            let ny = y as f32 / size.y as f32;

            // Build mountain ridge from sum of sine waves + noise
            let mut ridge = horizon;
            for (freq, amp) in frequencies.iter().zip(amplitudes.iter()) {
                ridge += (nx * std::f32::consts::TAU * freq).sin() * amp;
            }
            // Add noise for natural irregularity
            ridge += fbm(nx * 8.0, 0.5, 3) * 0.06 - 0.03;

            if ny > ridge {
                // Below ridge — solid mountain with subtle vertical gradient
                let shade = 1.0 - (ny - ridge) * 0.3;
                bytes.extend_from_slice(&[
                    to_u8(color[0] * shade),
                    to_u8(color[1] * shade),
                    to_u8(color[2] * shade),
                    255,
                ]);
            } else {
                bytes.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    make_image(size, bytes, false)
}

fn tree_silhouette_strip(
    size: UVec2,
    tree_line: f32,
    tree_count: u32,
    color: [f32; 3],
    opacity: f32,
) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    let tree_spacing = size.x as f32 / tree_count as f32;

    for y in 0..size.y {
        for x in 0..size.x {
            let ny = y as f32 / size.y as f32;
            let nx = x as f32;

            // Base ground line
            let ground = tree_line + fbm(nx * 0.006, 3.0, 2) * 0.05;

            if ny > ground {
                // Ground area
                let shade = 1.0 - (ny - ground) * 0.4;
                bytes.extend_from_slice(&[
                    to_u8(color[0] * shade),
                    to_u8(color[1] * shade),
                    to_u8(color[2] * shade),
                    to_u8(opacity),
                ]);
            } else {
                // Check if pixel is inside any tree
                let mut in_tree = false;
                for i in 0..tree_count {
                    let seed = hash_u32(i * 7919 + 1013);
                    let jitter = (seed & 0xFF) as f32 / 255.0 - 0.5;
                    let tree_center_x = (i as f32 + 0.5 + jitter * 0.6) * tree_spacing;

                    // Tree height varies
                    let height_seed = hash_u32(i * 3571 + 2903);
                    let height_factor = 0.6 + (height_seed & 0xFF) as f32 / 255.0 * 0.4;
                    let tree_top = ground - height_factor * 0.35;

                    // Conical pine tree shape
                    let dist_from_center = (nx - tree_center_x).abs();
                    let tree_progress = (ny - tree_top) / (ground - tree_top);
                    if tree_progress > 0.0 && tree_progress < 1.0 {
                        // Width increases from top to bottom
                        let max_width = tree_spacing * 0.35 * tree_progress;
                        // Jagged edges for pine needles
                        let edge_noise =
                            fbm(nx * 0.05 + i as f32 * 10.0, ny * 0.08, 2) * tree_spacing * 0.08;
                        if dist_from_center < max_width + edge_noise {
                            in_tree = true;
                            break;
                        }
                    }

                    // Trunk
                    let trunk_width = tree_spacing * 0.04;
                    if dist_from_center < trunk_width && ny > ground - 0.05 && ny <= ground {
                        in_tree = true;
                        break;
                    }
                }

                if in_tree {
                    // Slight shade variation within tree
                    let variation = fbm(nx * 0.02, ny * 0.02, 2) * 0.1;
                    bytes.extend_from_slice(&[
                        to_u8(color[0] + variation),
                        to_u8(color[1] + variation * 0.5),
                        to_u8(color[2] + variation),
                        to_u8(opacity),
                    ]);
                } else {
                    bytes.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
    }
    make_image(size, bytes, false)
}

fn ground_strip(size: UVec2, dark: [f32; 3], light: [f32; 3]) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        for x in 0..size.x {
            let ny = y as f32 / size.y as f32;
            let noise = fbm(x as f32 * 0.03, y as f32 * 0.05, 3);
            let t = (noise * 0.8 + ny * 0.2).clamp(0.0, 1.0);
            bytes.extend_from_slice(&[
                to_u8(lerp(dark[0], light[0], t)),
                to_u8(lerp(dark[1], light[1], t)),
                to_u8(lerp(dark[2], light[2], t)),
                255,
            ]);
        }
    }
    make_image(size, bytes, false)
}

// --- City scene textures ---

fn city_sky_gradient(size: UVec2) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    for y in 0..size.y {
        let t = y as f32 / size.y as f32;
        // Dusk sky: deep purple-blue at top, warm orange at horizon
        let r = lerp(0.08, 0.85, t * t);
        let g = lerp(0.06, 0.45, t);
        let b = lerp(0.22, 0.35, t);
        for x in 0..size.x {
            let cloud = fbm(x as f32 * 0.006, y as f32 * 0.012, 3);
            let cloud_amount = ((cloud - 0.4) * 2.5).clamp(0.0, 0.12);
            bytes.extend_from_slice(&[
                to_u8(r + cloud_amount * 0.8),
                to_u8(g + cloud_amount * 0.4),
                to_u8(b + cloud_amount * 0.3),
                255,
            ]);
        }
    }
    make_image(size, bytes, false)
}

fn building_silhouette_strip(
    size: UVec2,
    baseline: f32,
    heights_pct: &[u32],
    color: [f32; 3],
    window_brightness: f32,
) -> Image {
    let mut bytes = Vec::with_capacity((size.x * size.y * 4) as usize);
    let building_count = heights_pct.len();
    let building_width = size.x as f32 / building_count as f32;

    for y in 0..size.y {
        for x in 0..size.x {
            let ny = y as f32 / size.y as f32;
            let building_index = ((x as f32 / building_width) as usize).min(building_count - 1);
            let building_height = heights_pct[building_index] as f32 / 100.0;
            let top = baseline - building_height * (1.0 - baseline);

            if ny > top {
                // Inside building
                let local_x = x as f32 - building_index as f32 * building_width;
                let gap = building_width * 0.06;

                if local_x < gap || local_x > building_width - gap {
                    // Gap between buildings — transparent
                    bytes.extend_from_slice(&[0, 0, 0, 0]);
                } else if window_brightness > 0.0 {
                    // Draw windows
                    let win_w = 4.0_f32;
                    let win_h = 5.0_f32;
                    let win_gap_x = 8.0_f32;
                    let win_gap_y = 10.0_f32;
                    let inner_x = local_x - gap;
                    let inner_y = (ny - top) * size.y as f32;

                    let wx = inner_x % win_gap_x;
                    let wy = inner_y % win_gap_y;
                    let is_window = wx > 1.0 && wx < win_w + 1.0 && wy > 2.0 && wy < win_h + 2.0;

                    if is_window {
                        // Randomly lit windows
                        let win_ix = (inner_x / win_gap_x) as u32;
                        let win_iy = (inner_y / win_gap_y) as u32;
                        let lit = hash_u32(
                            win_ix
                                .wrapping_mul(1301)
                                .wrapping_add(win_iy.wrapping_mul(7919))
                                .wrapping_add(building_index as u32 * 4591),
                        ) & 3;
                        if lit == 0 {
                            // Lit window — warm yellow
                            let b_var = window_brightness * (0.7 + (lit as f32) * 0.1);
                            bytes.extend_from_slice(&[
                                to_u8(0.95 * b_var),
                                to_u8(0.85 * b_var),
                                to_u8(0.45 * b_var),
                                255,
                            ]);
                        } else {
                            // Dark window
                            bytes.extend_from_slice(&[
                                to_u8(color[0] * 0.7),
                                to_u8(color[1] * 0.7),
                                to_u8(color[2] * 0.7),
                                255,
                            ]);
                        }
                    } else {
                        bytes.extend_from_slice(&[
                            to_u8(color[0]),
                            to_u8(color[1]),
                            to_u8(color[2]),
                            255,
                        ]);
                    }
                } else {
                    // No windows (far buildings are just silhouettes)
                    bytes.extend_from_slice(&[
                        to_u8(color[0]),
                        to_u8(color[1]),
                        to_u8(color[2]),
                        255,
                    ]);
                }
            } else {
                bytes.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    make_image(size, bytes, false)
}

// ---------------------------------------------------------------------------
// Legacy procedural texture helpers (kept for backward compat)
// ---------------------------------------------------------------------------

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
    make_image(size, bytes, false)
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
    make_image(size, bytes, false)
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
    make_image(size, bytes, false)
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
    make_image(size, bytes, false)
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
    make_image(size, bytes, true)
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
    make_image(size, bytes, false)
}

// ---------------------------------------------------------------------------
// Shared entity spawners
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct FollowDot;

#[derive(Component)]
pub struct PixelDrift;

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
            ParallaxRig::default(),
            Transform::from_translation(origin),
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
            layer,
            Sprite::from_image(image),
        ))
        .id()
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

/// Spawns a rich 7-layer forest parallax scene with atmospheric perspective.
pub fn add_rich_forest_stack(commands: &mut Commands, rig: Entity, textures: &DemoTextures) {
    // Layer 0: Sky gradient — moves 1:1 with camera (background fill)
    spawn_tiled_layer(
        commands,
        rig,
        "Forest Sky",
        textures.forest_sky.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::splat(0.1))
            .with_repeat(ParallaxAxes::both())
            .with_coverage_margin(Vec2::new(200.0, 100.0))
            .with_scale(Vec2::splat(3.0)),
    );

    // Layer 1: Far mountains — very slow scroll, faded blue
    spawn_tiled_layer(
        commands,
        rig,
        "Far Mountains",
        textures.forest_far_mountains.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.15, 0.3))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -60.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 192.0)),
    );

    // Layer 2: Near mountains — slightly faster scroll, darker
    spawn_tiled_layer(
        commands,
        rig,
        "Near Mountains",
        textures.forest_near_mountains.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.30, 0.4))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -90.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 192.0)),
    );

    // Layer 3: Far trees — distant treeline
    spawn_tiled_layer(
        commands,
        rig,
        "Far Trees",
        textures.forest_far_trees.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.50, 0.6))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -100.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 224.0)),
    );

    // Layer 4: Mid trees
    spawn_tiled_layer(
        commands,
        rig,
        "Mid Trees",
        textures.forest_mid_trees.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.70, 0.8))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -120.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 256.0)),
    );

    // Layer 5: Near trees — closest trees, darkest, fastest scroll
    spawn_tiled_layer(
        commands,
        rig,
        "Near Trees",
        textures.forest_near_trees.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.90, 0.9))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -140.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 288.0)),
    );

    // Layer 6: Ground
    spawn_tiled_layer(
        commands,
        rig,
        "Forest Ground",
        textures.forest_ground.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::new(1.0, 1.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -330.0))
            .with_scale(Vec2::new(2.0, 2.0))
            .with_coverage_margin(Vec2::new(200.0, 0.0)),
    );
}

/// Spawns a 5-layer city skyline parallax scene at dusk.
pub fn add_city_stack(commands: &mut Commands, rig: Entity, textures: &DemoTextures) {
    // Layer 0: Dusk sky gradient
    spawn_tiled_layer(
        commands,
        rig,
        "City Sky",
        textures.city_sky.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::splat(0.05))
            .with_repeat(ParallaxAxes::both())
            .with_coverage_margin(Vec2::new(200.0, 100.0))
            .with_scale(Vec2::splat(3.0)),
    );

    // Layer 1: Far buildings — barely moving silhouettes
    spawn_tiled_layer(
        commands,
        rig,
        "Far Buildings",
        textures.city_far_buildings.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.20, 0.3))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -80.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 256.0)),
    );

    // Layer 2: Mid buildings — some windows visible
    spawn_tiled_layer(
        commands,
        rig,
        "Mid Buildings",
        textures.city_mid_buildings.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.50, 0.6))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -100.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 256.0)),
    );

    // Layer 3: Near buildings — detailed with lit windows
    spawn_tiled_layer(
        commands,
        rig,
        "Near Buildings",
        textures.city_near_buildings.clone(),
        ParallaxLayer::segmented()
            .with_camera_factor(Vec2::new(0.80, 0.9))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -110.0))
            .with_scale(Vec2::splat(2.0))
            .with_source_size(Vec2::new(512.0, 320.0)),
    );

    // Layer 4: Ground / road
    spawn_tiled_layer(
        commands,
        rig,
        "City Ground",
        textures.city_ground.clone(),
        ParallaxLayer::tiled()
            .with_camera_factor(Vec2::new(1.0, 1.0))
            .with_repeat(ParallaxAxes::horizontal())
            .with_origin(Vec2::new(0.0, -340.0))
            .with_scale(Vec2::new(2.0, 2.0))
            .with_coverage_margin(Vec2::new(200.0, 0.0)),
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

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn to_u8(v: f32) -> u8 {
    (v * 255.0).clamp(0.0, 255.0) as u8
}

fn make_image(size: UVec2, bytes: Vec<u8>, nearest: bool) -> Image {
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
