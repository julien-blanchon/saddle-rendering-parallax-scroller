# Saddle Rendering Parallax Scroller

Reusable 2D parallax backgrounds for Bevy. The crate is built around independent rigs and layers, supports camera-relative and auto-scrolling motion, can render either seamless tiled sprites or repeated authored strips without per-frame respawn churn, and now supports perspective-aware 2.5D depth mapping for dolly/zoom scenes.

## Quick Start

```toml
saddle-rendering-parallax-scroller = { git = "https://github.com/julien-blanchon/saddle-rendering-parallax-scroller" }
```

```rust,no_run
use bevy::prelude::*;
use saddle_rendering_parallax_scroller::{
    ParallaxCameraTarget, ParallaxLayer, ParallaxLayerBundle, ParallaxRigBundle,
    ParallaxScrollerPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ParallaxScrollerPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = commands.spawn((Camera2d, Name::new("Camera"))).id();
    let rig = commands
        .spawn((
            Name::new("Parallax Rig"),
            ParallaxRigBundle::default(),
            ParallaxCameraTarget::new(camera),
        ))
        .id();

    commands.spawn((
        Name::new("Sky Layer"),
        ChildOf(rig),
        ParallaxLayerBundle {
            layer: ParallaxLayer::tiled(),
            sprite: Sprite::from_image(asset_server.load("sky.png")),
            ..default()
        },
    ));
}
```

`ParallaxLayer.camera_factor` uses screen-space-friendly semantics:

- `Vec2::ONE` keeps the layer locked to the camera
- `Vec2::ZERO` leaves the layer world-fixed
- values between `0` and `1` create background-style parallax
- values above `1` exaggerate foreground motion

## Public API

| Type | Purpose |
|------|---------|
| `ParallaxScrollerPlugin` | Registers runtime activation, camera tracking, offset accumulation, layout, diagnostics, and optional debug gizmo hooks |
| `ParallaxScrollerSystems` | Public ordering phases: `TrackCamera`, `UpdateOffsets`, `ApplyLayout`, `Diagnostics`, `Debug` |
| `ParallaxRig` | Marker/config for one independent parallax setup |
| `ParallaxCameraTarget` | Binds a rig to one camera entity |
| `ParallaxLayer` | Per-layer authoring config for camera factor, auto-scroll, repeat, bounds, snap mode, tint, phase, origin, scale, and strategy |
| `ParallaxRigBundle` | Ergonomic bundle for rig entities |
| `ParallaxLayerBundle` | Ergonomic bundle for layer entities with a `Sprite` |
| `ParallaxDepthMapping` | Optional perspective-aware depth response that converts layer Z into physical translation/scale parallax |
| `ParallaxLayerStrategy` | Chooses `TiledSprite` or `Segmented` rendering |
| `ParallaxTiledSprite` | Tiled-sprite strategy config: `stretch_value` and minimum coverage |
| `ParallaxSegmented` | Segment-wrap strategy config: extra offscreen rings |
| `ParallaxBounds` / `AxisRange` | Optional finite clamp ranges for non-repeating axes |
| `ParallaxSnap` | `None`, `Pixel`, or grid snapping |
| `ParallaxDiagnostics` | BRP/E2E-friendly runtime diagnostics resource |
| `ParallaxDebugSettings` | Optional gizmo controls for viewport, coverage, wrap span, and offset visualization |

## Layer Strategies

### `TiledSprite`

Best for seamless textures and soft patterns. The layer entity itself renders a `SpriteImageMode::Tiled` sprite sized to cover the active viewport.

Use it for:

- skies
- fog bands
- starfields
- repeating clouds
- underwater caustic overlays

### `Segmented`

Best for authored strips and silhouettes. The layer entity renders the center segment and the crate maintains a stable ring of cloned child sprites around it.

Use it for:

- mountain silhouettes
- foreground foliage bands
- repeated skyline panels
- title-screen vistas built from authored cards

## Configuration Notes

- `source_size` is the local size of one tile or one segment before `scale` is applied. If omitted, the crate derives it from `Sprite.rect` or the loaded image size.
- `coverage_margin` is extra world-space overscan added beyond the current viewport for repeating layers.
- `bounds` are applied only on non-repeating axes.
- `phase` is a manual offset added before wrapping or clamping.
- `auto_scroll` accumulates every frame and combines with camera-relative motion.
- `snap` is applied after wrapping/clamping so pixel-grid output stays stable near wrap boundaries.
- `depth_mapping` is optional. When present on a perspective-bound rig, the crate derives extra camera motion and scale from `layer.depth` relative to a configurable reference plane. This is the path to use for 2.5D card-stack scenes; start with `camera_factor = Vec2::ZERO` for physically mapped layers.

