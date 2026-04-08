# Configuration

## `ParallaxRig`

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `enabled` | `bool` | `true` | Freezes camera tracking and layer layout for the rig when `false` |
| `origin` | `Vec2` | `Vec2::ZERO` | Extra rig-local translation added to every child layer |
| `speed_multiplier` | `f32` | `1.0` | Multiplies auto-scroll dt for all child layers. `0.0` = frozen, `2.0` = double speed. Does not affect camera-factor |

## `ParallaxCameraTarget`

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `camera` | `Entity` | required | Binds the rig to one Bevy camera entity |

## `ParallaxLayer`

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `enabled` | `bool` | `true` | Skips auto-scroll accumulation when `false`; camera-relative layout still applies unless the parent rig is disabled |
| `camera_factor` | `Vec2` | `Vec2::ONE` | `1` follows the camera, `0` stays world-fixed, values in between create parallax |
| `auto_scroll` | `Vec2` | `Vec2::ZERO` | Adds world-units-per-second drift on top of camera-relative motion |
| `origin` | `Vec2` | `Vec2::ZERO` | Static local translation inside the rig |
| `phase` | `Vec2` | `Vec2::ZERO` | Manual phase shift applied before wrapping/clamping |
| `depth` | `f32` | `0.0` | Local Z value written to the layer transform |
| `depth_mapping` | `Option<ParallaxDepthMapping>` | `None` | Optional perspective-aware translation/scale response derived from `depth` |
| `repeat` | `ParallaxAxes` | `both()` | Enables wrap math per axis |
| `bounds` | `ParallaxBounds` | none | Clamp range for non-repeating axes |
| `snap` | `ParallaxSnap` | `None` | Rounds final offsets for pixel-stable output |
| `coverage_margin` | `Vec2` | `Vec2::ZERO` | Overscan added beyond the viewport on repeating axes |
| `source_size` | `Option<Vec2>` | `None` | One tile or one segment size before `scale`; derived from sprite image/rect when omitted |
| `scale` | `Vec2` | `Vec2::ONE` | Written to `Transform.scale.xy`; affects rendered size and wrap span |
| `tint` | `Color` | `Color::WHITE` | Written to `Sprite.color` |
| `strategy` | `ParallaxLayerStrategy` | `TiledSprite` | Chooses tiling vs segment cloning |
| `user_offset` | `Vec2` | `Vec2::ZERO` | Added on top of the computed parallax offset during `WriteTransforms` |
| `user_scale` | `Vec2` | `Vec2::ONE` | Multiplied on top of the computed parallax scale during `WriteTransforms` |
| `rotation` | `f32` | `0.0` | Rotation in radians around Z axis, written to `Transform.rotation` |

### Builder Methods

`ParallaxLayer` provides a fluent builder API for ergonomic construction:

```rust
ParallaxLayer::tiled()           // default tiled strategy
ParallaxLayer::segmented()       // default segmented strategy
    .with_camera_factor(Vec2)
    .with_auto_scroll(Vec2)
    .with_repeat(ParallaxAxes)
    .with_depth(f32)
    .with_depth_mapping(ParallaxDepthMapping)
    .with_bounds(ParallaxBounds)
    .with_snap(ParallaxSnap)
    .with_source_size(Vec2)
    .with_origin(Vec2)
    .with_phase(Vec2)
    .with_scale(Vec2)
    .with_tint(Color)
    .with_coverage_margin(Vec2)
    .with_enabled(bool)
    .with_strategy(ParallaxLayerStrategy)
    .with_user_offset(Vec2)
    .with_user_scale(Vec2)
    .with_rotation(f32)
```

## `ParallaxDepthMapping`

`ParallaxDepthMapping` only participates when the bound camera uses `Projection::Perspective`. It treats `ParallaxLayer.depth` as the physical layer plane and compares it against a reference plane.

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `reference_plane_z` | `f32` | `0.0` | World-space Z plane treated as the gameplay/reference depth |
| `translation_response` | `Vec2` | `Vec2::ONE` | Multiplies the physically derived translation factor per axis |
| `scale_response` | `f32` | `1.0` | Blends authored `scale` toward full perspective scale (`0 = no extra scale`, `1 = physical ratio`) |

Recommended starting point for 2.5D scenes:

