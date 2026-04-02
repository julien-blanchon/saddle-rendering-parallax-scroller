# Configuration

## `ParallaxRig`

| Field | Type | Default | Effect |
|-------|------|---------|--------|
| `enabled` | `bool` | `true` | Freezes camera tracking and layer layout for the rig when `false` |
| `origin` | `Vec2` | `Vec2::ZERO` | Extra rig-local translation added to every child layer |

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
| `repeat` | `ParallaxAxes` | `both()` | Enables wrap math per axis |
| `bounds` | `ParallaxBounds` | none | Clamp range for non-repeating axes |
| `snap` | `ParallaxSnap` | `None` | Rounds final offsets for pixel-stable output |
| `coverage_margin` | `Vec2` | `Vec2::ZERO` | Overscan added beyond the viewport on repeating axes |
| `source_size` | `Option<Vec2>` | `None` | One tile or one segment size before `scale`; derived from sprite image/rect when omitted |
| `scale` | `Vec2` | `Vec2::ONE` | Written to `Transform.scale.xy`; affects rendered size and wrap span |
| `tint` | `Color` | `Color::WHITE` | Written to `Sprite.color` |
| `strategy` | `ParallaxLayerStrategy` | `TiledSprite` | Chooses tiling vs segment cloning |

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

### Pixel-Art Clouds

- `snap = Pixel`
- nearest-neighbor textures
- even-valued `source_size`

This minimizes shimmer during slow camera drift.