## Examples

Every shipped example includes `saddle-pane` controls for camera motion, parallax response, and runtime diagnostics. All textures are procedurally generated with atmospheric perspective — no external asset files needed.

| Example | Purpose | Run |
|---------|---------|-----|
| `basic` | Minimal side-scroller-style background stack | `cargo run -p saddle-rendering-parallax-scroller-example-basic` |
| `forest_scene` | Rich 7-layer forest with atmospheric depth (mountains, tree lines, ground) | `cargo run -p saddle-rendering-parallax-scroller-example-forest-scene` |
| `city_skyline` | Urban dusk panorama with building silhouettes and lit windows | `cargo run -p saddle-rendering-parallax-scroller-example-city-skyline` |
| `platformer_demo` | Keyboard-controlled character (WASD/Space) with camera-follow parallax | `cargo run -p saddle-rendering-parallax-scroller-example-platformer-demo` |
| `endless_runner` | Auto-scrolling city with speed control (Up/Down arrows) | `cargo run -p saddle-rendering-parallax-scroller-example-endless-runner` |
| `autoscroll_starfield` | Pure auto-scroll background motion | `cargo run -p saddle-rendering-parallax-scroller-example-autoscroll_starfield` |
| `camera_follow` | Camera motion plus layer response | `cargo run -p saddle-rendering-parallax-scroller-example-camera_follow` |
| `zoom_parallax` | Perspective-aware 2.5D dolly/zoom stack | `cargo run -p saddle-rendering-parallax-scroller-example-zoom_parallax` |
| `pixel_art_snap` | Pixel-stable snapping vs unsnapped drift | `cargo run -p saddle-rendering-parallax-scroller-example-pixel_art_snap` |
| `finite_bounds` | Non-infinite clamped vista | `cargo run -p saddle-rendering-parallax-scroller-example-finite_bounds` |
| `multi_rig` | Two independent rigs in one world | `cargo run -p saddle-rendering-parallax-scroller-example-multi_rig` |
| `stress_many_layers` | Large mixed stack for perf and no-churn inspection | `cargo run -p saddle-rendering-parallax-scroller-example-stress_many_layers` |

For batch verification, examples and the lab honor:

```bash
PARALLAX_SCROLLER_EXIT_AFTER_SECONDS=3 cargo run -p saddle-rendering-parallax-scroller-example-basic
```

## Crate-Local Lab

The richer verification app lives at `shared/rendering/saddle-rendering-parallax-scroller/examples/lab`:

```bash
cargo run -p saddle-rendering-parallax-scroller-lab
```

It includes:

- a camera-follow forest stack
- an auto-scrolling starfield rig
- a finite vista band
- a snapped vs unsnapped comparison strip
- runtime diagnostics text for BRP and E2E

Useful scenarios:

```bash
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_scroller_smoke
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_camera_motion
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_finite_bounds
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_zoom
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_pixel_snap
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_depth_mapping
```

## BRP

The lab starts a BRP listener on port `15742` in `dev` builds. Useful inspection targets:

- resource: `saddle_rendering_parallax_scroller::resources::ParallaxDiagnostics`
- component: `saddle_rendering_parallax_scroller::components::ParallaxLayer`
- component: `saddle_rendering_parallax_scroller::components::ParallaxRig`

Example commands:

```bash
cargo run -p saddle-rendering-parallax-scroller-lab
BRP_PORT=15742 uv run --project .codex/skills/bevy-brp/script brp resource get 'saddle_rendering_parallax_scroller::resources::ParallaxDiagnostics'
BRP_PORT=15742 uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
BRP_PORT=15742 uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle_rendering_parallax_scroller_lab.png
```

## Limitations / Tradeoffs

- The crate is designed for `Camera2d` and orthographic-style 2D composition first. It still works without a bound camera, but viewport-aware coverage is strongest when a camera is bound. `ParallaxDepthMapping` activates only when that bound camera uses a `Perspective` projection.
- `TiledSprite` inherits Bevy tiled-sprite semantics. In practice that means tile size is driven by the source image size, `stretch_value`, and `scale`.
- The segment strategy repeats one authored sprite/strip. It does not yet stitch heterogeneous authored segment sequences.
- The crate intentionally owns `Transform.translation` and `Transform.scale` on layer entities. Configure movement through `ParallaxLayer`, not by animating the layer transform directly.

More detail lives in [architecture.md](docs/architecture.md) and [configuration.md](docs/configuration.md).