- `camera_factor = Vec2::ZERO`
- `depth_mapping = Some(ParallaxDepthMapping::default())`
- negative `depth` for background cards
- positive `depth` for foreground cards

## `ParallaxAxes`

| Constructor | Meaning |
|-------------|---------|
| `none()` | no repeat |
| `horizontal()` | repeat only on X |
| `vertical()` | repeat only on Y |
| `both()` | repeat on both axes |

## `ParallaxBounds`

`ParallaxBounds` is only applied on axes where repeat is disabled.

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `x` | `Option<AxisRange>` | `None` | Horizontal clamp |
| `y` | `Option<AxisRange>` | `None` | Vertical clamp |

`AxisRange::new(min, max)` normalizes inverted input automatically.

## `ParallaxSnap`

| Variant | Effect |
|---------|--------|
| `None` | Keep smooth float offsets |
| `Pixel` | Round both axes to whole units |
| `Grid(Vec2)` | Round each axis to the provided step |

Recommended use:

- pixel art layers: `Pixel`
- chunky stylized 2D: `Grid(Vec2::splat(2.0))` or similar
- smooth painterly / HD layers: `None`

## `ParallaxLayerStrategy`

### `TiledSprite(ParallaxTiledSprite)`

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `stretch_value` | `f32` | `1.0` | Native Bevy tiled-sprite repeat threshold; also scales the effective wrap span |
| `minimum_coverage` | `Vec2` | `Vec2::ZERO` | Forces a floor under the auto-sized tiled sprite coverage |

Interaction notes:

- wrap span = `source_size * abs(scale) * stretch_value`
- coverage grows with viewport size on repeating axes
- non-repeating axes fall back to `source_size` unless `minimum_coverage` is larger

### `Segmented(ParallaxSegmented)`

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `extra_rings` | `UVec2` | `UVec2::ONE` | Extra offscreen rings of cloned segments per axis |

Interaction notes:

- one segment's world size is `source_size * abs(scale)`
- the total segment count is always odd, with the layer entity as the center segment
- increasing `extra_rings` reduces the chance of visible edge turnover during very fast camera motion

## `ParallaxDebugSettings`

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `enabled` | `bool` | `false` | Turns all parallax gizmo drawing on or off |
| `draw_viewport_bounds` | `bool` | `true` | Draws the current bound-camera viewport rectangle |
| `draw_layer_bounds` | `bool` | `true` | Draws both effective coverage and wrap-span rectangles |
| `draw_offsets` | `bool` | `false` | Draws a line from the rig-local origin to each resolved layer center |
| `viewport_color` | `Color` | cyan-ish alpha | Gizmo color for viewport bounds |
| `coverage_color` | `Color` | green alpha | Gizmo color for effective tiled or segmented coverage |
| `wrap_color` | `Color` | amber alpha | Gizmo color for wrap spans |
| `offset_color` | `Color` | red alpha | Gizmo color for resolved offset vectors |

## Practical Tuning Patterns

### Background Sky

- `camera_factor = Vec2::ONE`
- `repeat = both`
- `strategy = TiledSprite`
- `coverage_margin` modest

This keeps the sky screen-locked while still allowing slow auto-scroll if desired.

### Midground Mountains

- `camera_factor.x = 0.8 - 0.95`
- `repeat = horizontal`
- `strategy = Segmented`

This keeps the strip readable while letting the camera motion sell depth.

### Foreground Foliage

- `camera_factor.x = 1.02 - 1.15`
- `repeat = horizontal`
- `strategy = Segmented`

Values above `1` exaggerate motion and make the layer feel close to the camera.

### Perspective 2.5D Cards

- `camera_factor = Vec2::ZERO`
- `depth_mapping = Some(ParallaxDepthMapping::default())`
- `depth = -8.0 .. -2.0` for background cards
- `depth = 2.0 .. 6.0` for foreground cards

This mode makes the layer respond to dolly/zoom moves using its physical plane depth instead of a hand-authored heuristic factor.

### Pixel-Art Clouds

- `snap = Pixel`
- nearest-neighbor textures
- even-valued `source_size`

This minimizes shimmer during slow camera drift.

### Multi-Layer Atmospheric Scene (Forest / City)

For a convincing depth scene with 5-7 layers, use **atmospheric perspective** — far layers are lighter/bluer, near layers are darker/more saturated:

| Layer | camera_factor.x | Strategy | Tint/Color Notes |
|-------|----------------|----------|-----------------|
| Sky | 0.05 - 0.10 | Tiled | Full-screen gradient fill |
| Far mountains | 0.15 - 0.25 | Segmented | Light, desaturated, blue-tinted |
| Near mountains | 0.30 - 0.40 | Segmented | Medium tone, less blue |
| Far trees/buildings | 0.50 - 0.60 | Segmented | Medium-dark, subtle blue |
| Mid trees/buildings | 0.70 - 0.80 | Segmented | Darker, more saturated |
| Near trees/buildings | 0.85 - 0.95 | Segmented | Darkest, full saturation |
| Ground | 1.00 | Tiled | Locked to camera for gameplay |

See `forest_scene` and `city_skyline` examples for complete implementations.

### Interactive / Platformer Parallax

For camera-follow parallax (no auto-scroll):

- Set up a `ParallaxCameraTarget` bound to the gameplay camera
- Move the camera with a smooth lerp following the player
- The parallax layers respond automatically to camera position changes
- Use `camera_factor` values from 0.1 (far) to 0.95 (near) for depth

See `platformer_demo` for a complete WASD + jump implementation.

### Custom Offset Effects

For wobble, shake, or event-driven offsets, use the `ComputeOffsets`/`WriteTransforms` hook:

1. Add a system `after(ComputeOffsets).before(WriteTransforms)` that modifies `ParallaxLayerComputed`
2. Or use `user_offset` / `user_scale` on `ParallaxLayer` for simpler additive effects

See `custom_offset` example for wobble, shake, and burst implementations.

### Endless Runner Speed Ramp

- Set `auto_scroll` on each layer for base speed
- Use `ParallaxRig::speed_multiplier` to increase difficulty over time
- Camera-factor ratios stay unchanged; only auto-scroll rates scale

See `speed_ramp` example.

### Pause / Slow-Mo

- Set `ParallaxTimeScale(0.0)` to freeze all auto-scroll globally
- Set `ParallaxTimeScale(0.25)` for slow-motion
- Use `ParallaxRig::speed_multiplier = 0.0` to freeze a specific rig

See `time_control` example.

## `ParallaxTimeScale`

Global resource controlling auto-scroll time.

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `0` | `f32` | `1.0` | Multiplies `dt` for all auto-scroll accumulation. `0.0` = paused, `1.0` = normal |

Applied on top of per-rig `speed_multiplier`: `effective_dt = delta_secs * time_scale * rig.speed_multiplier`.

## `ParallaxLayerComputed`

Intermediate component written by `ComputeOffsets`, read by `WriteTransforms`. This is the hook point for custom offset logic.

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `offset` | `Vec2` | `Vec2::ZERO` | Computed parallax offset (before `user_offset` is added) |
| `scale` | `Vec2` | `Vec2::ONE` | Computed effective scale (before `user_scale` is multiplied) |
| `depth` | `f32` | `0.0` | Layer depth, passed through for convenience |

## `RigRuntimeState`

Public per-rig component populated by `TrackCamera`. Read-only for consumers.

| Field | Type | Effect |
|-------|------|--------|
| `camera_target` | `Option<Entity>` | Resolved camera entity |
| `camera_position` | `Vec2` | Camera world position (2D) |
| `camera_depth` | `f32` | Camera Z depth |
| `camera_is_perspective` | `bool` | Whether the camera uses perspective projection |
| `viewport_size` | `Vec2` | Viewport size in world units |

## `LayerRuntimeState`

Public per-layer component populated by `ComputeOffsets`. Read-only for consumers.

| Field | Type | Effect |
|-------|------|--------|
| `effective_camera_factor` | `Vec2` | Final camera factor after depth mapping |
| `effective_scale` | `Vec2` | Final scale after depth mapping |
| `auto_phase` | `Vec2` | Accumulated auto-scroll offset |
| `depth_ratio` | `Option<f32>` | Perspective depth ratio (if applicable) |
| `effective_offset` | `Vec2` | Final snapped offset |
| `wrap_span` | `Vec2` | One-tile/segment world size for wrapping |
| `coverage_size` | `Vec2` | Total coverage area |
| `segment_grid` | `UVec2` | Segment grid dimensions |
